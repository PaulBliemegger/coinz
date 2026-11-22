pub mod auth;
pub mod db;
pub mod domain;
pub mod error;
pub mod imports;
pub mod ledger;

use axum::{
    Router,
    body::Body,
    http::{HeaderValue, Request},
    middleware::{self, Next},
    response::Response,
    routing::{get, post},
};
use sqlx::SqlitePool;
use std::path::PathBuf;
use time::Duration;
use tower_http::{
    limit::RequestBodyLimitLayer,
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};
use tower_sessions::{ExpiredDeletion, Expiry, SessionManagerLayer, cookie::SameSite};
use tower_sessions_sqlx_store::SqliteStore;

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub origin: String,
    pub dummy_hash: String,
}

#[derive(Clone)]
pub struct Config {
    pub origin: String,
    pub secure_cookie: bool,
    pub static_dir: PathBuf,
}

pub async fn app(
    pool: SqlitePool,
    config: Config,
) -> Result<Router, Box<dyn std::error::Error + Send + Sync>> {
    let store = SqliteStore::new(pool.clone());
    store.migrate().await?;
    store.delete_expired().await?;
    let session_layer = SessionManagerLayer::new(store)
        .with_name("coinz_session")
        .with_http_only(true)
        .with_secure(config.secure_cookie)
        .with_same_site(SameSite::Strict)
        .with_expiry(Expiry::OnInactivity(Duration::days(7)));
    let dummy_hash =
        tokio::task::spawn_blocking(|| auth::hash_password("unusable-dummy-password")).await??;
    let state = AppState {
        pool,
        origin: config.origin,
        dummy_hash,
    };
    let api = Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/session", get(auth::session_info))
        .route("/login", post(auth::login))
        .route("/logout", post(auth::logout))
        .route("/sources", get(ledger::sources).post(ledger::create_source))
        .route(
            "/transactions",
            get(ledger::transactions).post(ledger::create_transaction),
        )
        .route(
            "/transactions/{id}",
            axum::routing::put(ledger::update_transaction)
                .patch(ledger::change_field)
                .delete(ledger::delete_transaction),
        )
        .route("/transactions/bulk-delete", post(ledger::bulk_delete))
        .route("/transactions/{id}/provenance", get(ledger::provenance))
        .route("/summary", get(ledger::summary))
        .fallback(|| async { crate::error::AppError::not_found() })
        .layer(RequestBodyLimitLayer::new(16 * 1024))
        .merge(
            Router::new()
                .route("/imports/preview", post(imports::preview))
                .route("/imports/commit", post(imports::commit))
                .layer(RequestBodyLimitLayer::new(2 * 1024 * 1024)),
        )
        .layer(middleware::from_fn_with_state(state.clone(), auth::csrf))
        .layer(middleware::from_fn(no_cache));
    let index = config.static_dir.join("index.html");
    Ok(Router::new()
        .nest("/api", api)
        .fallback_service(ServeDir::new(config.static_dir).not_found_service(ServeFile::new(index)))
        .with_state(state)
        .layer(session_layer)
        .layer(TraceLayer::new_for_http())
        .layer(middleware::from_fn(security_headers)))
}

async fn no_cache(request: Request<Body>, next: Next) -> Response {
    let mut response = next.run(request).await;
    response
        .headers_mut()
        .insert("cache-control", HeaderValue::from_static("no-store"));
    response
}

async fn security_headers(request: Request<Body>, next: Next) -> Response {
    let mut response = next.run(request).await;
    for (name, value) in [
        ("x-content-type-options", "nosniff"),
        ("x-frame-options", "DENY"),
        ("referrer-policy", "same-origin"),
    ] {
        response
            .headers_mut()
            .insert(name, HeaderValue::from_static(value));
    }
    response
}
