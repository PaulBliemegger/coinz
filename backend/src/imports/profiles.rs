use crate::{
    AppState, auth, domain,
    error::{AppError, Result},
};
use axum::{
    Json,
    extract::{Path, State},
};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqliteConnection};
use tower_sessions::Session;
use uuid::Uuid;

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Layout {
    pub date: String,
    pub amount: String,
    pub description: String,
    pub external_id: Option<String>,
    pub delimiter: String,
    pub date_format: String,
    pub decimal: String,
    pub grouping: String,
}
impl Layout {
    pub fn validate(&self) -> Result<()> {
        for header in [&self.date, &self.amount, &self.description]
            .into_iter()
            .chain(self.external_id.iter())
        {
            domain::text(header, "Mapped header", 200)?;
        }
        let mut unique = std::collections::HashSet::new();
        if [&self.date, &self.amount, &self.description]
            .into_iter()
            .chain(self.external_id.iter())
            .any(|h| !unique.insert(h))
        {
            return Err(AppError::invalid(
                "Choose a different column for each field.",
            ));
        }
        if ![",", ";"].contains(&self.delimiter.as_str())
            || !["YYYY-MM-DD", "DD.MM.YYYY", "DD/MM/YYYY"].contains(&self.date_format.as_str())
            || ![".", ","].contains(&self.decimal.as_str())
            || !["", ".", ",", " "].contains(&self.grouping.as_str())
            || self.decimal == self.grouping
        {
            return Err(AppError::invalid(
                "Choose supported delimiter, date, decimal and grouping settings.",
            ));
        }
        Ok(())
    }
}
#[derive(Serialize)]
pub struct Profile {
    id: String,
    name: String,
    revision: i64,
    layout: Layout,
}
#[derive(FromRow)]
struct Stored {
    id: String,
    name: String,
    revision: i64,
    layout: String,
}
impl Stored {
    fn decode(self) -> Result<Profile> {
        Ok(Profile {
            id: self.id,
            name: self.name,
            revision: self.revision,
            layout: serde_json::from_str(&self.layout).map_err(AppError::internal)?,
        })
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Save {
    name: String,
    layout: Layout,
    revision: Option<i64>,
}

pub async fn list(State(state): State<AppState>, session: Session) -> Result<Json<Vec<Profile>>> {
    let user = auth::user(&session, &state.pool).await?;
    let rows = sqlx::query_as::<_, Stored>(
        "SELECT id,name,revision,layout FROM import_profiles WHERE user_id=? ORDER BY name,id",
    )
    .bind(user.id)
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(
        rows.into_iter()
            .map(Stored::decode)
            .collect::<Result<_>>()?,
    ))
}
pub async fn create(
    State(state): State<AppState>,
    session: Session,
    Json(input): Json<Save>,
) -> Result<(axum::http::StatusCode, Json<Profile>)> {
    let user = auth::user(&session, &state.pool).await?;
    input.layout.validate()?;
    let name = domain::text(&input.name, "Profile name", 80)?;
    let id = Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO import_profiles(id,user_id,name,layout) VALUES(?,?,?,?)")
        .bind(&id)
        .bind(user.id)
        .bind(&name)
        .bind(serde_json::to_string(&input.layout).map_err(AppError::internal)?)
        .execute(&state.pool)
        .await?;
    Ok((
        axum::http::StatusCode::CREATED,
        Json(Profile {
            id,
            name,
            revision: 1,
            layout: input.layout,
        }),
    ))
}
pub async fn update(
    State(state): State<AppState>,
    session: Session,
    Path(id): Path<String>,
    Json(input): Json<Save>,
) -> Result<Json<Profile>> {
    let user = auth::user(&session, &state.pool).await?;
    input.layout.validate()?;
    let name = domain::text(&input.name, "Profile name", 80)?;
    let mut tx = state.pool.begin_with("BEGIN IMMEDIATE").await?;
    let old = load(&mut tx, &user.id, &id).await?;
    if input.revision != Some(old.revision) {
        return Err(AppError::invalid(
            "Profile changed. Reload profiles before saving changes.",
        ));
    }
    sqlx::query(
        "UPDATE import_profiles SET name=?,layout=?,revision=revision+1 WHERE id=? AND user_id=?",
    )
    .bind(&name)
    .bind(serde_json::to_string(&input.layout).map_err(AppError::internal)?)
    .bind(&id)
    .bind(user.id)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(Json(Profile {
        id,
        name,
        revision: old.revision + 1,
        layout: input.layout,
    }))
}
async fn load(conn: &mut SqliteConnection, user: &str, id: &str) -> Result<Profile> {
    sqlx::query_as::<_, Stored>(
        "SELECT id,name,revision,layout FROM import_profiles WHERE id=? AND user_id=?",
    )
    .bind(id)
    .bind(user)
    .fetch_optional(conn)
    .await?
    .ok_or_else(AppError::not_found)?
    .decode()
}
// A revision pins preview and commit to the same interpretation, checked under the write lock.
pub async fn resolve(
    conn: &mut SqliteConnection,
    user: &str,
    id: &str,
    revision: Option<i64>,
) -> Result<Layout> {
    let profile = load(conn, user, id).await?;
    if revision != Some(profile.revision) {
        return Err(AppError::invalid(
            "Profile changed. Preview again with the current profile.",
        ));
    }
    Ok(profile.layout)
}
