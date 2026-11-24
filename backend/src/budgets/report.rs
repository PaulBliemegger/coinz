use super::{Category, Income, Plan, Receipt};
use crate::{classification, domain, error::Result};
use sqlx::{FromRow, SqliteConnection};
use std::collections::HashMap;

#[derive(FromRow)]
struct Actual {
    id: String,
    date: String,
    description: String,
    amount_cents: i64,
    category_id: Option<String>,
}
pub async fn load(
    conn: &mut SqliteConnection,
    owner: &str,
    month: &str,
    as_of: &str,
) -> Result<Plan> {
    let (start, end) = domain::month_range(month)?;
    let year_start = format!("{}-01-01", &month[..4]);
    let saved: Option<(i64, i64)> = sqlx::query_as(
        "SELECT savings_cents,revision FROM budget_months WHERE user_id=? AND month=?",
    )
    .bind(owner)
    .bind(month)
    .fetch_optional(&mut *conn)
    .await?;
    let (savings, revision) = saved.unwrap_or((0, 0));
    let actuals: Vec<Actual> = sqlx::query_as("SELECT t.id,t.date,t.description,t.amount_cents,ec.category_id FROM transactions t LEFT JOIN entry_categories ec ON ec.transaction_id=t.id AND ec.user_id=t.user_id WHERE t.user_id=? AND t.date>=? AND t.date<? AND t.date<=? ORDER BY t.date,t.id").bind(owner).bind(&year_start).bind(&end).bind(as_of).fetch_all(&mut *conn).await?;
    let mut expenses: HashMap<Option<String>, Vec<i64>> = HashMap::new();
    let mut year_expenses: HashMap<String, Vec<i64>> = HashMap::new();
    let mut per_month: HashMap<(String, String), Vec<i64>> = HashMap::new();
    let mut receipts = Vec::new();
    for item in &actuals {
        if item.amount_cents < 0 {
            if item.date >= start {
                expenses
                    .entry(item.category_id.clone())
                    .or_default()
                    .push(-item.amount_cents);
            }
            if let Some(category) = &item.category_id {
                year_expenses
                    .entry(category.clone())
                    .or_default()
                    .push(-item.amount_cents);
                per_month
                    .entry((category.clone(), item.date[..7].to_owned()))
                    .or_default()
                    .push(-item.amount_cents);
            }
        } else if item.date >= start {
            receipts.push(Receipt {
                id: item.id.clone(),
                date: item.date.clone(),
                description: item.description.clone(),
                amount_cents: item.amount_cents.to_string(),
            });
        }
    }
    let yearly: Vec<(String,String,i64)> = sqlx::query_as("SELECT category_id,month,amount_cents FROM category_budgets WHERE user_id=? AND month>=? AND month<=? ORDER BY month").bind(owner).bind(&month[..4]).bind(month).fetch_all(&mut *conn).await?;
    let items: Vec<classification::Item> = sqlx::query_as("SELECT id,name,color,archived FROM classifications WHERE user_id=? AND kind='category' ORDER BY name_key,id").bind(owner).fetch_all(&mut *conn).await?;
    let mut categories = Vec::new();
    let mut budgeted_ids = Vec::new();
    let mut budgets = Vec::new();
    for category in items {
        let matching: Vec<_> = yearly
            .iter()
            .filter(|(id, _, _)| id == &category.id)
            .collect();
        let budget = matching
            .iter()
            .find(|(_, m, _)| m == month)
            .map(|(_, _, amount)| *amount);
        let spending = domain::checked_total(
            expenses
                .get(&Some(category.id.clone()))
                .into_iter()
                .flatten()
                .copied(),
        )?;
        if let Some(amount) = budget {
            budgeted_ids.push(category.id.clone());
            budgets.push(amount);
        }
        let budgeted_spending = domain::checked_total(
            matching
                .iter()
                .flat_map(|(_, m, _)| {
                    per_month
                        .get(&(category.id.clone(), m.clone()))
                        .into_iter()
                        .flatten()
                })
                .copied(),
        )?;
        categories.push(Category {
            year_spending_cents: domain::checked_total(
                year_expenses
                    .get(&category.id)
                    .into_iter()
                    .flatten()
                    .copied(),
            )?
            .to_string(),
            year_budget_cents: domain::checked_total(matching.iter().map(|(_, _, a)| *a))?
                .to_string(),
            year_budgeted_spending_cents: budgeted_spending.to_string(),
            budgeted_months: matching.len(),
            category,
            budget_cents: budget.map(|a| a.to_string()),
            spending_cents: spending.to_string(),
            remaining_cents: budget.map(|a| (i128::from(a) - i128::from(spending)).to_string()),
        });
    }
    let rows: Vec<(String,String,i64,bool)> = sqlx::query_as("SELECT id,name,amount_cents,complete FROM expected_income WHERE user_id=? AND month=? ORDER BY rowid").bind(owner).bind(month).fetch_all(&mut *conn).await?;
    let mut income = Vec::new();
    let mut planned = Vec::new();
    let mut pending = Vec::new();
    for (id, name, amount, complete) in rows {
        let links: Vec<(String,String,i64)> = sqlx::query_as("SELECT t.id,t.date,t.amount_cents FROM income_receipts r JOIN transactions t ON t.id=r.transaction_id AND t.user_id=r.user_id WHERE r.user_id=? AND r.expected_id=? ORDER BY t.date,t.id").bind(owner).bind(&id).fetch_all(&mut *conn).await?;
        let valid =
            |date: &str, cents: i64| cents > 0 && date >= start.as_str() && date < end.as_str();
        let received = domain::checked_total(
            links
                .iter()
                .filter(|(_, date, c)| valid(date, *c) && date.as_str() <= as_of)
                .map(|(_, _, c)| *c),
        )?;
        let outstanding = if complete {
            0
        } else {
            (amount - received).max(0)
        };
        planned.push(amount);
        pending.push(outstanding);
        income.push(Income {
            id,
            name,
            planned_cents: amount.to_string(),
            complete,
            invalid_receipt_count: links.iter().filter(|(_, date, c)| !valid(date, *c)).count(),
            transaction_ids: links.into_iter().map(|(id, _, _)| id).collect(),
            received_cents: received.to_string(),
            pending_cents: outstanding.to_string(),
        });
    }
    let recorded = domain::checked_total(
        actuals
            .iter()
            .filter(|a| a.date >= start && a.amount_cents > 0)
            .map(|a| a.amount_cents),
    )?;
    let spending = domain::checked_total(expenses.values().flatten().copied())?;
    let unbudgeted = domain::checked_total(
        expenses
            .iter()
            .filter(|(id, _)| id.as_ref().is_none_or(|id| !budgeted_ids.contains(id)))
            .flat_map(|(_, a)| a)
            .copied(),
    )?;
    let budget_total = domain::checked_total(budgets)?;
    let planned_total = domain::checked_total(planned)?;
    let pending_total = domain::checked_total(pending)?;
    // Derived differences can be negative; i128 avoids overflow while keeping exact cents.
    let diff = |terms: &[i64]| {
        terms
            .iter()
            .map(|n| i128::from(*n))
            .sum::<i128>()
            .to_string()
    };
    Ok(Plan {
        month: month.into(),
        as_of: as_of.into(),
        revision,
        savings_cents: savings.to_string(),
        categories,
        income,
        receipts,
        recorded_income_cents: recorded.to_string(),
        spending_cents: spending.to_string(),
        unbudgeted_spending_cents: unbudgeted.to_string(),
        budget_total_cents: budget_total.to_string(),
        planned_income_cents: planned_total.to_string(),
        pending_income_cents: pending_total.to_string(),
        unallocated_cents: diff(&[planned_total, -savings, -budget_total]),
        from_recorded_cents: diff(&[recorded, -spending, -savings]),
        including_pending_cents: diff(&[recorded, pending_total, -spending, -savings]),
    })
}
