mod format;
use crate::{
    AppState, auth,
    error::{AppError, Result},
    rules,
};
use axum::{
    Json,
    extract::State,
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};
use format::*;
use sqlx::SqliteConnection;
use std::collections::HashMap;
use tower_sessions::Session;
use uuid::Uuid;

async fn empty(conn: &mut SqliteConnection, owner: &str) -> Result<()> {
    let occupied: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM classifications WHERE user_id=?1 UNION ALL SELECT 1 FROM transaction_sources WHERE user_id=?1 UNION ALL SELECT 1 FROM import_profiles WHERE user_id=?1 UNION ALL SELECT 1 FROM label_rules WHERE user_id=?1 UNION ALL SELECT 1 FROM budget_months WHERE user_id=?1 UNION ALL SELECT 1 FROM transactions WHERE user_id=?1)").bind(owner).fetch_one(conn).await?;
    if occupied {
        return Err(AppError::invalid(
            "Restore settings into an empty account, before importing entries. Existing entries or setup will never be overwritten.",
        ));
    }
    Ok(())
}
async fn snapshot(conn: &mut SqliteConnection, owner: &str) -> Result<Settings> {
    let classifications = sqlx::query_as("SELECT id,kind,name,color,archived FROM classifications WHERE user_id=? ORDER BY kind,name,id").bind(owner).fetch_all(&mut *conn).await?;
    let source_rows: Vec<(String,String,String)> = sqlx::query_as("SELECT s.id,s.name,COALESCE((SELECT MIN(reference) FROM portable_source_refs r WHERE r.user_id=s.user_id AND r.source_id=s.id),s.id) FROM transaction_sources s WHERE s.user_id=? ORDER BY s.name,3").bind(owner).fetch_all(&mut *conn).await?;
    let source_refs: HashMap<_, _> = source_rows
        .iter()
        .map(|(id, _, reference)| (id.clone(), reference.clone()))
        .collect();
    let sources = source_rows
        .into_iter()
        .map(|(_, name, id)| Source { id, name })
        .collect();
    let rows: Vec<(String, String)> =
        sqlx::query_as("SELECT name,layout FROM import_profiles WHERE user_id=? ORDER BY name,id")
            .bind(owner)
            .fetch_all(&mut *conn)
            .await?;
    let profiles = rows
        .into_iter()
        .map(|(name, layout)| {
            Ok(Profile {
                name,
                layout: serde_json::from_str(&layout).map_err(AppError::internal)?,
            })
        })
        .collect::<Result<_>>()?;
    let rules = rules::snapshot(conn, owner)
        .await?
        .rules
        .into_iter()
        .map(|r| {
            Ok(Rule {
                name: r.name,
                operator: serde_json::from_value(serde_json::Value::String(r.operator))
                    .map_err(AppError::internal)?,
                pattern: r.pattern,
                source_id: r
                    .source_id
                    .map(|id| {
                        source_refs
                            .get(&id)
                            .cloned()
                            .ok_or_else(AppError::not_found)
                    })
                    .transpose()?,
                category_id: r.category_id,
                label_ids: r.label_ids,
                enabled: r.enabled,
                pause_reason: r.pause_reason,
            })
        })
        .collect::<Result<_>>()?;
    let months: Vec<(String,String)> = sqlx::query_as("SELECT month,CAST(savings_cents AS TEXT) FROM budget_months WHERE user_id=? ORDER BY month").bind(owner).fetch_all(&mut *conn).await?;
    let mut budgets = Vec::new();
    for (month, savings_cents) in months {
        let categories = sqlx::query_as("SELECT category_id,CAST(amount_cents AS TEXT) AS amount_cents FROM category_budgets WHERE user_id=? AND month=? ORDER BY category_id").bind(owner).bind(&month).fetch_all(&mut *conn).await?;
        let rows: Vec<(String,String,String,bool)> = sqlx::query_as("SELECT id,name,CAST(amount_cents AS TEXT),complete FROM expected_income WHERE user_id=? AND month=? ORDER BY rowid").bind(owner).bind(&month).fetch_all(&mut *conn).await?;
        let mut income = Vec::new();
        for (id, name, amount_cents, complete) in rows {
            let entry_refs = sqlx::query_scalar("SELECT COALESCE((SELECT MIN(reference) FROM portable_entry_refs p WHERE p.user_id=r.user_id AND p.transaction_id=r.transaction_id),r.transaction_id) FROM income_receipts r WHERE r.user_id=?1 AND r.expected_id=?2 UNION SELECT entry_ref FROM portable_income_receipts WHERE user_id=?1 AND expected_id=?2 ORDER BY 1").bind(owner).bind(id).fetch_all(&mut *conn).await?;
            income.push(Income {
                name,
                amount_cents,
                complete,
                entry_refs,
            });
        }
        budgets.push(Budget {
            month,
            savings_cents,
            categories,
            income,
        });
    }
    let setup_complete =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM classification_setup WHERE user_id=?)")
            .bind(owner)
            .fetch_one(conn)
            .await?;
    Ok(Settings {
        format: "coinz-settings".into(),
        version: 1,
        setup_complete,
        classifications,
        sources,
        profiles,
        rules,
        budgets,
    })
}
pub async fn export(State(state): State<AppState>, session: Session) -> Result<Response> {
    let user = auth::user(&session, &state.pool).await?;
    let mut tx = state.pool.begin().await?;
    let settings = snapshot(&mut tx, &user.id).await?;
    settings.validate()?;
    let bytes = serde_json::to_vec_pretty(&settings).map_err(AppError::internal)?;
    if bytes.len() > MAX_BYTES {
        return Err(AppError::invalid(
            "Settings exceed the 8 MiB export limit. No partial file was produced.",
        ));
    }
    tx.commit().await?;
    Ok((
        [
            (header::CONTENT_TYPE, "application/json; charset=utf-8"),
            (
                header::CONTENT_DISPOSITION,
                "attachment; filename=\"coinz-settings.json\"",
            ),
        ],
        bytes,
    )
        .into_response())
}
pub async fn preview(
    State(state): State<AppState>,
    session: Session,
    Json(settings): Json<Settings>,
) -> Result<Json<Summary>> {
    let user = auth::user(&session, &state.pool).await?;
    let summary = settings.validate()?;
    let mut tx = state.pool.begin().await?;
    empty(&mut tx, &user.id).await?;
    tx.commit().await?;
    Ok(Json(summary))
}
pub async fn restore(
    State(state): State<AppState>,
    session: Session,
    Json(settings): Json<Settings>,
) -> Result<StatusCode> {
    let user = auth::user(&session, &state.pool).await?;
    settings.validate()?;
    let mut tx = state.pool.begin_with("BEGIN IMMEDIATE").await?;
    empty(&mut tx, &user.id).await?;
    let mut ids = HashMap::new();
    for item in &settings.classifications {
        let id = Uuid::new_v4().to_string();
        sqlx::query("INSERT INTO classifications(id,user_id,kind,name,name_key,color,archived) VALUES(?,?,?,?,?,?,?)").bind(&id).bind(&user.id).bind(&item.kind).bind(item.name.trim()).bind(item.name.trim().to_lowercase()).bind(&item.color).bind(item.archived).execute(&mut *tx).await?;
        ids.insert(item.id.clone(), id);
    }
    let mut sources = HashMap::new();
    for source in &settings.sources {
        let id = Uuid::new_v4().to_string();
        sqlx::query("INSERT INTO transaction_sources(id,user_id,name) VALUES(?,?,?)")
            .bind(&id)
            .bind(&user.id)
            .bind(source.name.trim())
            .execute(&mut *tx)
            .await?;
        sqlx::query("INSERT INTO portable_source_refs(user_id,reference,source_id) VALUES(?,?,?)")
            .bind(&user.id)
            .bind(&source.id)
            .bind(&id)
            .execute(&mut *tx)
            .await?;
        sources.insert(source.id.clone(), id);
    }
    for profile in &settings.profiles {
        sqlx::query("INSERT INTO import_profiles(id,user_id,name,layout) VALUES(?,?,?,?)")
            .bind(Uuid::new_v4().to_string())
            .bind(&user.id)
            .bind(profile.name.trim())
            .bind(serde_json::to_string(&profile.layout).map_err(AppError::internal)?)
            .execute(&mut *tx)
            .await?;
    }
    // All references were checked before writing; lookup errors still roll back the entire restore.
    let mapped = |map: &HashMap<String, String>, id: &String| {
        map.get(id).cloned().ok_or_else(AppError::not_found)
    };
    for rule in &settings.rules {
        let id = Uuid::new_v4().to_string();
        sqlx::query("INSERT INTO label_rules(id,user_id,name,operator,pattern,source_id,category_id,enabled,pause_reason) VALUES(?,?,?,?,?,?,?,?,?)")
            .bind(&id).bind(&user.id).bind(rule.name.trim()).bind(rule.operator.as_str()).bind(rule.pattern.trim())
            .bind(rule.source_id.as_ref().map(|id| mapped(&sources,id)).transpose()?).bind(rule.category_id.as_ref().map(|id| mapped(&ids,id)).transpose()?).bind(rule.enabled).bind(&rule.pause_reason).execute(&mut *tx).await?;
        for label in &rule.label_ids {
            sqlx::query("INSERT INTO label_rule_labels(rule_id,user_id,label_id) VALUES(?,?,?)")
                .bind(&id)
                .bind(&user.id)
                .bind(mapped(&ids, label)?)
                .execute(&mut *tx)
                .await?;
        }
    }
    rules::bump(&mut tx, &user.id).await?;
    for budget in &settings.budgets {
        sqlx::query("INSERT INTO budget_months(user_id,month,savings_cents) VALUES(?,?,?)")
            .bind(&user.id)
            .bind(&budget.month)
            .bind(cents(&budget.savings_cents)?)
            .execute(&mut *tx)
            .await?;
        for target in &budget.categories {
            sqlx::query("INSERT INTO category_budgets(user_id,month,category_id,amount_cents) VALUES(?,?,?,?)").bind(&user.id).bind(&budget.month).bind(mapped(&ids,&target.category_id)?).bind(cents(&target.amount_cents)?).execute(&mut *tx).await?;
        }
        for income in &budget.income {
            let id = Uuid::new_v4().to_string();
            sqlx::query("INSERT INTO expected_income(id,user_id,month,name,amount_cents,complete) VALUES(?,?,?,?,?,?)").bind(&id).bind(&user.id).bind(&budget.month).bind(income.name.trim()).bind(cents(&income.amount_cents)?).bind(income.complete).execute(&mut *tx).await?;
            for reference in &income.entry_refs {
                sqlx::query("INSERT INTO portable_income_receipts(user_id,entry_ref,expected_id) VALUES(?,?,?)").bind(&user.id).bind(reference).bind(&id).execute(&mut *tx).await?;
            }
        }
    }
    sqlx::query("DELETE FROM classification_setup WHERE user_id=?")
        .bind(&user.id)
        .execute(&mut *tx)
        .await?;
    if settings.setup_complete {
        sqlx::query("INSERT OR IGNORE INTO classification_setup(user_id) VALUES(?)")
            .bind(&user.id)
            .execute(&mut *tx)
            .await?;
    }
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn reconnect(
    conn: &mut SqliteConnection,
    owner: &str,
    entry: &str,
    reference: &str,
) -> Result<()> {
    let expected: Option<String> = sqlx::query_scalar(
        "SELECT expected_id FROM portable_income_receipts WHERE user_id=? AND entry_ref=?",
    )
    .bind(owner)
    .bind(reference)
    .fetch_optional(&mut *conn)
    .await?;
    if let Some(expected) = expected {
        let current: Option<String> = sqlx::query_scalar(
            "SELECT expected_id FROM income_receipts WHERE user_id=? AND transaction_id=?",
        )
        .bind(owner)
        .bind(entry)
        .fetch_optional(&mut *conn)
        .await?;
        if current.as_ref().is_some_and(|id| id != &expected) {
            return Err(AppError::invalid(
                "This entry is already linked to different expected income. Resolve the budget receipt before restoring it.",
            ));
        }
        sqlx::query("INSERT OR IGNORE INTO income_receipts(user_id,expected_id,transaction_id) VALUES(?,?,?)").bind(owner).bind(&expected).bind(entry).execute(&mut *conn).await?;
        if current.is_none() {
            sqlx::query("UPDATE transactions SET revision=revision+1 WHERE user_id=? AND id=?")
                .bind(owner)
                .bind(entry)
                .execute(&mut *conn)
                .await?;
            sqlx::query("UPDATE budget_months SET revision=revision+1 WHERE user_id=? AND month=(SELECT month FROM expected_income WHERE id=? AND user_id=?)").bind(owner).bind(&expected).bind(owner).execute(&mut *conn).await?;
        }
        sqlx::query("DELETE FROM portable_income_receipts WHERE user_id=? AND entry_ref=?")
            .bind(owner)
            .bind(reference)
            .execute(conn)
            .await?;
    }
    Ok(())
}
