use crate::{
    AppState, auth, domain,
    error::{AppError, Result},
};
use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};
use tower_sessions::Session;
use uuid::Uuid;

#[derive(Serialize, FromRow)]
pub struct Source {
    pub id: String,
    name: String,
    latest_transaction_date: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NewSource {
    name: String,
}

pub(crate) async fn owned_source(pool: &SqlitePool, owner: &str, id: &str) -> Result<()> {
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM transaction_sources WHERE id = ? AND user_id = ?)",
    )
    .bind(id)
    .bind(owner)
    .fetch_one(pool)
    .await?;
    if !exists {
        return Err(AppError::not_found());
    }
    Ok(())
}

pub async fn sources(State(state): State<AppState>, session: Session) -> Result<Json<Vec<Source>>> {
    let user = auth::user(&session, &state.pool).await?;
    let rows = sqlx::query_as::<_, Source>(
        "SELECT a.id, a.name, (SELECT MAX(t.date) FROM transactions t WHERE t.source_id = a.id AND t.user_id = a.user_id) AS latest_transaction_date FROM transaction_sources a WHERE a.user_id = ? ORDER BY a.created_at, a.id")
        .bind(user.id).fetch_all(&state.pool).await?;
    Ok(Json(rows))
}

pub async fn create_source(
    State(state): State<AppState>,
    session: Session,
    Json(input): Json<NewSource>,
) -> Result<(StatusCode, Json<serde_json::Value>)> {
    let user = auth::user(&session, &state.pool).await?;
    let name = domain::text(&input.name, "Source name", 80)?;
    let id = Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO transaction_sources (id, user_id, name) VALUES (?, ?, ?)")
        .bind(&id)
        .bind(user.id)
        .bind(name)
        .execute(&state.pool)
        .await?;
    Ok((StatusCode::CREATED, Json(serde_json::json!({ "id": id }))))
}

#[derive(Deserialize)]
pub struct Filter {
    month: Option<String>,
    source_id: Option<String>,
}

impl Filter {
    async fn bounds(&self, pool: &SqlitePool, owner: &str) -> Result<(String, String, String)> {
        let month = self
            .month
            .clone()
            .unwrap_or_else(|| Utc::now().format("%Y-%m").to_string());
        let (start, end) = domain::month_range(&month)?;
        if let Some(id) = &self.source_id {
            owned_source(pool, owner, id).await?;
        }
        Ok((month, start, end))
    }
}

#[derive(FromRow)]
struct TransactionRow {
    id: String,
    source_id: Option<String>,
    source_name: Option<String>,
    date: String,
    description: String,
    amount_cents: i64,
}

#[derive(Serialize)]
pub struct Transaction {
    id: String,
    source_id: Option<String>,
    source_name: Option<String>,
    date: String,
    description: String,
    amount_cents: String,
}

async fn filtered_rows(
    pool: &SqlitePool,
    owner: &str,
    filter: &Filter,
) -> Result<(String, Vec<TransactionRow>)> {
    let (month, start, end) = filter.bounds(pool, owner).await?;
    let rows = sqlx::query_as::<_, TransactionRow>(
        "SELECT t.id, t.source_id, a.name AS source_name, t.date, t.description, t.amount_cents
         FROM transactions t LEFT JOIN transaction_sources a ON a.id = t.source_id AND a.user_id = t.user_id
         WHERE t.user_id = ? AND t.date >= ? AND t.date < ? AND (? IS NULL OR t.source_id = ?)
         ORDER BY t.date DESC, t.created_at DESC, t.id DESC",
    )
    .bind(owner)
    .bind(start)
    .bind(end)
    .bind(&filter.source_id)
    .bind(&filter.source_id)
    .fetch_all(pool)
    .await?;
    Ok((month, rows))
}

pub async fn transactions(
    State(state): State<AppState>,
    session: Session,
    Query(filter): Query<Filter>,
) -> Result<Json<Vec<Transaction>>> {
    let user = auth::user(&session, &state.pool).await?;
    let (_, rows) = filtered_rows(&state.pool, &user.id, &filter).await?;
    Ok(Json(
        rows.into_iter()
            .map(|row| Transaction {
                id: row.id,
                source_id: row.source_id,
                source_name: row.source_name,
                date: row.date,
                description: row.description,
                amount_cents: row.amount_cents.to_string(),
            })
            .collect(),
    ))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NewTransaction {
    source_id: Option<String>,
    date: String,
    description: String,
    amount: String,
    kind: domain::Kind,
}

impl NewTransaction {
    async fn validate(&self, pool: &SqlitePool, owner: &str) -> Result<(String, i64)> {
        if let Some(id) = &self.source_id {
            owned_source(pool, owner, id).await?;
        }
        domain::date(&self.date)?;
        Ok((
            domain::text(&self.description, "Description", 200)?,
            self.kind.amount(&self.amount)?,
        ))
    }
}

pub async fn create_transaction(
    State(state): State<AppState>,
    session: Session,
    Json(input): Json<NewTransaction>,
) -> Result<(StatusCode, Json<serde_json::Value>)> {
    let user = auth::user(&session, &state.pool).await?;
    let (description, amount) = input.validate(&state.pool, &user.id).await?;
    let id = Uuid::new_v4().to_string();
    let mut tx = state.pool.begin_with("BEGIN IMMEDIATE").await?;
    sqlx::query("INSERT INTO transactions (id, user_id, source_id, date, description, amount_cents) VALUES (?, ?, ?, ?, ?, ?)")
        .bind(&id).bind(&user.id).bind(&input.source_id).bind(input.date).bind(description).bind(amount).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok((StatusCode::CREATED, Json(serde_json::json!({ "id": id }))))
}

pub async fn update_transaction(
    State(state): State<AppState>,
    session: Session,
    Path(id): Path<String>,
    Json(input): Json<NewTransaction>,
) -> Result<StatusCode> {
    let user = auth::user(&session, &state.pool).await?;
    let (description, amount) = input.validate(&state.pool, &user.id).await?;
    let mut tx = state.pool.begin_with("BEGIN IMMEDIATE").await?;
    let result = sqlx::query("UPDATE transactions SET source_id = ?, date = ?, description = ?, amount_cents = ? WHERE id = ? AND user_id = ?")
        .bind(&input.source_id).bind(input.date).bind(description).bind(amount).bind(id).bind(&user.id).execute(&mut *tx).await?;
    if result.rows_affected() == 0 {
        return Err(AppError::not_found());
    }
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn delete_transaction(
    State(state): State<AppState>,
    session: Session,
    Path(id): Path<String>,
) -> Result<StatusCode> {
    let user = auth::user(&session, &state.pool).await?;
    let mut tx = state.pool.begin_with("BEGIN IMMEDIATE").await?;
    let _deleted: String =
        sqlx::query_scalar("DELETE FROM transactions WHERE id = ? AND user_id = ? RETURNING id")
            .bind(id)
            .bind(&user.id)
            .fetch_optional(&mut *tx)
            .await?
            .ok_or_else(AppError::not_found)?;
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Serialize)]
pub struct Summary {
    month: String,
    income_cents: String,
    spending_cents: String,
    net_cents: String,
    transaction_count: usize,
}

pub async fn summary(
    State(state): State<AppState>,
    session: Session,
    Query(filter): Query<Filter>,
) -> Result<Json<Summary>> {
    let user = auth::user(&session, &state.pool).await?;
    let (month, rows) = filtered_rows(&state.pool, &user.id, &filter).await?;
    let income = domain::checked_total(
        rows.iter()
            .filter(|row| row.amount_cents > 0)
            .map(|row| row.amount_cents),
    )?;
    let spending = domain::checked_total(
        rows.iter()
            .filter(|row| row.amount_cents < 0)
            .map(|row| -row.amount_cents),
    )?;
    let net = domain::checked_total([income, -spending])?;
    Ok(Json(Summary {
        month,
        income_cents: income.to_string(),
        spending_cents: spending.to_string(),
        net_cents: net.to_string(),
        transaction_count: rows.len(),
    }))
}
