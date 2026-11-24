use std::collections::BTreeMap;

use serde::Serialize;
use sqlx::{FromRow, SqliteConnection};

use crate::{
    classification::Item,
    error::{AppError, Result},
};

#[derive(Serialize)]
pub struct CategorySpending {
    category: Option<Item>,
    spending_cents: String,
    entry_count: usize,
    share_percent: String,
}

#[derive(FromRow)]
struct CategorizedAmount {
    category_id: Option<String>,
    name: String,
    color: String,
    archived: bool,
    amount_cents: i64,
}

struct Bucket {
    category: Option<Item>,
    spending: i128,
    count: usize,
}

pub async fn load(
    conn: &mut SqliteConnection,
    owner: &str,
    start: &str,
    end: &str,
) -> Result<Vec<CategorySpending>> {
    // One category per entry; labels never join this query and cannot multiply spending.
    let rows: Vec<CategorizedAmount> = sqlx::query_as(
        "SELECT ec.category_id,COALESCE(c.name,'Uncategorized') AS name,COALESCE(c.color,'sand') AS color,COALESCE(c.archived,0) AS archived,t.amount_cents
         FROM transactions t
         LEFT JOIN entry_categories ec ON ec.transaction_id=t.id AND ec.user_id=t.user_id
         LEFT JOIN classifications c ON c.id=ec.category_id AND c.user_id=t.user_id AND c.kind='category'
         WHERE t.user_id=? AND t.date>=? AND t.date<? AND t.amount_cents<0",
    ).bind(owner).bind(start).bind(end).fetch_all(conn).await?;
    let mut buckets = BTreeMap::new();
    for row in rows {
        let bucket = buckets
            .entry(row.category_id.clone())
            .or_insert_with(|| Bucket {
                category: row.category_id.map(|id| Item {
                    id,
                    name: row.name,
                    color: row.color,
                    archived: row.archived,
                }),
                spending: 0,
                count: 0,
            });
        bucket.spending -= i128::from(row.amount_cents);
        bucket.count += 1;
    }
    let total: i128 = buckets.values().map(|bucket| bucket.spending).sum();
    i64::try_from(total)
        .map_err(|_| AppError::invalid("These entries exceed the supported spending range."))?;
    let mut buckets: Vec<Bucket> = buckets.into_values().collect();
    buckets.sort_by(|a, b| {
        b.spending.cmp(&a.spending).then_with(|| {
            let name = |bucket: &Bucket| {
                bucket
                    .category
                    .as_ref()
                    .map_or("uncategorized".into(), |c| c.name.to_lowercase())
            };
            name(a).cmp(&name(b))
        })
    });
    Ok(buckets
        .into_iter()
        .map(|bucket| {
            // Rounded to one decimal place using integers; displayed shares can sum to 99.9/100.1.
            let tenths = (bucket.spending * 1000 + total / 2) / total;
            CategorySpending {
                category: bucket.category,
                spending_cents: bucket.spending.to_string(),
                entry_count: bucket.count,
                share_percent: format!("{}.{:01}", tenths / 10, tenths % 10),
            }
        })
        .collect())
}
