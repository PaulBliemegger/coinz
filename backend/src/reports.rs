use crate::{
    AppState, auth, classification, domain,
    error::{AppError, Result},
};
use axum::{
    Json,
    extract::{Query, State},
};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, QueryBuilder, Sqlite};
use tower_sessions::Session;

mod category_spending;
pub mod split;
pub mod trends;

#[derive(Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct Filter {
    start: Option<String>,
    end: Option<String>,
    q: Option<String>,
    page: Option<u32>,
    page_size: Option<u32>,
    category_ids: Option<String>,
    label_ids: Option<String>,
    #[serde(default)]
    label_match: LabelMatch,
    #[serde(default)]
    uncategorized: bool,
    #[serde(default)]
    unlabeled: bool,
    #[serde(default)]
    entry_type: EntryType,
    months: Option<String>,
}
#[derive(Deserialize, Default)]
#[serde(rename_all = "snake_case")]
enum EntryType {
    #[default]
    All,
    Spending,
    Income,
}
#[derive(Deserialize, Default, PartialEq)]
#[serde(rename_all = "snake_case")]
enum LabelMatch {
    #[default]
    Any,
    All,
}
fn ids(value: &Option<String>) -> Vec<&str> {
    value
        .as_deref()
        .unwrap_or("")
        .split(',')
        .filter(|s| !s.is_empty())
        .collect()
}
impl Filter {
    fn validate(&self) -> Result<()> {
        if let Some(raw) = &self.months {
            let values = raw.split(',').collect::<Vec<_>>();
            if values.is_empty()
                || values.len() > 12
                || values
                    .iter()
                    .collect::<std::collections::HashSet<_>>()
                    .len()
                    != values.len()
            {
                return Err(AppError::invalid("Choose up to twelve distinct months."));
            }
            for month in values {
                domain::month_range(month)?;
            }
        }
        if let Some(s) = &self.start {
            domain::date(s)?;
        }
        if let Some(s) = &self.end {
            domain::date(s)?;
        }
        if let (Some(start), Some(end)) = (&self.start, &self.end)
            && start > end
        {
            return Err(AppError::invalid(
                "The start date must not follow the end date.",
            ));
        }
        if self.q.as_ref().is_some_and(|q| q.chars().count() > 200)
            || self.page == Some(0)
            || self.page_size.is_some_and(|n| !(1..=100).contains(&n))
        {
            return Err(AppError::invalid(
                "Use a search of at most 200 characters and a valid page size (1–100).",
            ));
        }
        for value in [&self.category_ids, &self.label_ids] {
            let values = ids(value);
            if values.len() > 100
                || values.iter().any(|id| uuid::Uuid::parse_str(id).is_err())
                || values
                    .iter()
                    .collect::<std::collections::HashSet<_>>()
                    .len()
                    != values.len()
            {
                return Err(AppError::invalid(
                    "Choose up to 100 distinct category or label IDs.",
                ));
            }
        }
        if self.unlabeled && !ids(&self.label_ids).is_empty() {
            return Err(AppError::invalid(
                "Choose labels or unlabeled entries, not both.",
            ));
        }
        Ok(())
    }
}
#[derive(Serialize, FromRow)]
pub struct Entry {
    #[sqlx(skip)]
    category: Option<classification::Item>,
    #[sqlx(skip)]
    labels: Vec<classification::Item>,
    id: String,
    source_id: Option<String>,
    source_name: Option<String>,
    date: String,
    description: String,
    amount_cents: String,
    origin: String,
    csv_linked: bool,
}
#[derive(Serialize, Default)]
pub struct Totals {
    income_cents: String,
    spending_cents: String,
    net_cents: String,
    transaction_count: usize,
}
fn totals(values: &[i64]) -> Result<Totals> {
    let income = domain::checked_total(values.iter().copied().filter(|n| *n > 0))?;
    let spending = domain::checked_total(values.iter().copied().filter(|n| *n < 0).map(|n| -n))?;
    Ok(Totals {
        income_cents: income.to_string(),
        spending_cents: spending.to_string(),
        net_cents: domain::checked_total([income, -spending])?.to_string(),
        transaction_count: values.len(),
    })
}
#[derive(Serialize)]
pub struct Entries {
    entries: Vec<Entry>,
    totals: Totals,
    page: u32,
    page_size: u32,
    pages: u32,
}
pub async fn entries(
    State(state): State<AppState>,
    session: Session,
    Query(filter): Query<Filter>,
) -> Result<Json<Entries>> {
    let user = auth::user(&session, &state.pool).await?;
    filter.validate()?;
    // A read transaction keeps page contents and all-result totals in one snapshot.
    let mut tx = state.pool.begin().await?;
    for (kind, values) in [
        (classification::Kind::Category, ids(&filter.category_ids)),
        (classification::Kind::Label, ids(&filter.label_ids)),
    ] {
        for id in values {
            classification::owned_item(&mut tx, &user.id, id, kind).await?;
        }
    }
    let values = amounts(&filter, &user.id, &mut tx).await?;
    let totals = totals(&values)?;
    let size = filter.page_size.unwrap_or(25);
    let pages = (values.len() as u32).div_ceil(size).max(1);
    let page = filter.page.unwrap_or(1).min(pages);
    let entries = page_rows(&filter, &user.id, page, size, &mut tx).await?;
    tx.commit().await?;
    Ok(Json(Entries {
        entries,
        totals,
        page,
        page_size: size,
        pages,
    }))
}
async fn amounts(
    filter: &Filter,
    owner: &str,
    conn: &mut sqlx::SqliteConnection,
) -> Result<Vec<i64>> {
    let mut sql = query(filter, owner, "SELECT t.amount_cents");
    Ok(sql.build_query_scalar().fetch_all(conn).await?)
}
fn query<'a>(filter: &'a Filter, owner: &'a str, select: &str) -> QueryBuilder<'a, Sqlite> {
    let mut sql = QueryBuilder::new(select.to_owned());
    sql.push(" FROM transactions t LEFT JOIN transaction_sources s ON s.id=t.source_id AND s.user_id=t.user_id WHERE t.user_id=").push_bind(owner);
    if let Some(start) = &filter.start {
        sql.push(" AND t.date>=").push_bind(start);
    }
    if let Some(end) = &filter.end {
        sql.push(" AND t.date<=").push_bind(end);
    }
    match filter.entry_type {
        EntryType::All => {}
        EntryType::Spending => {
            sql.push(" AND t.amount_cents<0");
        }
        EntryType::Income => {
            sql.push(" AND t.amount_cents>0");
        }
    }
    if let Some(months) = &filter.months {
        sql.push(" AND substr(t.date,1,7) IN (");
        let mut list = sql.separated(",");
        for month in months.split(',') {
            list.push_bind(month);
        }
        sql.push(")");
    }
    if let Some(q) = &filter.q {
        sql.push(" AND instr(lower(t.description),lower(")
            .push_bind(q.trim())
            .push("))>0");
    }
    let categories = ids(&filter.category_ids);
    if !categories.is_empty() || filter.uncategorized {
        sql.push(" AND (");
        if !categories.is_empty() {
            sql.push("EXISTS(SELECT 1 FROM entry_categories ec WHERE ec.user_id=t.user_id AND ec.transaction_id=t.id AND ec.category_id IN (");
            let mut list = sql.separated(",");
            for id in categories {
                list.push_bind(id);
            }
            sql.push("))");
            if filter.uncategorized {
                sql.push(" OR ");
            }
        }
        if filter.uncategorized {
            sql.push("NOT EXISTS(SELECT 1 FROM entry_categories ec WHERE ec.user_id=t.user_id AND ec.transaction_id=t.id)");
        }
        sql.push(")");
    }
    let labels = ids(&filter.label_ids);
    if filter.label_match == LabelMatch::All {
        for id in labels {
            sql.push(" AND EXISTS(SELECT 1 FROM entry_labels el WHERE el.user_id=t.user_id AND el.transaction_id=t.id AND el.label_id=").push_bind(id).push(")");
        }
    } else if !labels.is_empty() {
        sql.push(" AND EXISTS(SELECT 1 FROM entry_labels el WHERE el.user_id=t.user_id AND el.transaction_id=t.id AND el.label_id IN (");
        let mut list = sql.separated(",");
        for id in labels {
            list.push_bind(id);
        }
        sql.push("))");
    }
    if filter.unlabeled {
        sql.push(" AND NOT EXISTS(SELECT 1 FROM entry_labels el WHERE el.user_id=t.user_id AND el.transaction_id=t.id)");
    }
    sql
}
async fn page_rows(
    filter: &Filter,
    owner: &str,
    page: u32,
    size: u32,
    conn: &mut sqlx::SqliteConnection,
) -> Result<Vec<Entry>> {
    let mut sql = query(
        filter,
        owner,
        "SELECT t.id,t.source_id,s.name AS source_name,t.date,t.description,CAST(t.amount_cents AS TEXT) AS amount_cents,t.origin,EXISTS(SELECT 1 FROM import_records r WHERE r.user_id=t.user_id AND r.transaction_id=t.id AND r.disposition='accepted') AS csv_linked",
    );
    sql.push(" ORDER BY t.date DESC,t.created_at DESC,t.id DESC LIMIT ")
        .push_bind(size as i64)
        .push(" OFFSET ")
        .push_bind(i64::from(page - 1) * i64::from(size));
    let mut entries: Vec<Entry> = sql.build_query_as().fetch_all(&mut *conn).await?;
    for entry in &mut entries {
        entry.category = classification::entry_category(conn, owner, &entry.id).await?;
        entry.labels = classification::entry_labels(conn, owner, &entry.id).await?;
    }
    Ok(entries)
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DashboardFilter {
    month: String,
}
#[derive(Serialize)]
pub struct Month {
    month: String,
    #[serde(flatten)]
    totals: Totals,
}
#[derive(Serialize)]
pub struct Dashboard {
    month: String,
    monthly: Totals,
    yearly: Totals,
    months: Vec<Month>,
    latest: Vec<Entry>,
    latest_date: Option<String>,
    latest_import: Option<String>,
    spending_categories: Vec<category_spending::CategorySpending>,
}
pub async fn dashboard(
    State(state): State<AppState>,
    session: Session,
    Query(input): Query<DashboardFilter>,
) -> Result<Json<Dashboard>> {
    let user = auth::user(&session, &state.pool).await?;
    let (start, end) = domain::month_range(&input.month)?;
    let year = &input.month[..4];
    let mut tx = state.pool.begin().await?;
    let mut months = Vec::new();
    let mut annual = Vec::new();
    let mut monthly = totals(&[])?;
    for n in 1..=12 {
        let month = format!("{year}-{n:02}");
        let (start, end) = domain::month_range(&month)?;
        let values: Vec<i64> = sqlx::query_scalar(
            "SELECT amount_cents FROM transactions WHERE user_id=? AND date>=? AND date<?",
        )
        .bind(&user.id)
        .bind(start)
        .bind(end)
        .fetch_all(&mut *tx)
        .await?;
        if month == input.month {
            monthly = totals(&values)?;
        }
        let summary = totals(&values)?;
        annual.extend(values);
        months.push(Month {
            month,
            totals: summary,
        });
    }
    let latest = page_rows(&Filter::default(), &user.id, 1, 5, &mut tx).await?;
    let latest_date = sqlx::query_scalar("SELECT MAX(date) FROM transactions WHERE user_id=?")
        .bind(&user.id)
        .fetch_one(&mut *tx)
        .await?;
    let latest_import =
        sqlx::query_scalar("SELECT MAX(created_at) FROM import_batches WHERE user_id=?")
            .bind(&user.id)
            .fetch_one(&mut *tx)
            .await?;
    let spending_categories = category_spending::load(&mut tx, &user.id, &start, &end).await?;
    tx.commit().await?;
    Ok(Json(Dashboard {
        month: input.month,
        monthly,
        yearly: totals(&annual)?,
        months,
        latest,
        latest_date,
        latest_import,
        spending_categories,
    }))
}
