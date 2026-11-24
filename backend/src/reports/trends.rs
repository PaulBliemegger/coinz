use axum::{
    Json,
    extract::{Query, State},
};
use chrono::Datelike;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use tower_sessions::Session;

use super::{Totals, totals};
use crate::{
    AppState, auth, classification, domain,
    error::{AppError, Result},
};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TrendFilter {
    month: Option<String>,
    year: Option<String>,
    months: Option<u8>,
    category_id: Option<String>,
}

#[derive(Serialize)]
pub struct TrendMonth {
    month: String,
    #[serde(flatten)]
    totals: Totals,
    category_spending_cents: String,
    category_entry_count: usize,
}

#[derive(Serialize)]
pub struct Trends {
    category: Option<classification::Item>,
    months: Vec<TrendMonth>,
}

#[derive(FromRow)]
struct Amount {
    amount_cents: i64,
    category_id: Option<String>,
}

fn month_window(month: &str, count: u8) -> Result<Vec<String>> {
    let date = domain::date(&format!("{month}-01"))?;
    if ![6, 12].contains(&count) {
        return Err(AppError::invalid("Choose a six- or twelve-month trend."));
    }
    let end = (date.year() - 1) * 12 + date.month0() as i32;
    Ok(((end + 1 - i32::from(count)).max(0)..=end)
        .map(|index| format!("{:04}-{:02}", index / 12 + 1, index % 12 + 1))
        .collect())
}

pub async fn get(
    State(state): State<AppState>,
    session: Session,
    Query(input): Query<TrendFilter>,
) -> Result<Json<Trends>> {
    let user = auth::user(&session, &state.pool).await?;
    let window = if let Some(year) = &input.year {
        if input.month.is_some() || input.months.is_some() || year.len() != 4 {
            return Err(AppError::invalid(
                "Choose a year or a rolling month window.",
            ));
        }
        domain::month_range(&format!("{year}-01"))?;
        (1..=12).map(|m| format!("{year}-{m:02}")).collect()
    } else {
        month_window(
            input
                .month
                .as_deref()
                .ok_or_else(|| AppError::invalid("Choose a year or month."))?,
            input.months.unwrap_or(6),
        )?
    };
    let mut tx = state.pool.begin().await?;
    let category = if let Some(id) = &input.category_id {
        if uuid::Uuid::parse_str(id).is_err() {
            return Err(AppError::invalid("Choose a valid category."));
        }
        Some(
            classification::owned_item(&mut tx, &user.id, id, classification::Kind::Category)
                .await?,
        )
    } else {
        None
    };
    let mut months = Vec::new();
    for month in window {
        let (start, end) = domain::month_range(&month)?;
        // Only the single category joins: labels cannot multiply either series.
        let rows: Vec<Amount> = sqlx::query_as("SELECT t.amount_cents,ec.category_id FROM transactions t LEFT JOIN entry_categories ec ON ec.user_id=t.user_id AND ec.transaction_id=t.id WHERE t.user_id=? AND t.date>=? AND t.date<?")
            .bind(&user.id).bind(start).bind(end).fetch_all(&mut *tx).await?;
        let all: Vec<i64> = rows.iter().map(|r| r.amount_cents).collect();
        let selected: Vec<i64> = rows
            .iter()
            .filter(|r| r.amount_cents < 0 && r.category_id == input.category_id)
            .map(|r| r.amount_cents)
            .collect();
        months.push(TrendMonth {
            month,
            totals: totals(&all)?,
            category_spending_cents: totals(&selected)?.spending_cents,
            category_entry_count: selected.len(),
        });
    }
    tx.commit().await?;
    Ok(Json(Trends { category, months }))
}
