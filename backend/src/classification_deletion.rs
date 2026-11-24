use crate::{
    AppState, auth,
    error::{AppError, Result},
    rules,
};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::SqliteConnection;
use tower_sessions::Session;

#[derive(Serialize)]
pub struct Preview {
    entry_count: usize,
    rule_count: usize,
    token: String,
}
async fn review(conn: &mut SqliteConnection, owner: &str, id: &str) -> Result<Preview> {
    let item: (String, String, String, bool) = sqlx::query_as(
        "SELECT kind,name,color,archived FROM classifications WHERE id=? AND user_id=?",
    )
    .bind(id)
    .bind(owner)
    .fetch_optional(&mut *conn)
    .await?
    .ok_or_else(AppError::not_found)?;
    let sql = if item.0 == "category" {
        "SELECT t.id,t.revision FROM transactions t JOIN entry_categories e ON e.transaction_id=t.id AND e.user_id=t.user_id WHERE e.user_id=? AND e.category_id=? ORDER BY t.id"
    } else {
        "SELECT t.id,t.revision FROM transactions t JOIN entry_labels e ON e.transaction_id=t.id AND e.user_id=t.user_id WHERE e.user_id=? AND e.label_id=? ORDER BY t.id"
    };
    let entries: Vec<(String, i64)> = sqlx::query_as(sql)
        .bind(owner)
        .bind(id)
        .fetch_all(&mut *conn)
        .await?;
    let rules = rules::affected_rules(conn, owner, id).await?;
    let bytes = serde_json::to_vec(&(id, item, &entries, &rules)).map_err(AppError::internal)?;
    Ok(Preview {
        entry_count: entries.len(),
        rule_count: rules.len(),
        token: format!("{:x}", Sha256::digest(bytes)),
    })
}
pub async fn preview(
    State(state): State<AppState>,
    session: Session,
    Path(id): Path<String>,
) -> Result<Json<Preview>> {
    let user = auth::user(&session, &state.pool).await?;
    let mut tx = state.pool.begin().await?;
    let result = review(&mut tx, &user.id, &id).await?;
    tx.commit().await?;
    Ok(Json(result))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Confirmation {
    token: String,
}
pub async fn delete(
    State(state): State<AppState>,
    session: Session,
    Path(id): Path<String>,
    Json(input): Json<Confirmation>,
) -> Result<StatusCode> {
    let user = auth::user(&session, &state.pool).await?;
    let mut tx = state.pool.begin_with("BEGIN IMMEDIATE").await?;
    let current = review(&mut tx, &user.id, &id).await?;
    if current.token != input.token {
        return Err(AppError::invalid(
            "The affected entries or rules changed. Review deletion again.",
        ));
    }
    rules::pause_target(&mut tx, &user.id, &id, true).await?;
    sqlx::query("DELETE FROM entry_categories WHERE user_id=? AND category_id=?")
        .bind(&user.id)
        .bind(&id)
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM entry_labels WHERE user_id=? AND label_id=?")
        .bind(&user.id)
        .bind(&id)
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM classifications WHERE user_id=? AND id=?")
        .bind(&user.id)
        .bind(&id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}
