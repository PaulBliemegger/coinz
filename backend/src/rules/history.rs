use super::{Definition, Rule, snapshot, validate};
use crate::{
    AppState, auth, classification,
    error::{AppError, Result},
};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::{FromRow, SqliteConnection};
use tower_sessions::Session;

#[derive(Serialize, FromRow)]
pub struct Example {
    id: String,
    date: String,
    description: String,
    amount_cents: String,
    revision: i64,
}
async fn examples(conn: &mut SqliteConnection, owner: &str) -> Result<Vec<Example>> {
    Ok(sqlx::query_as("SELECT id,date,description,CAST(amount_cents AS TEXT) AS amount_cents,revision FROM transactions WHERE user_id=? ORDER BY date DESC,id").bind(owner).fetch_all(conn).await?)
}
#[derive(Serialize)]
pub struct Preview {
    match_count: usize,
    examples: Vec<Example>,
}
pub async fn preview_definition(
    State(state): State<AppState>,
    session: Session,
    Json(input): Json<Definition>,
) -> Result<Json<Preview>> {
    let user = auth::user(&session, &state.pool).await?;
    let mut tx = state.pool.begin().await?;
    let (_, pattern) = validate(&mut tx, &user.id, &input).await?;
    let entries: Vec<Example> = sqlx::query_as("SELECT id,date,description,CAST(amount_cents AS TEXT) AS amount_cents,revision FROM transactions WHERE user_id=? AND (? IS NULL OR source_id=?) ORDER BY date DESC,id").bind(&user.id).bind(&input.source_id).bind(&input.source_id).fetch_all(&mut *tx).await?;
    let matches: Vec<_> = entries
        .into_iter()
        .filter(|entry| input.operator.matches(&pattern, &entry.description))
        .collect();
    let result = Preview {
        match_count: matches.len(),
        examples: matches.into_iter().take(10).collect(),
    };
    tx.commit().await?;
    Ok(Json(result))
}
#[derive(Serialize)]
pub struct Change {
    entry: Example,
    category_id: Option<String>,
    label_ids: Vec<String>,
}
#[derive(Serialize)]
pub struct ApplyPreview {
    affected_entries: usize,
    token: String,
    examples: Vec<Change>,
}
async fn changes(conn: &mut SqliteConnection, owner: &str, rule: &Rule) -> Result<Vec<Change>> {
    if !rule.enabled {
        return Err(AppError::invalid(
            "Enable this rule before applying it to history.",
        ));
    }
    let mut result = Vec::new();
    for entry in examples(conn, owner).await? {
        let source: Option<String> =
            sqlx::query_scalar("SELECT source_id FROM transactions WHERE user_id=? AND id=?")
                .bind(owner)
                .bind(&entry.id)
                .fetch_one(&mut *conn)
                .await?;
        if !rule.matches(&entry.description, source.as_deref()) {
            continue;
        }
        let category = classification::entry_category(conn, owner, &entry.id).await?;
        let labels = classification::entry_labels(conn, owner, &entry.id).await?;
        let category_id = if category.is_none() {
            rule.category_id.clone()
        } else {
            None
        };
        let label_ids: Vec<_> = rule
            .label_ids
            .iter()
            .filter(|id| !labels.iter().any(|l| &l.id == *id))
            .cloned()
            .collect();
        if category_id.is_some() || !label_ids.is_empty() {
            result.push(Change {
                entry,
                category_id,
                label_ids,
            });
        }
    }
    Ok(result)
}
async fn review(
    conn: &mut SqliteConnection,
    owner: &str,
    id: &str,
) -> Result<(String, Vec<Change>)> {
    let snapshot = snapshot(conn, owner).await?;
    let rule = snapshot
        .rules
        .iter()
        .find(|r| r.id == id)
        .ok_or_else(AppError::not_found)?;
    let changes = changes(conn, owner, rule).await?;
    let bytes = serde_json::to_vec(&(snapshot.revision, rule.id.as_str(), rule.revision, &changes))
        .map_err(AppError::internal)?;
    Ok((format!("{:x}", Sha256::digest(bytes)), changes))
}
pub async fn preview_apply(
    State(state): State<AppState>,
    session: Session,
    Path(id): Path<String>,
) -> Result<Json<ApplyPreview>> {
    let user = auth::user(&session, &state.pool).await?;
    let mut tx = state.pool.begin().await?;
    let (token, changes) = review(&mut tx, &user.id, &id).await?;
    let result = ApplyPreview {
        affected_entries: changes.len(),
        token,
        examples: changes.into_iter().take(10).collect(),
    };
    tx.commit().await?;
    Ok(Json(result))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Confirmation {
    pub token: String,
}
pub async fn apply(
    State(state): State<AppState>,
    session: Session,
    Path(id): Path<String>,
    Json(input): Json<Confirmation>,
) -> Result<StatusCode> {
    let user = auth::user(&session, &state.pool).await?;
    let mut tx = state.pool.begin_with("BEGIN IMMEDIATE").await?;
    let (token, changes) = review(&mut tx, &user.id, &id).await?;
    if token != input.token {
        return Err(AppError::invalid(
            "The rule or matching entries changed. Preview history again.",
        ));
    }
    for change in changes {
        if let Some(id) = change.category_id {
            classification::set_category(&mut tx, &user.id, &change.entry.id, Some(&id)).await?;
        }
        for id in change.label_ids {
            classification::change_label(&mut tx, &user.id, &change.entry.id, &id, true).await?;
        }
    }
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}
