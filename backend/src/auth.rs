use crate::{
    AppState,
    error::{AppError, Result},
};
use argon2::{
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::{SaltString, rand_core::OsRng},
};
use axum::{
    Json,
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::{FromRow, SqlitePool};
use tower_sessions::Session;
use uuid::Uuid;

#[derive(Clone, Serialize, FromRow)]
pub struct User {
    pub id: String,
    pub username: String,
}

pub fn hash_password(password: &str) -> std::result::Result<String, String> {
    Argon2::default()
        .hash_password(password.as_bytes(), &SaltString::generate(&mut OsRng))
        .map(|hash| hash.to_string())
        .map_err(|error| error.to_string())
}

pub async fn create_user(
    pool: &SqlitePool,
    username: &str,
    password: &str,
) -> std::result::Result<User, Box<dyn std::error::Error + Send + Sync>> {
    let username = normalize_username(username).map_err(|error| error.1)?;
    if !(12..=128).contains(&password.chars().count()) {
        return Err("Password must contain 12–128 characters.".into());
    }
    let password = password.to_owned();
    let hash = tokio::task::spawn_blocking(move || hash_password(&password)).await??;
    let user = User {
        id: Uuid::new_v4().to_string(),
        username,
    };
    sqlx::query("INSERT INTO users (id, username, password_hash) VALUES (?, ?, ?)")
        .bind(&user.id)
        .bind(&user.username)
        .bind(hash)
        .execute(pool)
        .await?;
    Ok(user)
}

fn normalize_username(username: &str) -> Result<String> {
    let username = username.trim().to_ascii_lowercase();
    if username.is_empty()
        || username.len() > 80
        || !username
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"_.-".contains(&c))
    {
        return Err(AppError::invalid(
            "Username must be 1–80 letters, numbers, dots, underscores, or hyphens.",
        ));
    }
    Ok(username)
}

pub async fn user(session: &Session, pool: &SqlitePool) -> Result<User> {
    let id = session
        .get::<String>("user_id")
        .await?
        .ok_or_else(AppError::unauthorized)?;
    sqlx::query_as::<_, User>("SELECT id, username FROM users WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(AppError::unauthorized)
}

async fn csrf_token(session: &Session) -> Result<String> {
    if let Some(token) = session.get::<String>("csrf_token").await? {
        return Ok(token);
    }
    let token = Uuid::new_v4().to_string();
    session.insert("csrf_token", &token).await?;
    Ok(token)
}

pub async fn session_info(State(state): State<AppState>, session: Session) -> Result<Json<Value>> {
    let user = match session.get::<String>("user_id").await? {
        Some(_) => Some(user(&session, &state.pool).await?),
        None => None,
    };
    Ok(Json(
        json!({ "user": user, "csrf_token": csrf_token(&session).await? }),
    ))
}

#[derive(Deserialize)]
pub struct Login {
    username: String,
    password: String,
}

pub async fn login(
    State(state): State<AppState>,
    session: Session,
    Json(input): Json<Login>,
) -> Result<Json<Value>> {
    let invalid = || {
        AppError(
            StatusCode::UNAUTHORIZED,
            "Username or password is incorrect.".into(),
        )
    };
    if input.password.len() > 512 || input.username.len() > 80 {
        return Err(invalid());
    }
    let username = normalize_username(&input.username).map_err(|_| invalid())?;
    let record: Option<(String, String, String)> =
        sqlx::query_as("SELECT id, username, password_hash FROM users WHERE username = ?")
            .bind(username)
            .fetch_optional(&state.pool)
            .await?;
    let hash = record
        .as_ref()
        .map(|row| row.2.clone())
        .unwrap_or(state.dummy_hash);
    let valid = tokio::task::spawn_blocking(move || {
        PasswordHash::new(&hash).is_ok_and(|hash| {
            Argon2::default()
                .verify_password(input.password.as_bytes(), &hash)
                .is_ok()
        })
    })
    .await
    .map_err(AppError::internal)?;
    if !valid {
        return Err(invalid());
    }
    let (id, username, _) = record.ok_or_else(invalid)?;
    session.cycle_id().await?;
    session.insert("user_id", &id).await?;
    session
        .insert("csrf_token", Uuid::new_v4().to_string())
        .await?;
    Ok(Json(
        json!({ "user": User { id, username }, "csrf_token": csrf_token(&session).await? }),
    ))
}

pub async fn logout(session: Session) -> Result<StatusCode> {
    session.flush().await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn csrf(
    State(state): State<AppState>,
    session: Session,
    request: Request<Body>,
    next: Next,
) -> Result<Response> {
    if !matches!(
        *request.method(),
        axum::http::Method::GET | axum::http::Method::HEAD | axum::http::Method::OPTIONS
    ) {
        let expected = session.get::<String>("csrf_token").await?;
        let supplied = request
            .headers()
            .get("x-csrf-token")
            .and_then(|v| v.to_str().ok());
        let origin_ok = request
            .headers()
            .get("origin")
            .is_none_or(|value| value.to_str().is_ok_and(|origin| origin == state.origin));
        if !origin_ok
            || expected
                .as_deref()
                .zip(supplied)
                .is_none_or(|(a, b)| a != b)
        {
            return Err(AppError(
                StatusCode::FORBIDDEN,
                "Your session changed. Refresh the page and try again.".into(),
            ));
        }
    }
    Ok(next.run(request).await)
}
