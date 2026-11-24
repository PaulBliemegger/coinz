use crate::{
    AppState, auth, domain,
    error::{AppError, Result},
};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqliteConnection};
use tower_sessions::Session;
use uuid::Uuid;

#[derive(Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    Category,
    Label,
}
impl Kind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Category => "category",
            Self::Label => "label",
        }
    }
}

#[derive(Serialize, FromRow)]
pub struct Item {
    pub id: String,
    pub name: String,
    pub color: String,
    pub archived: bool,
}
#[derive(Serialize)]
pub struct Catalog {
    categories: Vec<Item>,
    labels: Vec<Item>,
    setup_complete: bool,
}

pub async fn catalog(State(state): State<AppState>, session: Session) -> Result<Json<Catalog>> {
    let user = auth::user(&session, &state.pool).await?;
    let mut tx = state.pool.begin().await?;
    let categories = items(&mut tx, &user.id, Kind::Category).await?;
    let labels = items(&mut tx, &user.id, Kind::Label).await?;
    let setup_complete =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM classification_setup WHERE user_id=?)")
            .bind(&user.id)
            .fetch_one(&mut *tx)
            .await?;
    tx.commit().await?;
    Ok(Json(Catalog {
        categories,
        labels,
        setup_complete,
    }))
}
async fn items(conn: &mut SqliteConnection, owner: &str, kind: Kind) -> Result<Vec<Item>> {
    Ok(sqlx::query_as("SELECT id,name,color,archived FROM classifications WHERE user_id=? AND kind=? ORDER BY CASE WHEN kind='label' THEN COALESCE((SELECT MAX(rowid) FROM entry_labels el WHERE el.user_id=classifications.user_id AND el.label_id=classifications.id),0) ELSE 0 END DESC,name_key,id").bind(owner).bind(kind.as_str()).fetch_all(conn).await?)
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NewItem {
    kind: Kind,
    name: String,
    color: String,
}
fn validated_name(name: &str, kind: Kind) -> Result<String> {
    let name = domain::text(name, "Name", 60)?;
    if kind == Kind::Category && name.to_lowercase() == "uncategorized" {
        return Err(AppError::invalid(
            "Uncategorized is the built-in fallback. Choose another name.",
        ));
    }
    Ok(name)
}
fn validate_color(color: &str) -> Result<()> {
    if !["sage", "clay", "sand", "lavender", "blue"].contains(&color) {
        return Err(AppError::invalid(
            "Choose one of the available label colours.",
        ));
    }
    Ok(())
}
async fn insert(conn: &mut SqliteConnection, owner: &str, input: &NewItem) -> Result<String> {
    let name = validated_name(&input.name, input.kind)?;
    validate_color(&input.color)?;
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM classifications WHERE user_id=? AND kind=? AND name_key=?)",
    )
    .bind(owner)
    .bind(input.kind.as_str())
    .bind(name.to_lowercase())
    .fetch_one(&mut *conn)
    .await?;
    if exists {
        return Err(AppError::invalid(
            "That name already exists, including archived items.",
        ));
    }
    let id = Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO classifications(id,user_id,kind,name,name_key,color) VALUES(?,?,?,?,?,?)",
    )
    .bind(&id)
    .bind(owner)
    .bind(input.kind.as_str())
    .bind(&name)
    .bind(name.to_lowercase())
    .bind(&input.color)
    .execute(conn)
    .await?;
    Ok(id)
}
pub async fn create(
    State(state): State<AppState>,
    session: Session,
    Json(input): Json<NewItem>,
) -> Result<(StatusCode, Json<serde_json::Value>)> {
    let user = auth::user(&session, &state.pool).await?;
    let mut tx = state.pool.begin_with("BEGIN IMMEDIATE").await?;
    let id = insert(&mut tx, &user.id, &input).await?;
    tx.commit().await?;
    Ok((StatusCode::CREATED, Json(serde_json::json!({"id":id}))))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpdateItem {
    name: String,
    color: String,
    archived: bool,
}
pub async fn update(
    State(state): State<AppState>,
    session: Session,
    Path(id): Path<String>,
    Json(input): Json<UpdateItem>,
) -> Result<StatusCode> {
    let user = auth::user(&session, &state.pool).await?;
    let mut tx = state.pool.begin_with("BEGIN IMMEDIATE").await?;
    let kind: String =
        sqlx::query_scalar("SELECT kind FROM classifications WHERE id=? AND user_id=?")
            .bind(&id)
            .bind(&user.id)
            .fetch_optional(&mut *tx)
            .await?
            .ok_or_else(AppError::not_found)?;
    let name = validated_name(
        &input.name,
        if kind == "category" {
            Kind::Category
        } else {
            Kind::Label
        },
    )?;
    validate_color(&input.color)?;
    let exists: bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM classifications WHERE user_id=? AND kind=? AND name_key=? AND id<>?)").bind(&user.id).bind(&kind).bind(name.to_lowercase()).bind(&id).fetch_one(&mut *tx).await?;
    if exists {
        return Err(AppError::invalid(
            "That name already exists, including archived items.",
        ));
    }
    sqlx::query(
        "UPDATE classifications SET name=?,name_key=?,color=?,archived=? WHERE user_id=? AND id=?",
    )
    .bind(&name)
    .bind(name.to_lowercase())
    .bind(input.color)
    .bind(input.archived)
    .bind(&user.id)
    .bind(&id)
    .execute(&mut *tx)
    .await?;
    if input.archived {
        crate::rules::pause_target(&mut tx, &user.id, &id, false).await?;
    }
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Setup {
    categories: Vec<String>,
    labels: Vec<String>,
}
pub async fn setup(
    State(state): State<AppState>,
    session: Session,
    Json(input): Json<Setup>,
) -> Result<StatusCode> {
    let user = auth::user(&session, &state.pool).await?;
    if input.categories.len() > 30 || input.labels.len() > 30 {
        return Err(AppError::invalid(
            "Choose at most 30 categories and labels during setup.",
        ));
    }
    let mut tx = state.pool.begin_with("BEGIN IMMEDIATE").await?;
    for (kind, names) in [
        (Kind::Category, input.categories),
        (Kind::Label, input.labels),
    ] {
        for name in names {
            let name = validated_name(&name, kind)?;
            let exists: bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM classifications WHERE user_id=? AND kind=? AND name_key=?)").bind(&user.id).bind(kind.as_str()).bind(name.to_lowercase()).fetch_one(&mut *tx).await?;
            if !exists {
                insert(
                    &mut tx,
                    &user.id,
                    &NewItem {
                        kind,
                        name,
                        color: "sage".into(),
                    },
                )
                .await?;
            }
        }
    }
    sqlx::query("INSERT OR IGNORE INTO classification_setup(user_id) VALUES(?)")
        .bind(&user.id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn owned_item(
    conn: &mut SqliteConnection,
    owner: &str,
    id: &str,
    kind: Kind,
) -> Result<Item> {
    sqlx::query_as(
        "SELECT id,name,color,archived FROM classifications WHERE id=? AND user_id=? AND kind=?",
    )
    .bind(id)
    .bind(owner)
    .bind(kind.as_str())
    .fetch_optional(conn)
    .await?
    .ok_or_else(AppError::not_found)
}
pub async fn entry_category(
    conn: &mut SqliteConnection,
    owner: &str,
    entry: &str,
) -> Result<Option<Item>> {
    Ok(sqlx::query_as("SELECT c.id,c.name,c.color,c.archived FROM entry_categories e JOIN classifications c ON c.id=e.category_id AND c.user_id=e.user_id WHERE e.user_id=? AND e.transaction_id=?").bind(owner).bind(entry).fetch_optional(conn).await?)
}
pub async fn entry_labels(
    conn: &mut SqliteConnection,
    owner: &str,
    entry: &str,
) -> Result<Vec<Item>> {
    Ok(sqlx::query_as("SELECT c.id,c.name,c.color,c.archived FROM entry_labels e JOIN classifications c ON c.id=e.label_id AND c.user_id=e.user_id WHERE e.user_id=? AND e.transaction_id=? ORDER BY c.name_key,c.id").bind(owner).bind(entry).fetch_all(conn).await?)
}
pub async fn set_category(
    conn: &mut SqliteConnection,
    owner: &str,
    entry: &str,
    category: Option<&str>,
) -> Result<()> {
    if let Some(id) = category {
        let item = owned_item(conn, owner, id, Kind::Category).await?;
        let current = entry_category(conn, owner, entry).await?;
        if item.archived && !current.is_some_and(|c| c.id == id) {
            return Err(AppError::invalid(
                "Restore this category before assigning it.",
            ));
        }
        sqlx::query("INSERT INTO entry_categories(transaction_id,user_id,category_id) VALUES(?,?,?) ON CONFLICT(transaction_id) DO UPDATE SET category_id=excluded.category_id").bind(entry).bind(owner).bind(id).execute(conn).await?;
    } else {
        sqlx::query("DELETE FROM entry_categories WHERE user_id=? AND transaction_id=?")
            .bind(owner)
            .bind(entry)
            .execute(conn)
            .await?;
    }
    Ok(())
}
pub async fn change_label(
    conn: &mut SqliteConnection,
    owner: &str,
    entry: &str,
    label: &str,
    add: bool,
) -> Result<()> {
    let item = owned_item(conn, owner, label, Kind::Label).await?;
    if add {
        let exists: bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM entry_labels WHERE user_id=? AND transaction_id=? AND label_id=?)").bind(owner).bind(entry).bind(label).fetch_one(&mut *conn).await?;
        if item.archived && !exists {
            return Err(AppError::invalid("Restore this label before assigning it."));
        }
        sqlx::query(
            "INSERT OR IGNORE INTO entry_labels(transaction_id,user_id,label_id) VALUES(?,?,?)",
        )
        .bind(entry)
        .bind(owner)
        .bind(label)
        .execute(conn)
        .await?;
    } else {
        sqlx::query("DELETE FROM entry_labels WHERE user_id=? AND transaction_id=? AND label_id=?")
            .bind(owner)
            .bind(entry)
            .bind(label)
            .execute(conn)
            .await?;
    }
    Ok(())
}
#[derive(Deserialize)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
pub enum Assignment {
    SetCategory { category_id: Option<String> },
    AddLabels { label_ids: Vec<String> },
    RemoveLabels { label_ids: Vec<String> },
}
#[derive(Deserialize)]
pub struct BulkAssignment {
    ids: Vec<String>,
    #[serde(flatten)]
    change: Assignment,
}
pub fn validate_ids(ids: &[String], allow_empty: bool) -> Result<()> {
    if (!allow_empty && ids.is_empty())
        || ids.len() > 100
        || ids.iter().collect::<std::collections::HashSet<_>>().len() != ids.len()
    {
        return Err(AppError::invalid(
            "Use up to 100 distinct IDs and select at least one item for this action.",
        ));
    }
    Ok(())
}
pub async fn assign(
    State(state): State<AppState>,
    session: Session,
    Json(input): Json<BulkAssignment>,
) -> Result<StatusCode> {
    let user = auth::user(&session, &state.pool).await?;
    validate_ids(&input.ids, false)?;
    let mut tx = state.pool.begin_with("BEGIN IMMEDIATE").await?;
    for id in &input.ids {
        let owned: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM transactions WHERE id=? AND user_id=?)",
        )
        .bind(id)
        .bind(&user.id)
        .fetch_one(&mut *tx)
        .await?;
        if !owned {
            return Err(AppError::not_found());
        }
        match &input.change {
            Assignment::SetCategory { category_id } => {
                set_category(&mut tx, &user.id, id, category_id.as_deref()).await?
            }
            Assignment::AddLabels { label_ids } | Assignment::RemoveLabels { label_ids } => {
                validate_ids(label_ids, false)?;
                for label in label_ids {
                    change_label(
                        &mut tx,
                        &user.id,
                        id,
                        label,
                        matches!(input.change, Assignment::AddLabels { .. }),
                    )
                    .await?;
                }
            }
        }
    }
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}
