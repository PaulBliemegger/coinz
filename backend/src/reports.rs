use crate::{
    AppState, auth, domain,
    error::{AppError, Result},
};
use axum::{
    Json,
    extract::{Query, State},
};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, QueryBuilder, Sqlite};
use tower_sessions::Session;

#[derive(Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct Filter {
    start: Option<String>,
    end: Option<String>,
    q: Option<String>,
    page: Option<u32>,
    page_size: Option<u32>,
}
impl Filter {
    fn validate(&self) -> Result<()> {
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
        Ok(())
    }
}
#[derive(Serialize, FromRow)]
pub struct Entry {
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
    if let Some(q) = &filter.q {
        sql.push(" AND instr(lower(t.description),lower(")
            .push_bind(q.trim())
            .push("))>0");
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
    Ok(sql.build_query_as().fetch_all(conn).await?)
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
}
pub async fn dashboard(
    State(state): State<AppState>,
    session: Session,
    Query(input): Query<DashboardFilter>,
) -> Result<Json<Dashboard>> {
    let user = auth::user(&session, &state.pool).await?;
    domain::month_range(&input.month)?;
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
    tx.commit().await?;
    Ok(Json(Dashboard {
        month: input.month,
        monthly,
        yearly: totals(&annual)?,
        months,
        latest,
        latest_date,
        latest_import,
    }))
}
