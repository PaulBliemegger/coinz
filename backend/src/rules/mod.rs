pub mod history;
mod matching;
pub use matching::Operator;

use crate::{
    AppState, auth,
    classification::{self, Kind},
    domain,
    error::{AppError, Result},
};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqliteConnection};
use std::collections::BTreeSet;
use tower_sessions::Session;
use uuid::Uuid;

#[derive(Clone, Serialize, FromRow)]
pub struct Rule {
    pub id: String,
    pub name: String,
    pub operator: String,
    pub pattern: String,
    pub source_id: Option<String>,
    pub category_id: Option<String>,
    pub enabled: bool,
    pub revision: i64,
    pub pause_reason: Option<String>,
    #[sqlx(skip)]
    pub label_ids: Vec<String>,
}
#[derive(Serialize)]
pub struct Snapshot {
    pub revision: i64,
    pub rules: Vec<Rule>,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    name: String,
    operator: Operator,
    pattern: String,
    source_id: Option<String>,
    category_id: Option<String>,
    #[serde(default)]
    label_ids: Vec<String>,
    enabled: bool,
    revision: Option<i64>,
}
impl Rule {
    fn matches(&self, description: &str, source: Option<&str>) -> bool {
        if self
            .source_id
            .as_deref()
            .is_some_and(|id| Some(id) != source)
        {
            return false;
        }
        let op = match self.operator.as_str() {
            "contains" => Operator::Contains,
            "contains_word" => Operator::ContainsWord,
            "equals" => Operator::Equals,
            _ => return false,
        };
        op.matches(&self.pattern, description)
    }
}
pub async fn revision(conn: &mut SqliteConnection, owner: &str) -> Result<i64> {
    Ok(
        sqlx::query_scalar("SELECT revision FROM label_rule_state WHERE user_id=?")
            .bind(owner)
            .fetch_optional(conn)
            .await?
            .unwrap_or(0),
    )
}
pub async fn bump(conn: &mut SqliteConnection, owner: &str) -> Result<()> {
    sqlx::query("INSERT INTO label_rule_state(user_id,revision) VALUES(?,1) ON CONFLICT(user_id) DO UPDATE SET revision=revision+1").bind(owner).execute(conn).await?;
    Ok(())
}
pub async fn snapshot(conn: &mut SqliteConnection, owner: &str) -> Result<Snapshot> {
    let revision = revision(conn, owner).await?;
    let mut rules: Vec<Rule> = sqlx::query_as("SELECT id,name,operator,pattern,source_id,category_id,enabled,revision,pause_reason FROM label_rules WHERE user_id=? ORDER BY name,id").bind(owner).fetch_all(&mut *conn).await?;
    for rule in &mut rules {
        rule.label_ids = sqlx::query_scalar("SELECT label_id FROM label_rule_labels WHERE rule_id=? AND user_id=? ORDER BY label_id").bind(&rule.id).bind(owner).fetch_all(&mut *conn).await?;
    }
    Ok(Snapshot { revision, rules })
}
pub async fn list(State(state): State<AppState>, session: Session) -> Result<Json<Snapshot>> {
    let user = auth::user(&session, &state.pool).await?;
    let mut tx = state.pool.begin().await?;
    let result = snapshot(&mut tx, &user.id).await?;
    tx.commit().await?;
    Ok(Json(result))
}
async fn validate(
    conn: &mut SqliteConnection,
    owner: &str,
    input: &Definition,
) -> Result<(String, String)> {
    let name = domain::text(&input.name, "Rule name", 60)?;
    let pattern = domain::text(&input.pattern, "Description text", 200)?;
    if matches!(input.operator, Operator::ContainsWord)
        && !pattern.chars().all(char::is_alphanumeric)
    {
        return Err(AppError::invalid(
            "Contains word accepts one word without spaces or punctuation. Use Contains for a phrase.",
        ));
    }
    classification::validate_ids(&input.label_ids, true)?;
    if input.category_id.is_none() && input.label_ids.is_empty() {
        return Err(AppError::invalid(
            "Choose a category or at least one label.",
        ));
    }
    if let Some(id) = &input.source_id {
        let owned: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM transaction_sources WHERE id=? AND user_id=?)",
        )
        .bind(id)
        .bind(owner)
        .fetch_one(&mut *conn)
        .await?;
        if !owned {
            return Err(AppError::not_found());
        }
    }
    for (kind, ids) in [
        (Kind::Category, input.category_id.iter().collect::<Vec<_>>()),
        (Kind::Label, input.label_ids.iter().collect()),
    ] {
        for id in ids {
            let item = classification::owned_item(conn, owner, id, kind).await?;
            if input.enabled && item.archived {
                return Err(AppError::invalid(
                    "Restore archived targets before enabling this rule.",
                ));
            }
        }
    }
    Ok((name, pattern))
}
async fn write_labels(
    conn: &mut SqliteConnection,
    owner: &str,
    rule: &str,
    labels: &[String],
) -> Result<()> {
    sqlx::query("DELETE FROM label_rule_labels WHERE user_id=? AND rule_id=?")
        .bind(owner)
        .bind(rule)
        .execute(&mut *conn)
        .await?;
    for id in labels {
        sqlx::query("INSERT INTO label_rule_labels(rule_id,user_id,label_id) VALUES(?,?,?)")
            .bind(rule)
            .bind(owner)
            .bind(id)
            .execute(&mut *conn)
            .await?;
    }
    Ok(())
}
pub async fn create(
    State(state): State<AppState>,
    session: Session,
    Json(input): Json<Definition>,
) -> Result<(StatusCode, Json<serde_json::Value>)> {
    let user = auth::user(&session, &state.pool).await?;
    let mut tx = state.pool.begin_with("BEGIN IMMEDIATE").await?;
    let (name, pattern) = validate(&mut tx, &user.id, &input).await?;
    let id = Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO label_rules(id,user_id,name,operator,pattern,source_id,category_id,enabled) VALUES(?,?,?,?,?,?,?,?)").bind(&id).bind(&user.id).bind(name).bind(input.operator.as_str()).bind(pattern).bind(&input.source_id).bind(&input.category_id).bind(input.enabled).execute(&mut *tx).await?;
    write_labels(&mut tx, &user.id, &id, &input.label_ids).await?;
    bump(&mut tx, &user.id).await?;
    tx.commit().await?;
    Ok((StatusCode::CREATED, Json(serde_json::json!({"id":id}))))
}
pub async fn update(
    State(state): State<AppState>,
    session: Session,
    Path(id): Path<String>,
    Json(input): Json<Definition>,
) -> Result<StatusCode> {
    let user = auth::user(&session, &state.pool).await?;
    let mut tx = state.pool.begin_with("BEGIN IMMEDIATE").await?;
    let current: i64 =
        sqlx::query_scalar("SELECT revision FROM label_rules WHERE id=? AND user_id=?")
            .bind(&id)
            .bind(&user.id)
            .fetch_optional(&mut *tx)
            .await?
            .ok_or_else(AppError::not_found)?;
    if input.revision != Some(current) {
        return Err(AppError::invalid(
            "This rule changed. Reload before saving.",
        ));
    }
    let (name, pattern) = validate(&mut tx, &user.id, &input).await?;
    sqlx::query("UPDATE label_rules SET name=?,operator=?,pattern=?,source_id=?,category_id=?,enabled=?,revision=revision+1,pause_reason=NULL WHERE id=? AND user_id=?").bind(name).bind(input.operator.as_str()).bind(pattern).bind(&input.source_id).bind(&input.category_id).bind(input.enabled).bind(&id).bind(&user.id).execute(&mut *tx).await?;
    write_labels(&mut tx, &user.id, &id, &input.label_ids).await?;
    bump(&mut tx, &user.id).await?;
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuleRevision {
    revision: i64,
}
pub async fn delete(
    State(state): State<AppState>,
    session: Session,
    Path(id): Path<String>,
    Json(input): Json<RuleRevision>,
) -> Result<StatusCode> {
    let user = auth::user(&session, &state.pool).await?;
    let mut tx = state.pool.begin_with("BEGIN IMMEDIATE").await?;
    let current: i64 =
        sqlx::query_scalar("SELECT revision FROM label_rules WHERE id=? AND user_id=?")
            .bind(&id)
            .bind(&user.id)
            .fetch_optional(&mut *tx)
            .await?
            .ok_or_else(AppError::not_found)?;
    if current != input.revision {
        return Err(AppError::invalid(
            "This rule changed. Reload before deleting.",
        ));
    }
    sqlx::query("DELETE FROM label_rules WHERE id=? AND user_id=?")
        .bind(id)
        .bind(&user.id)
        .execute(&mut *tx)
        .await?;
    bump(&mut tx, &user.id).await?;
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Clone, Serialize)]
pub struct MatchedRule {
    id: String,
    name: String,
    operator: String,
    pattern: String,
    category_id: Option<String>,
    label_ids: Vec<String>,
}
#[derive(Default, Clone, Serialize)]
pub struct Suggestions {
    pub revision: i64,
    pub category_id: Option<String>,
    pub category_candidates: Vec<String>,
    pub label_ids: Vec<String>,
    pub conflict: bool,
    pub matched_rules: Vec<MatchedRule>,
}
pub fn suggest(snapshot: &Snapshot, description: &str, source: Option<&str>) -> Suggestions {
    let hits: Vec<_> = snapshot
        .rules
        .iter()
        .filter(|r| r.enabled && r.matches(description, source))
        .collect();
    let categories: Vec<_> = hits
        .iter()
        .filter_map(|r| r.category_id.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    Suggestions {
        revision: snapshot.revision,
        category_id: if categories.len() == 1 {
            categories.first().cloned()
        } else {
            None
        },
        conflict: categories.len() > 1,
        category_candidates: categories,
        label_ids: hits
            .iter()
            .flat_map(|r| r.label_ids.clone())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect(),
        matched_rules: hits
            .iter()
            .map(|r| MatchedRule {
                id: r.id.clone(),
                name: r.name.clone(),
                operator: r.operator.clone(),
                pattern: r.pattern.clone(),
                category_id: r.category_id.clone(),
                label_ids: r.label_ids.clone(),
            })
            .collect(),
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SuggestInput {
    description: String,
    source_id: Option<String>,
}
pub async fn suggestions(
    State(state): State<AppState>,
    session: Session,
    Json(input): Json<SuggestInput>,
) -> Result<Json<Suggestions>> {
    let user = auth::user(&session, &state.pool).await?;
    let description = domain::text(&input.description, "Description", 200)?;
    let mut tx = state.pool.begin().await?;
    if let Some(id) = &input.source_id {
        let owned: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM transaction_sources WHERE id=? AND user_id=?)",
        )
        .bind(id)
        .bind(&user.id)
        .fetch_one(&mut *tx)
        .await?;
        if !owned {
            return Err(AppError::not_found());
        }
    }
    let rules = snapshot(&mut tx, &user.id).await?;
    let result = suggest(&rules, &description, input.source_id.as_deref());
    tx.commit().await?;
    Ok(Json(result))
}

pub async fn affected_rules(
    conn: &mut SqliteConnection,
    owner: &str,
    target: &str,
) -> Result<Vec<(String, i64)>> {
    Ok(sqlx::query_as("SELECT r.id,r.revision FROM label_rules r WHERE r.user_id=? AND (r.category_id=? OR EXISTS(SELECT 1 FROM label_rule_labels l WHERE l.rule_id=r.id AND l.user_id=r.user_id AND l.label_id=?)) ORDER BY r.id").bind(owner).bind(target).bind(target).fetch_all(conn).await?)
}
pub async fn pause_target(
    conn: &mut SqliteConnection,
    owner: &str,
    target: &str,
    deleting: bool,
) -> Result<()> {
    let rules = affected_rules(conn, owner, target).await?;
    for (id, _) in &rules {
        sqlx::query("UPDATE label_rules SET enabled=0,revision=revision+1,pause_reason=?,category_id=CASE WHEN ? AND category_id=? THEN NULL ELSE category_id END WHERE id=? AND user_id=?").bind(if deleting {"A target category or label was deleted. Review the actions before enabling."} else {"A target category or label was archived."}).bind(deleting).bind(target).bind(id).bind(owner).execute(&mut *conn).await?;
        if deleting {
            sqlx::query(
                "DELETE FROM label_rule_labels WHERE rule_id=? AND user_id=? AND label_id=?",
            )
            .bind(id)
            .bind(owner)
            .bind(target)
            .execute(&mut *conn)
            .await?;
        }
    }
    if !rules.is_empty() {
        bump(conn, owner).await?;
    }
    Ok(())
}
