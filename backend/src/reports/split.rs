use super::category_spending;
use crate::{
    AppState, auth,
    classification::Item,
    domain,
    error::{AppError, Result},
};
use axum::{
    Json,
    extract::{Query, State},
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};
use tower_sessions::Session;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SplitFilter {
    month: String,
    as_of: String,
    average_months: Option<String>,
}
#[derive(Serialize)]
pub struct MonthOption {
    month: String,
    entry_count: i64,
    eligible: bool,
}
#[derive(Serialize)]
pub struct Slice {
    category: Option<Item>,
    amount_cents: String,
}
#[derive(Serialize)]
pub struct Split {
    month: String,
    months: Vec<MonthOption>,
    selected_months: Vec<String>,
    current: Vec<Slice>,
    average: Vec<Slice>,
    current_total_cents: String,
    average_total_cents: String,
}

pub async fn get(
    State(state): State<AppState>,
    session: Session,
    Query(input): Query<SplitFilter>,
) -> Result<Json<Split>> {
    let user = auth::user(&session, &state.pool).await?;
    domain::month_range(&input.month)?;
    domain::date(&input.as_of)?;
    let year = &input.month[..4];
    let mut tx = state.pool.begin().await?;
    let mut months = Vec::new();
    for n in 1..=12 {
        let month = format!("{year}-{n:02}");
        let (start, end) = domain::month_range(&month)?;
        let entry_count = sqlx::query_scalar(
            "SELECT COUNT(*) FROM transactions WHERE user_id=? AND date>=? AND date<?",
        )
        .bind(&user.id)
        .bind(start)
        .bind(end)
        .fetch_one(&mut *tx)
        .await?;
        months.push(MonthOption {
            eligible: month.as_str() < &input.as_of[..7],
            month,
            entry_count,
        });
    }
    let mut selected = if let Some(raw) = &input.average_months {
        if raw.is_empty() {
            Vec::new()
        } else {
            raw.split(',').map(str::to_owned).collect()
        }
    } else {
        months
            .iter()
            .filter(|m| m.eligible && m.entry_count > 0)
            .map(|m| m.month.clone())
            .collect::<Vec<_>>()
    };
    if selected.len() > 12
        || selected.iter().collect::<HashSet<_>>().len() != selected.len()
        || selected.iter().any(|m| {
            !months
                .iter()
                .any(|option| option.month == *m && option.eligible)
        })
    {
        return Err(AppError::invalid(
            "Choose distinct completed months from this overview year.",
        ));
    }
    selected.sort();
    let (start, end) = domain::month_range(&input.month)?;
    let current = category_spending::load(&mut tx, &user.id, &start, &end)
        .await?
        .into_iter()
        .map(|s| Slice {
            category: s.category,
            amount_cents: s.spending_cents,
        })
        .collect::<Vec<_>>();
    let mut buckets: BTreeMap<Option<String>, (Option<Item>, i128)> = BTreeMap::new();
    for month in &selected {
        let (start, end) = domain::month_range(month)?;
        for item in category_spending::load(&mut tx, &user.id, &start, &end).await? {
            let key = item.category.as_ref().map(|c| c.id.clone());
            let cents = item
                .spending_cents
                .parse::<i64>()
                .map_err(AppError::internal)?;
            let entry = buckets.entry(key).or_insert((item.category, 0));
            entry.1 += i128::from(cents);
        }
    }
    let total: i128 = buckets.values().map(|(_, n)| *n).sum();
    let count = selected.len() as i128;
    let average_total = if count == 0 {
        0
    } else {
        (total + count / 2) / count
    };
    let mut average = Vec::new();
    if count > 0 {
        let mut values = buckets
            .into_values()
            .map(|(category, amount)| (category, amount / count, amount % count))
            .collect::<Vec<_>>();
        let base: i128 = values.iter().map(|(_, n, _)| *n).sum();
        values.sort_by_key(|a| std::cmp::Reverse(a.2));
        // Distribute rounding cents so slices always sum to the rounded average total.
        for (index, (_, amount, _)) in values.iter_mut().enumerate() {
            if (index as i128) < average_total - base {
                *amount += 1;
            }
        }
        values.sort_by_key(|a| std::cmp::Reverse(a.1));
        average = values
            .into_iter()
            .filter(|(_, amount, _)| *amount > 0)
            .map(|(category, amount, _)| Slice {
                category,
                amount_cents: amount.to_string(),
            })
            .collect();
    }
    let current_total = domain::checked_total(
        current
            .iter()
            .map(|s| s.amount_cents.parse::<i64>().map_err(AppError::internal))
            .collect::<Result<Vec<_>>>()?,
    )?;
    tx.commit().await?;
    Ok(Json(Split {
        month: input.month,
        months,
        selected_months: selected,
        current,
        average,
        current_total_cents: current_total.to_string(),
        average_total_cents: average_total.to_string(),
    }))
}
