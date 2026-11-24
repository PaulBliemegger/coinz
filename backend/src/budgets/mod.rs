use axum::{
    Json,
    extract::{Path, Query, State},
};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::collections::HashSet;
use tower_sessions::Session;
use uuid::Uuid;

use crate::{
    AppState, auth, classification, domain,
    error::{AppError, Result},
};
mod report;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AsOf {
    as_of: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlanInput {
    revision: i64,
    savings: String,
    categories: Vec<CategoryInput>,
    income: Vec<IncomeInput>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CategoryInput {
    category_id: String,
    amount: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct IncomeInput {
    id: Option<String>,
    name: String,
    amount: String,
    complete: bool,
    transaction_ids: Vec<String>,
}
#[derive(Serialize, FromRow)]
pub struct Receipt {
    pub id: String,
    pub date: String,
    pub description: String,
    pub amount_cents: String,
}
#[derive(Serialize)]
pub struct Income {
    pub id: String,
    pub name: String,
    pub planned_cents: String,
    pub complete: bool,
    pub transaction_ids: Vec<String>,
    pub received_cents: String,
    pub pending_cents: String,
    pub invalid_receipt_count: usize,
    pub unrestored_receipt_count: usize,
}
#[derive(Serialize)]
pub struct Category {
    pub category: classification::Item,
    pub budget_cents: Option<String>,
    pub spending_cents: String,
    pub remaining_cents: Option<String>,
    pub year_spending_cents: String,
    pub year_budget_cents: String,
    pub year_budgeted_spending_cents: String,
    pub budgeted_months: usize,
}
#[derive(Serialize)]
pub struct Plan {
    pub month: String,
    pub as_of: String,
    pub revision: i64,
    pub savings_cents: String,
    pub categories: Vec<Category>,
    pub income: Vec<Income>,
    pub receipts: Vec<Receipt>,
    pub recorded_income_cents: String,
    pub spending_cents: String,
    pub unbudgeted_spending_cents: String,
    pub budget_total_cents: String,
    pub planned_income_cents: String,
    pub pending_income_cents: String,
    pub unallocated_cents: String,
    pub from_recorded_cents: String,
    pub including_pending_cents: String,
}

pub async fn get(
    State(state): State<AppState>,
    session: Session,
    Path(month): Path<String>,
    Query(query): Query<AsOf>,
) -> Result<Json<Plan>> {
    let user = auth::user(&session, &state.pool).await?;
    domain::month_range(&month)?;
    domain::date(&query.as_of)?;
    let mut tx = state.pool.begin().await?;
    let plan = report::load(&mut tx, &user.id, &month, &query.as_of).await?;
    tx.commit().await?;
    Ok(Json(plan))
}

fn nonnegative(value: &str) -> Result<i64> {
    let amount = domain::parse_money(value)?;
    if amount < 0 {
        return Err(AppError::invalid(
            "Budget amounts must be zero or positive.",
        ));
    }
    Ok(amount)
}

pub async fn save(
    State(state): State<AppState>,
    session: Session,
    Path(month): Path<String>,
    Json(input): Json<PlanInput>,
) -> Result<Json<serde_json::Value>> {
    let user = auth::user(&session, &state.pool).await?;
    let (start, end) = domain::month_range(&month)?;
    if input.categories.len() > 500 || input.income.len() > 100 {
        return Err(AppError::invalid(
            "Use up to 500 category budgets and 100 expected income items.",
        ));
    }
    let savings = nonnegative(&input.savings)?;
    let mut tx = state.pool.begin_with("BEGIN IMMEDIATE").await?;
    let current: Option<i64> =
        sqlx::query_scalar("SELECT revision FROM budget_months WHERE user_id=? AND month=?")
            .bind(&user.id)
            .bind(&month)
            .fetch_optional(&mut *tx)
            .await?;
    if current.unwrap_or(0) != input.revision {
        return Err(AppError::invalid(
            "This month's plan changed. Reload before saving your changes.",
        ));
    }
    let unrestored: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM portable_income_receipts r JOIN expected_income e ON e.id=r.expected_id AND e.user_id=r.user_id WHERE e.user_id=? AND e.month=?)").bind(&user.id).bind(&month).fetch_one(&mut *tx).await?;
    if unrestored {
        return Err(AppError::invalid(
            "Restore this month's income entries from CSV before editing its budget. Some saved receipt links are still waiting for their entries.",
        ));
    }
    let old_links: HashSet<(String,String)>=sqlx::query_as("SELECT r.transaction_id,r.expected_id FROM income_receipts r JOIN expected_income e ON e.user_id=r.user_id AND e.id=r.expected_id WHERE e.user_id=? AND e.month=?").bind(&user.id).bind(&month).fetch_all(&mut *tx).await?.into_iter().collect();
    let mut category_ids = HashSet::new();
    let mut category_amounts = Vec::new();
    for category in &input.categories {
        if !category_ids.insert(&category.category_id) {
            return Err(AppError::invalid("Choose each budget category only once."));
        }
        let item = classification::owned_item(
            &mut tx,
            &user.id,
            &category.category_id,
            classification::Kind::Category,
        )
        .await?;
        if item.archived {
            let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM category_budgets WHERE user_id=? AND month=? AND category_id=?)").bind(&user.id).bind(&month).bind(&item.id).fetch_one(&mut *tx).await?;
            if !exists {
                return Err(AppError::invalid(
                    "Restore an archived category before adding a new budget for it.",
                ));
            }
        }
        category_amounts.push(nonnegative(&category.amount)?);
    }
    let mut income_ids = HashSet::new();
    let mut receipt_ids = HashSet::new();
    let mut income_amounts = Vec::new();
    for item in &input.income {
        domain::text(&item.name, "Expected income name", 100)?;
        income_amounts.push(nonnegative(&item.amount)?);
        if let Some(id) = &item.id {
            let owned: bool = sqlx::query_scalar(
                "SELECT EXISTS(SELECT 1 FROM expected_income WHERE user_id=? AND month=? AND id=?)",
            )
            .bind(&user.id)
            .bind(&month)
            .bind(id)
            .fetch_one(&mut *tx)
            .await?;
            if !owned {
                return Err(AppError::not_found());
            }
            if !income_ids.insert(id) {
                return Err(AppError::invalid("Expected income items must be distinct."));
            }
        }
        if item.transaction_ids.len() > 500 {
            return Err(AppError::invalid(
                "Choose up to 500 receipts per income item.",
            ));
        }
        for receipt in &item.transaction_ids {
            if !receipt_ids.insert(receipt) {
                return Err(AppError::invalid(
                    "Each recorded income entry can fulfill only one expected item.",
                ));
            }
            let valid: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM transactions WHERE user_id=? AND id=? AND amount_cents>0 AND date>=? AND date<?)").bind(&user.id).bind(receipt).bind(&start).bind(&end).fetch_one(&mut *tx).await?;
            if !valid {
                return Err(AppError::invalid(
                    "Choose an owned income entry from this calendar month for each receipt.",
                ));
            }
            let elsewhere: bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM income_receipts r JOIN expected_income e ON e.id=r.expected_id AND e.user_id=r.user_id WHERE r.user_id=? AND r.transaction_id=? AND e.month<>?)").bind(&user.id).bind(receipt).bind(&month).fetch_one(&mut *tx).await?;
            if elsewhere {
                return Err(AppError::invalid(
                    "This income is linked to another month's plan. Unlink it there first.",
                ));
            }
        }
    }
    let budget_total = domain::checked_total(category_amounts.iter().copied())?;
    let income_total = domain::checked_total(income_amounts.iter().copied())?;
    domain::checked_total([income_total, -savings, -budget_total])?;
    sqlx::query("INSERT INTO budget_months(user_id,month,savings_cents,revision) VALUES(?,?,?,1) ON CONFLICT(user_id,month) DO UPDATE SET savings_cents=excluded.savings_cents,revision=budget_months.revision+1").bind(&user.id).bind(&month).bind(savings).execute(&mut *tx).await?;
    sqlx::query("DELETE FROM category_budgets WHERE user_id=? AND month=?")
        .bind(&user.id)
        .bind(&month)
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM expected_income WHERE user_id=? AND month=?")
        .bind(&user.id)
        .bind(&month)
        .execute(&mut *tx)
        .await?;
    for (category, amount) in input.categories.iter().zip(category_amounts) {
        sqlx::query(
            "INSERT INTO category_budgets(user_id,month,category_id,amount_cents) VALUES(?,?,?,?)",
        )
        .bind(&user.id)
        .bind(&month)
        .bind(&category.category_id)
        .bind(amount)
        .execute(&mut *tx)
        .await?;
    }
    for (item, amount) in input.income.iter().zip(income_amounts) {
        let id = item
            .id
            .clone()
            .unwrap_or_else(|| Uuid::new_v4().to_string());
        sqlx::query("INSERT INTO expected_income(id,user_id,month,name,amount_cents,complete) VALUES(?,?,?,?,?,?)").bind(&id).bind(&user.id).bind(&month).bind(item.name.trim()).bind(amount).bind(item.complete).execute(&mut *tx).await?;
        for receipt in &item.transaction_ids {
            sqlx::query(
                "INSERT INTO income_receipts(user_id,expected_id,transaction_id) VALUES(?,?,?)",
            )
            .bind(&user.id)
            .bind(&id)
            .bind(receipt)
            .execute(&mut *tx)
            .await?;
        }
    }
    let new_links: HashSet<(String,String)>=sqlx::query_as("SELECT r.transaction_id,r.expected_id FROM income_receipts r JOIN expected_income e ON e.user_id=r.user_id AND e.id=r.expected_id WHERE e.user_id=? AND e.month=?").bind(&user.id).bind(&month).fetch_all(&mut *tx).await?.into_iter().collect();
    let changed: HashSet<_> = old_links
        .symmetric_difference(&new_links)
        .map(|(id, _)| id)
        .collect();
    for id in changed {
        sqlx::query("UPDATE transactions SET revision=revision+1 WHERE user_id=? AND id=?")
            .bind(&user.id)
            .bind(id)
            .execute(&mut *tx)
            .await?;
    }
    tx.commit().await?;
    Ok(Json(serde_json::json!({"saved":true})))
}
