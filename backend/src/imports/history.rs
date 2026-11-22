use crate::{
    AppState, auth, domain,
    error::{AppError, Result},
};
use axum::{
    Json,
    extract::{Path, State},
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::{FromRow, SqliteConnection};
use std::collections::HashSet;
use tower_sessions::Session;

#[derive(Serialize, FromRow)]
pub struct Batch {
    id: String,
    filename: String,
    source_name: String,
    created_at: String,
    start_date: Option<String>,
    end_date: Option<String>,
    added: i64,
    matched: i64,
    known: i64,
    excluded: i64,
    removed: i64,
}
const BATCH_SELECT: &str = "SELECT b.id,b.filename,s.name AS source_name,b.created_at,b.start_date,b.end_date,b.added,b.matched,b.known,b.excluded,b.removed FROM import_batches b JOIN transaction_sources s ON s.id=b.source_id AND s.user_id=b.user_id";
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Page {
    page: Option<u32>,
}
#[derive(Serialize)]
pub struct History {
    batches: Vec<Batch>,
    page: u32,
    pages: u32,
}
pub async fn list(
    State(state): State<AppState>,
    session: Session,
    axum::extract::Query(input): axum::extract::Query<Page>,
) -> Result<Json<History>> {
    let user = auth::user(&session, &state.pool).await?;
    if input.page == Some(0) {
        return Err(AppError::invalid("Use a positive page number."));
    }
    let mut tx = state.pool.begin().await?;
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM import_batches WHERE user_id=?")
        .bind(&user.id)
        .fetch_one(&mut *tx)
        .await?;
    let pages = ((count as u32).div_ceil(20)).max(1);
    let page = input.page.unwrap_or(1).min(pages);
    let batches = sqlx::query_as::<_, Batch>(&format!(
        "{BATCH_SELECT} WHERE b.user_id=? ORDER BY b.created_at DESC,b.id DESC LIMIT 20 OFFSET ?"
    ))
    .bind(&user.id)
    .bind(i64::from(page - 1) * 20)
    .fetch_all(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(Json(History {
        batches,
        page,
        pages,
    }))
}
#[derive(Serialize, FromRow)]
struct Member {
    transaction_id: Option<String>,
    revision: Option<i64>,
    original_revision: i64,
    overlapping: bool,
    date: String,
    description: String,
    amount_cents: String,
}
impl Member {
    fn edited(&self) -> bool {
        self.revision != Some(self.original_revision)
    }
    fn preview(&self) -> Option<EntryPreview> {
        self.transaction_id.as_ref().map(|id| EntryPreview {
            id: id.clone(),
            date: self.date.clone(),
            description: self.description.clone(),
            amount_cents: self.amount_cents.clone(),
            overlapping: self.overlapping,
        })
    }
}
#[derive(Serialize)]
pub struct EntryPreview {
    id: String,
    date: String,
    description: String,
    amount_cents: String,
    overlapping: bool,
}

#[derive(Serialize)]
pub struct Review {
    batch: Batch,
    removable: usize,
    edited: usize,
    overlapping: usize,
    already_deleted: usize,
    token: String,
    removable_entries: Vec<EntryPreview>,
    edited_entries: Vec<EntryPreview>,
}
async fn review(
    conn: &mut SqliteConnection,
    owner: &str,
    id: &str,
) -> Result<(Review, Vec<Member>)> {
    let batch = sqlx::query_as::<_, Batch>(&format!("{BATCH_SELECT} WHERE b.user_id=? AND b.id=?"))
        .bind(owner)
        .bind(id)
        .fetch_optional(&mut *conn)
        .await?
        .ok_or_else(AppError::not_found)?;
    let members=sqlx::query_as::<_,Member>("SELECT e.transaction_id,t.revision,e.revision AS original_revision,
       COALESCE(t.date,'') AS date,COALESCE(t.description,'') AS description,CAST(COALESCE(t.amount_cents,0) AS TEXT) AS amount_cents,
       EXISTS(SELECT 1 FROM import_batch_entries other WHERE other.user_id=e.user_id AND other.transaction_id=e.transaction_id AND other.batch_id<>e.batch_id) AS overlapping
       FROM import_batch_entries e LEFT JOIN transactions t ON t.id=e.transaction_id AND t.user_id=e.user_id
       WHERE e.user_id=? AND e.batch_id=? AND e.role='added' ORDER BY t.date DESC,t.created_at DESC,t.id DESC,e.rowid")
       .bind(owner).bind(id).fetch_all(conn).await?;
    let mut result = Review {
        batch,
        removable: 0,
        edited: 0,
        overlapping: 0,
        already_deleted: 0,
        token: String::new(),
        removable_entries: Vec::new(),
        edited_entries: Vec::new(),
    };
    for m in &members {
        if m.transaction_id.is_none() {
            result.already_deleted += 1;
        } else if let Some(entry) = m.preview() {
            if m.overlapping {
                result.overlapping += 1;
            }
            if m.edited() {
                result.edited += 1;
                result.edited_entries.push(entry);
            } else {
                result.removable += 1;
                result.removable_entries.push(entry);
            }
        }
    }
    result.token = format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(&members).map_err(AppError::internal)?)
    );
    Ok((result, members))
}
pub async fn detail(
    State(state): State<AppState>,
    session: Session,
    Path(id): Path<String>,
) -> Result<Json<Review>> {
    let user = auth::user(&session, &state.pool).await?;
    let mut tx = state.pool.begin().await?;
    Ok(Json(review(&mut tx, &user.id, &id).await?.0))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Removal {
    token: String,
    #[serde(default)]
    selected_edited_ids: Vec<String>,
}
pub async fn remove(
    State(state): State<AppState>,
    session: Session,
    Path(id): Path<String>,
    Json(input): Json<Removal>,
) -> Result<Json<serde_json::Value>> {
    let user = auth::user(&session, &state.pool).await?;
    let mut tx = state.pool.begin_with("BEGIN IMMEDIATE").await?;
    let (current, members) = review(&mut tx, &user.id, &id).await?;
    if input.token != current.token {
        return Err(AppError::invalid(
            "Entries changed since this review. Reload the batch before removing entries.",
        ));
    }
    let selected: HashSet<_> = input.selected_edited_ids.iter().collect();
    if selected.len() != input.selected_edited_ids.len()
        || selected.len() > 1000
        || selected.iter().any(|id| {
            !members
                .iter()
                .any(|m| m.edited() && m.transaction_id.as_ref() == Some(*id))
        })
    {
        return Err(AppError::invalid(
            "Select distinct edited entries from this batch's current review.",
        ));
    }
    let mut removed = 0;
    for m in members {
        if (!m.edited()
            || m.transaction_id
                .as_ref()
                .is_some_and(|id| selected.contains(id)))
            && let Some(id) = m.transaction_id
        {
            removed += sqlx::query("DELETE FROM transactions WHERE user_id=? AND id=?")
                .bind(&user.id)
                .bind(id)
                .execute(&mut *tx)
                .await?
                .rows_affected();
        }
    }
    sqlx::query("UPDATE import_batches SET removed=removed+? WHERE user_id=? AND id=?")
        .bind(removed as i64)
        .bind(&user.id)
        .bind(id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(Json(serde_json::json!({"removed":removed})))
}
pub(super) async fn link(
    conn: &mut SqliteConnection,
    batch: &str,
    user: &str,
    id: &str,
    role: &str,
) -> Result<()> {
    sqlx::query("INSERT INTO import_batch_entries(batch_id,user_id,transaction_id,role,revision) SELECT ?,user_id,id,?,revision FROM transactions WHERE user_id=? AND id=? AND NOT EXISTS(SELECT 1 FROM import_batch_entries WHERE batch_id=? AND transaction_id=?)")
        .bind(batch).bind(role).bind(user).bind(id).bind(batch).bind(id).execute(conn).await?;
    Ok(())
}
pub(super) fn filename(value: Option<&str>) -> Result<String> {
    // Keep only a display basename; raw files and local paths are never stored.
    domain::text(
        value
            .unwrap_or("CSV import")
            .rsplit(['/', '\\'])
            .next()
            .unwrap_or("CSV import"),
        "Filename",
        160,
    )
}
