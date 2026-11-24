//! Coinz entry snapshots share import review/commit, but never invent bank IDs.
use super::parse::Row;
use crate::{
    AppState, auth, domain,
    error::{AppError, Result},
};
use axum::{extract::State, http::header, response::IntoResponse};
use serde::Serialize;
use sqlx::{FromRow, SqliteConnection};
use tower_sessions::Session;
use uuid::Uuid;

pub const HEADERS: [&str; 9] = [
    "date",
    "amount",
    "description",
    "category",
    "labels",
    "source",
    "entry_ref",
    "source_ref",
    "origin",
];
pub const MAX_BYTES: usize = 16 * 1024 * 1024;
pub const MAX_ROWS: usize = 50_000;

#[derive(Clone, Serialize)]
pub struct Metadata {
    pub category: String,
    pub labels: Vec<String>,
    pub source: String,
    pub entry_ref: String,
    pub source_ref: String,
    pub origin: String,
}

pub fn recognizes(input: &str) -> bool {
    csv::Reader::from_reader(input.as_bytes())
        .headers()
        .is_ok_and(|headers| headers.iter().eq(HEADERS))
}

pub fn parse(input: &str) -> Result<Vec<Row>> {
    if input.len() > MAX_BYTES {
        return Err(AppError::invalid("Coinz exports must be at most 16 MiB."));
    }
    let mut reader = csv::Reader::from_reader(input.as_bytes());
    if !reader
        .headers()
        .map_err(AppError::internal)?
        .iter()
        .eq(HEADERS)
    {
        return Err(AppError::invalid(
            "Use the complete, unchanged coinz export header.",
        ));
    }
    let mut rows = Vec::new();
    let mut sources = std::collections::HashMap::new();
    for record in reader.records() {
        if rows.len() >= MAX_ROWS {
            return Err(AppError::invalid("Restore at most 50,000 entries at once."));
        }
        let record = record.map_err(|_| {
            AppError::invalid("Coinz export rows must have nine correctly quoted fields.")
        })?;
        let mut row = Row {
            record: rows.len() + 2,
            date: record[0].to_owned(),
            description: record[2].to_owned(),
            amount_cents: String::new(),
            external_id: String::new(),
            cents: 0,
            error: None,
            portable: None,
        };
        let parsed = (|| -> Result<Metadata> {
            domain::date(&row.date)?;
            domain::text(&row.description, "Description", 200)?;
            row.cents = domain::parse_money(&record[1])?;
            if row.cents == 0 || record[1].contains(',') {
                return Err(AppError::invalid(
                    "Use a nonzero, signed EUR amount with a dot decimal separator.",
                ));
            }
            row.amount_cents = row.cents.to_string();
            Uuid::parse_str(&record[6])
                .map_err(|_| AppError::invalid("Entry references must be UUIDs."))?;
            if !["manual", "csv", "unknown"].contains(&&record[8]) {
                return Err(AppError::invalid(
                    "Use a valid entry origin: manual, csv or unknown.",
                ));
            }
            if !record[3].is_empty() {
                domain::text(&record[3], "Category", 60)?;
                if record[3].trim().to_lowercase() == "uncategorized" {
                    return Err(AppError::invalid("Leave category empty for Uncategorized."));
                }
            }
            let mut labels: Vec<String> = serde_json::from_str(&record[4])
                .map_err(|_| AppError::invalid("Labels must be a JSON array of names."))?;
            if labels.len() > 100 {
                return Err(AppError::invalid("Restore at most 100 labels per entry."));
            }
            let mut names = std::collections::HashSet::new();
            for label in &mut labels {
                *label = domain::text(label, "Label", 60)?;
                if !names.insert(label.to_lowercase()) {
                    return Err(AppError::invalid("Do not repeat label names on an entry."));
                }
            }
            labels.sort();
            if record[5].is_empty() != record[7].is_empty() {
                return Err(AppError::invalid(
                    "Source name and source reference must both be present or both be empty.",
                ));
            }
            if !record[7].is_empty() {
                domain::text(&record[5], "Source", 80)?;
                Uuid::parse_str(&record[7])
                    .map_err(|_| AppError::invalid("Source references must be UUIDs."))?;
                if sources
                    .insert(record[7].to_ascii_lowercase(), record[5].trim().to_owned())
                    .is_some_and(|name| name != record[5].trim())
                {
                    return Err(AppError::invalid(
                        "One source reference has conflicting names in this export.",
                    ));
                }
            }
            Ok(Metadata {
                category: record[3].trim().to_owned(),
                labels,
                source: record[5].trim().to_owned(),
                entry_ref: record[6].to_ascii_lowercase(),
                source_ref: record[7].to_ascii_lowercase(),
                origin: record[8].to_owned(),
            })
        })();
        match parsed {
            Ok(metadata) => row.portable = Some(metadata),
            Err(error) => row.error = Some(error.1),
        }
        rows.push(row);
    }
    if rows.is_empty() {
        return Err(AppError::invalid("The CSV has no transaction records."));
    }
    Ok(rows)
}

#[derive(FromRow)]
struct Entry {
    id: String,
    date: String,
    description: String,
    amount_cents: i64,
    category: String,
    labels: String,
    source: String,
    entry_ref: String,
    source_ref: String,
    origin: String,
}
async fn entries(conn: &mut SqliteConnection, owner: &str, id: Option<&str>) -> Result<Vec<Entry>> {
    Ok(sqlx::query_as::<_, Entry>(
        "SELECT t.id,t.date,t.description,t.amount_cents,t.origin,COALESCE(c.name,'') AS category,
        COALESCE((SELECT json_group_array(name) FROM (SELECT l.name FROM entry_labels el JOIN classifications l ON l.id=el.label_id AND l.user_id=el.user_id WHERE el.user_id=t.user_id AND el.transaction_id=t.id ORDER BY l.name)), '[]') AS labels,
        COALESCE(s.name,'') AS source,
        COALESCE((SELECT MIN(reference) FROM portable_entry_refs r WHERE r.user_id=t.user_id AND r.transaction_id=t.id),t.id) AS entry_ref,
        COALESCE((SELECT MIN(reference) FROM portable_source_refs r WHERE r.user_id=t.user_id AND r.source_id=t.source_id),t.source_id,'') AS source_ref
        FROM transactions t LEFT JOIN entry_categories ec ON ec.transaction_id=t.id AND ec.user_id=t.user_id
        LEFT JOIN classifications c ON c.id=ec.category_id AND c.user_id=ec.user_id
        LEFT JOIN transaction_sources s ON s.id=t.source_id AND s.user_id=t.user_id
        WHERE t.user_id=? AND (? IS NULL OR t.id=?) ORDER BY t.date,t.id")
        .bind(owner).bind(id).bind(id).fetch_all(conn).await?)
}

fn decimal(cents: i64) -> String {
    let absolute = cents.unsigned_abs();
    format!(
        "{}{}.{:02}",
        if cents < 0 { "-" } else { "" },
        absolute / 100,
        absolute % 100
    )
}

pub async fn export(State(state): State<AppState>, session: Session) -> Result<impl IntoResponse> {
    let user = auth::user(&session, &state.pool).await?;
    let mut tx = state.pool.begin().await?;
    let rows = entries(&mut tx, &user.id, None).await?;
    if rows.len() > MAX_ROWS {
        return Err(AppError::invalid(
            "This ledger exceeds the 50,000-entry portable export limit. No partial export was produced.",
        ));
    }
    let mut csv = csv::Writer::from_writer(Vec::new());
    csv.write_record(HEADERS).map_err(AppError::internal)?;
    for entry in rows {
        csv.write_record([
            entry.date,
            decimal(entry.amount_cents),
            entry.description,
            entry.category,
            entry.labels,
            entry.source,
            entry.entry_ref,
            entry.source_ref,
            entry.origin,
        ])
        .map_err(AppError::internal)?;
    }
    tx.commit().await?;
    let bytes = csv.into_inner().map_err(AppError::internal)?;
    if bytes.len() > MAX_BYTES {
        return Err(AppError::invalid(
            "This ledger exceeds the 16 MiB portable export limit. No partial export was produced.",
        ));
    }
    Ok((
        [
            (header::CONTENT_TYPE, "text/csv; charset=utf-8"),
            (
                header::CONTENT_DISPOSITION,
                "attachment; filename=\"coinz-entries.csv\"",
            ),
        ],
        bytes,
    ))
}

/// Resolve only inside this owner's namespace, then compare the live entry to protect edits.
pub async fn existing(
    conn: &mut SqliteConnection,
    owner: &str,
    row: &Row,
) -> Result<Option<(String, bool)>> {
    let Some(metadata) = &row.portable else {
        return Ok(None);
    };
    let ids: Vec<String> = sqlx::query_scalar("SELECT id FROM transactions WHERE user_id=? AND (id=? OR id IN (SELECT transaction_id FROM portable_entry_refs WHERE user_id=? AND reference=?))")
        .bind(owner).bind(&metadata.entry_ref).bind(owner).bind(&metadata.entry_ref).fetch_all(&mut *conn).await?;
    if ids.len() > 1 {
        return Err(AppError::invalid(
            "This entry reference is ambiguous. Review the export.",
        ));
    }
    let Some(id) = ids.first() else {
        return Ok(None);
    };
    let entry = entries(conn, owner, Some(id))
        .await?
        .pop()
        .ok_or_else(AppError::not_found)?;
    let labels: Vec<String> = serde_json::from_str(&entry.labels).map_err(AppError::internal)?;
    let same = entry.date == row.date
        && entry.description == row.description
        && entry.amount_cents == row.cents
        && entry.category == metadata.category
        && labels == metadata.labels
        && entry.source == metadata.source
        && entry.source_ref == metadata.source_ref
        && entry.origin == metadata.origin;
    Ok(Some((entry.id, same)))
}

pub async fn source(
    conn: &mut SqliteConnection,
    owner: &str,
    metadata: &Metadata,
) -> Result<Option<String>> {
    if metadata.source_ref.is_empty() {
        return Ok(None);
    }
    let found: Vec<(String, String)> = sqlx::query_as("SELECT id,name FROM transaction_sources WHERE user_id=? AND (id=? OR id IN (SELECT source_id FROM portable_source_refs WHERE user_id=? AND reference=?))")
        .bind(owner).bind(&metadata.source_ref).bind(owner).bind(&metadata.source_ref).fetch_all(&mut *conn).await?;
    if found.len() > 1
        || found
            .first()
            .is_some_and(|(_, name)| name != &metadata.source)
    {
        return Err(AppError::invalid(
            "This source reference conflicts with an existing source. Review the export before restoring.",
        ));
    }
    let id = match found.into_iter().next() {
        Some((id, _)) => id,
        None => {
            let id = Uuid::new_v4().to_string();
            sqlx::query("INSERT INTO transaction_sources(id,user_id,name) VALUES(?,?,?)")
                .bind(&id)
                .bind(owner)
                .bind(&metadata.source)
                .execute(&mut *conn)
                .await?;
            id
        }
    };
    sqlx::query("INSERT INTO portable_source_refs(user_id,reference,source_id) VALUES(?,?,?) ON CONFLICT(user_id,reference) DO NOTHING")
        .bind(owner).bind(&metadata.source_ref).bind(&id).execute(conn).await?;
    Ok(Some(id))
}

async fn vocabulary(
    conn: &mut SqliteConnection,
    owner: &str,
    kind: &str,
    name: &str,
) -> Result<String> {
    let id: Option<String> = sqlx::query_scalar(
        "SELECT id FROM classifications WHERE user_id=? AND kind=? AND name_key=?",
    )
    .bind(owner)
    .bind(kind)
    .bind(name.trim().to_lowercase())
    .fetch_optional(&mut *conn)
    .await?;
    if let Some(id) = id {
        return Ok(id);
    }
    let id = Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO classifications(id,user_id,kind,name,name_key,color) VALUES(?,?,?,?,?,'sage')",
    )
    .bind(&id)
    .bind(owner)
    .bind(kind)
    .bind(name.trim())
    .bind(name.trim().to_lowercase())
    .execute(conn)
    .await?;
    Ok(id)
}

pub async fn apply(
    conn: &mut SqliteConnection,
    owner: &str,
    id: &str,
    metadata: &Metadata,
) -> Result<()> {
    if !metadata.category.is_empty() {
        let category = vocabulary(conn, owner, "category", &metadata.category).await?;
        // Restore archived assignments too; normal editing still requires restoring a target first.
        sqlx::query(
            "INSERT INTO entry_categories(transaction_id,user_id,category_id) VALUES(?,?,?)",
        )
        .bind(id)
        .bind(owner)
        .bind(category)
        .execute(&mut *conn)
        .await?;
    }
    for name in &metadata.labels {
        let label = vocabulary(conn, owner, "label", name).await?;
        sqlx::query("INSERT INTO entry_labels(transaction_id,user_id,label_id) VALUES(?,?,?)")
            .bind(id)
            .bind(owner)
            .bind(label)
            .execute(&mut *conn)
            .await?;
    }
    link(conn, owner, id, &metadata.entry_ref).await
}

pub async fn link(
    conn: &mut SqliteConnection,
    owner: &str,
    id: &str,
    reference: &str,
) -> Result<()> {
    sqlx::query("INSERT INTO portable_entry_refs(user_id,reference,transaction_id) VALUES(?,?,?) ON CONFLICT(user_id,reference) DO NOTHING")
        .bind(owner).bind(reference).bind(id).execute(conn).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(
        amount: &str,
        category: &str,
        labels: &str,
        source: &str,
        source_ref: &str,
    ) -> String {
        let mut writer = csv::Writer::from_writer(Vec::new());
        writer.write_record(HEADERS).unwrap();
        writer
            .write_record([
                "2026-11-03",
                amount,
                "Merchant, \"Example\"\nA second line",
                category,
                labels,
                source,
                "ad8ce4d8-a281-4cf2-860f-ea4d3d4f2419",
                source_ref,
                "manual",
            ])
            .unwrap();
        String::from_utf8(writer.into_inner().unwrap()).unwrap()
    }

    #[test]
    fn native_csv_preserves_quoted_text_and_validates_money_and_metadata() {
        let csv = fixture(
            "-12.34",
            "Dining",
            r#"["Trip, friends","Books; gifts"]"#,
            "",
            "",
        );
        assert!(recognizes(&csv));
        let rows = parse(&csv).unwrap();
        assert_eq!(rows[0].cents, -1234);
        assert_eq!(rows[0].description, "Merchant, \"Example\"\nA second line");
        assert_eq!(
            rows[0].portable.as_ref().unwrap().labels,
            ["Books; gifts", "Trip, friends"]
        );
        for value in ["0", "1.001", "NaN", "12,34", "92233720368547758.08"] {
            assert!(
                parse(&fixture(value, "", "[]", "", "")).unwrap()[0]
                    .error
                    .is_some(),
                "{value}"
            );
        }
        for labels in ["not JSON", "[1]", "[\"\"]", "[\"Trip\",\"trip\"]"] {
            assert!(
                parse(&fixture("1", "", labels, "", "")).unwrap()[0]
                    .error
                    .is_some()
            );
        }
        assert!(
            parse(&fixture("1", "Uncategorized", "[]", "", "")).unwrap()[0]
                .error
                .is_some()
        );
        assert!(
            parse(&fixture("1", "", "[]", "Missing reference", "")).unwrap()[0]
                .error
                .is_some()
        );
        assert!(
            parse(&fixture("1", "", "[]", "Stream", "bad-reference")).unwrap()[0]
                .error
                .is_some()
        );
        assert_eq!(decimal(i64::MAX), "92233720368547758.07");
        assert_eq!(decimal(-1), "-0.01");
    }
}
