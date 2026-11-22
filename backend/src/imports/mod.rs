pub mod history;
mod parse;

use crate::{
    AppState, auth,
    error::{AppError, Result},
};
use axum::{Json, extract::State};
use parse::Row;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::{FromRow, SqliteConnection};
use std::collections::{HashMap, HashSet};
use tower_sessions::Session;
use uuid::Uuid;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Input {
    source_id: String,
    csv: String,
    filename: Option<String>,
    #[serde(default)]
    decisions: Vec<Decision>,
    selected_records: Option<Vec<usize>>,
}

#[derive(Deserialize)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
enum Decision {
    Keep {
        record: usize,
    },
    Skip {
        record: usize,
    },
    Match {
        record: usize,
        transaction_id: String,
    },
}

impl Decision {
    fn record(&self) -> usize {
        match self {
            Self::Keep { record } | Self::Skip { record } | Self::Match { record, .. } => *record,
        }
    }
}

#[derive(Serialize, FromRow)]
struct Candidate {
    id: String,
    date: String,
    description: String,
    amount_cents: String,
}

#[derive(Serialize)]
struct PreviewRow {
    #[serde(flatten)]
    row: Row,
    status: &'static str,
    candidates: Vec<Candidate>,
}

#[derive(Serialize)]
pub struct Preview {
    rows: Vec<PreviewRow>,
}

#[derive(FromRow)]
struct Evidence {
    date: String,
    description: String,
    amount_cents: i64,
    transaction_id: Option<String>,
}

async fn owned_source(conn: &mut SqliteConnection, user: &str, source: &str) -> Result<()> {
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM transaction_sources WHERE user_id = ? AND id = ?)",
    )
    .bind(user)
    .bind(source)
    .fetch_one(conn)
    .await?;
    if !exists {
        return Err(AppError::not_found());
    }
    Ok(())
}

async fn classify(
    conn: &mut SqliteConnection,
    user: &str,
    input: &Input,
    hash: &str,
    rows: Vec<Row>,
) -> Result<Preview> {
    owned_source(conn, user, &input.source_id).await?;
    // Inspect the whole file so ID-less rows also see stable-ID counterparts later on.
    let fields_with_ids: HashSet<_> = rows
        .iter()
        .filter(|row| row.error.is_none() && !row.external_id.is_empty())
        .map(|row| (row.date.clone(), row.description.clone(), row.cents))
        .collect();
    let mut seen_ids = HashSet::new();
    let mut seen_fields = HashSet::new();
    let mut result = Vec::new();
    for row in rows {
        let mut status = "new";
        let mut candidates = Vec::new();
        let repeated_id = !row.external_id.is_empty() && !seen_ids.insert(row.external_id.clone());
        let fields = (row.date.clone(), row.description.clone(), row.cents);
        let matches_identified_row = fields_with_ids.contains(&fields);
        let repeated_fields = !seen_fields.insert(fields);
        let repeated_file: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM import_records WHERE user_id=? AND source_id=? AND file_hash=? AND record_number=?)")
            .bind(user).bind(&input.source_id).bind(hash).bind(row.record as i64).fetch_one(&mut *conn).await?;
        if row.error.is_some() {
            status = "invalid";
        } else if repeated_file {
            status = "known";
        } else if repeated_id {
            status = "conflict";
        } else {
            let evidence = if row.external_id.is_empty() {
                None
            } else {
                sqlx::query_as::<_, Evidence>("SELECT date,description,amount_cents,transaction_id FROM import_records WHERE user_id=? AND source_id=? AND external_id=? AND disposition='accepted' LIMIT 1")
                    .bind(user).bind(&input.source_id).bind(&row.external_id).fetch_optional(&mut *conn).await?
            };
            if let Some(original) = evidence {
                status = if original.date == row.date
                    && original.description == row.description
                    && original.amount_cents == row.cents
                {
                    "known"
                } else {
                    "conflict"
                };
            } else {
                // Different stable IDs are separate purchases. Exact manual matches and
                // ID-less overlaps are suggestions only, including original edited fields.
                candidates = sqlx::query_as::<_, Candidate>(
                    "SELECT t.id,t.date,t.description,CAST(t.amount_cents AS TEXT) AS amount_cents FROM transactions t
                     WHERE t.user_id=? AND (t.source_id=? OR t.source_id IS NULL OR EXISTS(
                       SELECT 1 FROM import_records r WHERE r.user_id=t.user_id AND r.source_id=? AND r.transaction_id=t.id AND r.disposition='accepted'))
                     AND ((t.date=? AND t.description=? AND t.amount_cents=?) OR EXISTS(
                       SELECT 1 FROM import_records r WHERE r.user_id=t.user_id AND r.source_id=? AND r.transaction_id=t.id AND r.disposition='accepted' AND r.date=? AND r.description=? AND r.amount_cents=?))
                     AND (?='' OR NOT EXISTS(SELECT 1 FROM import_records r WHERE r.user_id=t.user_id AND r.source_id=? AND r.transaction_id=t.id AND r.external_id<>'' AND r.disposition='accepted'))
                     ORDER BY t.id LIMIT 20")
                    .bind(user).bind(&input.source_id).bind(&input.source_id).bind(&row.date).bind(&row.description).bind(row.cents)
                    .bind(&input.source_id).bind(&row.date).bind(&row.description).bind(row.cents)
                    .bind(&row.external_id).bind(&input.source_id).fetch_all(&mut *conn).await?;
                if !candidates.is_empty()
                    || (row.external_id.is_empty() && (repeated_fields || matches_identified_row))
                {
                    status = "review";
                }
            }
        }
        result.push(PreviewRow {
            row,
            status,
            candidates,
        });
    }
    Ok(Preview { rows: result })
}

pub async fn preview(
    State(state): State<AppState>,
    session: Session,
    Json(input): Json<Input>,
) -> Result<Json<Preview>> {
    let user = auth::user(&session, &state.pool).await?;
    let rows = parse::parse(&input.csv)?;
    let hash = format!("{:x}", Sha256::digest(input.csv.as_bytes()));
    let mut tx = state.pool.begin().await?;
    Ok(Json(
        classify(&mut tx, &user.id, &input, &hash, rows).await?,
    ))
}

#[derive(Serialize)]
pub struct Outcome {
    added: usize,
    matched: usize,
    skipped: usize,
    known: usize,
    latest_month: Option<String>,
    excluded: usize,
    income_cents: String,
    spending_cents: String,
}

pub async fn commit(
    State(state): State<AppState>,
    session: Session,
    Json(input): Json<Input>,
) -> Result<Json<Outcome>> {
    let user = auth::user(&session, &state.pool).await?;
    let rows = parse::parse(&input.csv)?;
    let hash = format!("{:x}", Sha256::digest(input.csv.as_bytes()));
    let mut decisions = HashMap::new();
    for decision in &input.decisions {
        if decisions.insert(decision.record(), decision).is_some()
            || !rows.iter().any(|r| r.record == decision.record())
        {
            return Err(AppError::invalid("Provide one decision per CSV record."));
        }
    }
    let selected = input
        .selected_records
        .as_ref()
        .map(|records| records.iter().copied().collect::<HashSet<_>>());
    if let (Some(records), Some(selected)) = (&input.selected_records, &selected)
        && (records.len() != selected.len()
            || records
                .iter()
                .any(|id| !rows.iter().any(|r| r.record == *id))
            || decisions.keys().any(|id| !selected.contains(id)))
    {
        return Err(AppError::invalid(
            "Select valid CSV record numbers and decide only selected rows.",
        ));
    }
    // Serialize both duplicate validation and writes, including simultaneous submits.
    let mut tx = state.pool.begin_with("BEGIN IMMEDIATE").await?;
    let preview = classify(&mut tx, &user.id, &input, &hash, rows).await?;
    let mut outcome = Outcome {
        added: 0,
        matched: 0,
        skipped: 0,
        known: 0,
        latest_month: None,
        excluded: 0,
        income_cents: String::new(),
        spending_cents: String::new(),
    };
    let batch = Uuid::new_v4().to_string();
    let filename = history::filename(input.filename.as_deref())?;
    let dates: Vec<_> = preview
        .rows
        .iter()
        .filter(|r| r.row.error.is_none())
        .map(|r| r.row.date.as_str())
        .collect();
    sqlx::query("INSERT INTO import_batches(id,user_id,source_id,filename,start_date,end_date) VALUES(?,?,?,?,?,?)")
        .bind(&batch).bind(&user.id).bind(&input.source_id).bind(filename).bind(dates.iter().min().copied()).bind(dates.iter().max().copied()).execute(&mut *tx).await?;
    let mut income = Vec::new();
    let mut spending = Vec::new();
    for item in preview.rows {
        let row = item.row;
        if selected
            .as_ref()
            .is_some_and(|ids| !ids.contains(&row.record))
            && item.status != "known"
        {
            outcome.excluded += 1;
            continue;
        }
        if item.status == "invalid" {
            return Err(AppError::invalid(format!(
                "Fix CSV record {} before importing.",
                row.record
            )));
        }
        if item.status == "known" {
            // Preserve another file's provenance without changing its linked entry.
            let evidence = sqlx::query_as::<_, Evidence>("SELECT date,description,amount_cents,transaction_id FROM import_records WHERE user_id=? AND source_id=? AND external_id=? AND external_id<>'' AND disposition='accepted' LIMIT 1")
                .bind(&user.id).bind(&input.source_id).bind(&row.external_id).fetch_optional(&mut *tx).await?;
            if let Some(evidence) = evidence {
                record(
                    &mut tx,
                    &user.id,
                    &input.source_id,
                    &hash,
                    &row,
                    evidence.transaction_id.as_deref(),
                    "accepted",
                )
                .await?;
            }
            let known_id: Option<String> = sqlx::query_scalar("SELECT transaction_id FROM import_records WHERE user_id=? AND source_id=? AND file_hash=? AND record_number=? AND disposition='accepted'")
                .bind(&user.id).bind(&input.source_id).bind(&hash).bind(row.record as i64).fetch_optional(&mut *tx).await?.flatten();
            if let Some(id) = known_id {
                history::link(&mut tx, &batch, &user.id, &id, "known").await?;
            }
            outcome.known += 1;
            continue;
        }
        let action = decisions.get(&row.record).copied();
        if item.status == "conflict" && !matches!(action, Some(Decision::Skip { .. })) {
            return Err(AppError::invalid(format!(
                "Record {} has a conflicting external ID. Skip it explicitly or correct the CSV.",
                row.record
            )));
        }
        if item.status == "review" && action.is_none() {
            return Err(AppError::invalid(format!(
                "Review possible duplicates for record {}.",
                row.record
            )));
        }
        let (id, disposition) = match action {
            Some(Decision::Skip { .. }) => {
                outcome.skipped += 1;
                (None, "skipped")
            }
            Some(Decision::Match { transaction_id, .. }) => {
                if !item.candidates.iter().any(|c| &c.id == transaction_id) {
                    return Err(AppError::invalid(
                        "The selected match is no longer available. Preview again.",
                    ));
                }
                history::link(&mut tx, &batch, &user.id, transaction_id, "matched").await?;
                outcome.matched += 1;
                (Some(transaction_id.clone()), "accepted")
            }
            Some(Decision::Keep { .. }) | None => {
                let id = Uuid::new_v4().to_string();
                sqlx::query("INSERT INTO transactions(id,user_id,source_id,date,description,amount_cents,origin) VALUES(?,?,?,?,?,?,'csv')")
                    .bind(&id).bind(&user.id).bind(&input.source_id).bind(&row.date).bind(&row.description).bind(row.cents).execute(&mut *tx).await?;
                history::link(&mut tx, &batch, &user.id, &id, "added").await?;
                outcome.added += 1;
                if row.cents > 0 {
                    income.push(row.cents);
                } else {
                    spending.push(-row.cents);
                }
                let month = row.date[..7].to_owned();
                if outcome
                    .latest_month
                    .as_ref()
                    .is_none_or(|current| current < &month)
                {
                    outcome.latest_month = Some(month);
                }
                (Some(id), "accepted")
            }
        };
        record(
            &mut tx,
            &user.id,
            &input.source_id,
            &hash,
            &row,
            id.as_deref(),
            disposition,
        )
        .await?;
    }
    outcome.income_cents = crate::domain::checked_total(income)?.to_string();
    outcome.spending_cents = crate::domain::checked_total(spending)?.to_string();
    sqlx::query(
        "UPDATE import_batches SET added=?,matched=?,known=?,excluded=? WHERE id=? AND user_id=?",
    )
    .bind(outcome.added as i64)
    .bind(outcome.matched as i64)
    .bind(outcome.known as i64)
    .bind(outcome.excluded as i64)
    .bind(&batch)
    .bind(&user.id)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(Json(outcome))
}

async fn record(
    conn: &mut SqliteConnection,
    user: &str,
    source: &str,
    hash: &str,
    row: &Row,
    id: Option<&str>,
    disposition: &str,
) -> Result<()> {
    sqlx::query("INSERT INTO import_records(user_id,source_id,file_hash,record_number,external_id,date,description,amount_cents,transaction_id,disposition) VALUES(?,?,?,?,?,?,?,?,?,?) ON CONFLICT(user_id,source_id,file_hash,record_number) DO NOTHING")
        .bind(user).bind(source).bind(hash).bind(row.record as i64).bind(&row.external_id).bind(&row.date).bind(&row.description).bind(row.cents).bind(id).bind(disposition).execute(conn).await?;
    Ok(())
}
