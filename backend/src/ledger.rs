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
use sqlx::{FromRow, SqliteConnection, SqlitePool};
use tower_sessions::Session;
use uuid::Uuid;

#[derive(FromRow)]
struct AccountRow {
    id: String,
    name: String,
    opening_date: String,
    opening_cents: i64,
}

#[derive(Serialize)]
pub struct Account {
    id: String,
    name: String,
    opening_date: String,
    opening_cents: String,
    balance_cents: String,
    latest_transaction_date: Option<String>,
}

#[derive(Deserialize)]
pub struct NewAccount {
    name: String,
    opening_date: String,
    opening_balance: String,
}

async fn owned_account(pool: &SqlitePool, owner: &str, id: &str) -> Result<AccountRow> {
    sqlx::query_as::<_, AccountRow>(
        "SELECT id, name, opening_date, opening_cents FROM accounts WHERE id = ? AND user_id = ?",
    )
    .bind(id)
    .bind(owner)
    .fetch_optional(pool)
    .await?
    .ok_or_else(AppError::not_found)
}

pub async fn accounts(
    State(state): State<AppState>,
    session: Session,
) -> Result<Json<Vec<Account>>> {
    let user = auth::user(&session, &state.pool).await?;
    let rows = sqlx::query_as::<_, AccountRow>("SELECT id, name, opening_date, opening_cents FROM accounts WHERE user_id = ? ORDER BY created_at, id")
        .bind(&user.id).fetch_all(&state.pool).await?;
    let mut result = Vec::with_capacity(rows.len());
    for row in rows {
        let amounts: Vec<(i64, String)> = sqlx::query_as(
            "SELECT amount_cents, date FROM transactions WHERE account_id = ? AND user_id = ?",
        )
        .bind(&row.id)
        .bind(&user.id)
        .fetch_all(&state.pool)
        .await?;
        let balance = domain::checked_total(
            std::iter::once(row.opening_cents).chain(amounts.iter().map(|(amount, _)| *amount)),
        )?;
        result.push(Account {
            id: row.id,
            name: row.name,
            opening_date: row.opening_date,
            opening_cents: row.opening_cents.to_string(),
            balance_cents: balance.to_string(),
            latest_transaction_date: amounts.iter().map(|(_, date)| date).max().cloned(),
        });
    }
    Ok(Json(result))
}

pub async fn create_account(
    State(state): State<AppState>,
    session: Session,
    Json(input): Json<NewAccount>,
) -> Result<(StatusCode, Json<serde_json::Value>)> {
    let user = auth::user(&session, &state.pool).await?;
    let name = domain::text(&input.name, "Account name", 80)?;
    let opening_date = domain::date(&input.opening_date)?.to_string();
    let opening_cents = domain::parse_money(&input.opening_balance)?;
    let id = Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO accounts (id, user_id, name, opening_date, opening_cents) VALUES (?, ?, ?, ?, ?)")
        .bind(&id).bind(user.id).bind(name).bind(opening_date).bind(opening_cents).execute(&state.pool).await?;
    Ok((StatusCode::CREATED, Json(serde_json::json!({ "id": id }))))
}

#[derive(Deserialize)]
pub struct Filter {
    month: Option<String>,
    account_id: Option<String>,
}

impl Filter {
    async fn bounds(&self, pool: &SqlitePool, owner: &str) -> Result<(String, String, String)> {
        let month = self
            .month
            .clone()
            .unwrap_or_else(|| Utc::now().format("%Y-%m").to_string());
        let (start, end) = domain::month_range(&month)?;
        if let Some(id) = &self.account_id {
            owned_account(pool, owner, id).await?;
        }
        Ok((month, start, end))
    }
}

#[derive(FromRow)]
struct TransactionRow {
    id: String,
    account_id: String,
    account_name: String,
    date: String,
    description: String,
    amount_cents: i64,
}

#[derive(Serialize)]
pub struct Transaction {
    id: String,
    account_id: String,
    account_name: String,
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
        "SELECT t.id, t.account_id, a.name AS account_name, t.date, t.description, t.amount_cents
         FROM transactions t JOIN accounts a ON a.id = t.account_id AND a.user_id = t.user_id
         WHERE t.user_id = ? AND t.date >= ? AND t.date < ? AND (? IS NULL OR t.account_id = ?)
         ORDER BY t.date DESC, t.created_at DESC, t.id DESC",
    )
    .bind(owner)
    .bind(start)
    .bind(end)
    .bind(&filter.account_id)
    .bind(&filter.account_id)
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
                account_id: row.account_id,
                account_name: row.account_name,
                date: row.date,
                description: row.description,
                amount_cents: row.amount_cents.to_string(),
            })
            .collect(),
    ))
}

#[derive(Deserialize)]
pub struct NewTransaction {
    account_id: String,
    date: String,
    description: String,
    amount: String,
    kind: domain::Kind,
}

impl NewTransaction {
    async fn validate(&self, pool: &SqlitePool, owner: &str) -> Result<(String, i64)> {
        let account = owned_account(pool, owner, &self.account_id).await?;
        let date = domain::date(&self.date)?;
        if date.to_string() < account.opening_date {
            return Err(AppError::invalid(
                "The transaction date must be on or after the account's opening date.",
            ));
        }
        Ok((
            domain::text(&self.description, "Description", 200)?,
            self.kind.amount(&self.amount)?,
        ))
    }
}

async fn validate_balance(
    conn: &mut SqliteConnection,
    owner: &str,
    account_id: &str,
) -> Result<()> {
    let opening: i64 =
        sqlx::query_scalar("SELECT opening_cents FROM accounts WHERE id = ? AND user_id = ?")
            .bind(account_id)
            .bind(owner)
            .fetch_one(&mut *conn)
            .await?;
    let amounts: Vec<i64> = sqlx::query_scalar(
        "SELECT amount_cents FROM transactions WHERE account_id = ? AND user_id = ?",
    )
    .bind(account_id)
    .bind(owner)
    .fetch_all(&mut *conn)
    .await?;
    domain::checked_total(std::iter::once(opening).chain(amounts))?;
    Ok(())
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
    sqlx::query("INSERT INTO transactions (id, user_id, account_id, date, description, amount_cents) VALUES (?, ?, ?, ?, ?, ?)")
        .bind(&id).bind(&user.id).bind(&input.account_id).bind(input.date).bind(description).bind(amount).execute(&mut *tx).await?;
    validate_balance(&mut tx, &user.id, &input.account_id).await?;
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
    let old_account: String =
        sqlx::query_scalar("SELECT account_id FROM transactions WHERE id = ? AND user_id = ?")
            .bind(&id)
            .bind(&user.id)
            .fetch_optional(&mut *tx)
            .await?
            .ok_or_else(AppError::not_found)?;
    let result = sqlx::query("UPDATE transactions SET account_id = ?, date = ?, description = ?, amount_cents = ? WHERE id = ? AND user_id = ?")
        .bind(&input.account_id).bind(input.date).bind(description).bind(amount).bind(id).bind(&user.id).execute(&mut *tx).await?;
    if result.rows_affected() == 0 {
        return Err(AppError::not_found());
    }
    validate_balance(&mut tx, &user.id, &old_account).await?;
    if old_account != input.account_id {
        validate_balance(&mut tx, &user.id, &input.account_id).await?;
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
    // RETURNING captures the account from the row actually changed, even with concurrent requests.
    let account: String = sqlx::query_scalar(
        "DELETE FROM transactions WHERE id = ? AND user_id = ? RETURNING account_id",
    )
    .bind(id)
    .bind(&user.id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(AppError::not_found)?;
    validate_balance(&mut tx, &user.id, &account).await?;
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
