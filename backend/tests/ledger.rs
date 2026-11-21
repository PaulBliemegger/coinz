use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
};
use coinz::{Config, auth, db};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use sqlx::SqlitePool;
use tempfile::TempDir;
use tower::ServiceExt;

const PASSWORD: &str = "synthetic-test-password";
const ORIGIN: &str = "http://coinz.test";

struct Harness {
    _directory: TempDir,
    pool: SqlitePool,
    app: Router,
}

impl Harness {
    async fn new() -> Self {
        let directory = tempfile::tempdir().unwrap();
        let pool = db::connect(directory.path()).await.unwrap();
        auth::create_user(&pool, "alice", PASSWORD).await.unwrap();
        auth::create_user(&pool, "bob", PASSWORD).await.unwrap();
        let app = coinz::app(pool.clone(), config()).await.unwrap();
        Self {
            _directory: directory,
            pool,
            app,
        }
    }
    async fn client(&self, username: &str) -> Client {
        let mut client = Client {
            app: self.app.clone(),
            cookie: None,
            csrf: None,
        };
        assert_eq!(
            client.call("GET", "/api/session", None).await.0,
            StatusCode::OK
        );
        let (status, _) = client
            .call(
                "POST",
                "/api/login",
                Some(json!({ "username": username, "password": PASSWORD })),
            )
            .await;
        assert_eq!(status, StatusCode::OK);
        client
    }
}

fn config() -> Config {
    Config {
        origin: ORIGIN.into(),
        secure_cookie: false,
        static_dir: "/missing-static-dir".into(),
    }
}

#[derive(Clone)]
struct Client {
    app: Router,
    cookie: Option<String>,
    csrf: Option<String>,
}

impl Client {
    async fn call(&mut self, method: &str, path: &str, data: Option<Value>) -> (StatusCode, Value) {
        self.send(method, path, data, true, ORIGIN).await
    }
    async fn send(
        &mut self,
        method: &str,
        path: &str,
        data: Option<Value>,
        csrf: bool,
        origin: &str,
    ) -> (StatusCode, Value) {
        let mut request = Request::builder()
            .method(method)
            .uri(path)
            .header("origin", origin);
        if let Some(cookie) = &self.cookie {
            request = request.header("cookie", cookie);
        }
        if csrf && let Some(token) = &self.csrf {
            request = request.header("x-csrf-token", token);
        }
        let body = if let Some(data) = data {
            request = request.header("content-type", "application/json");
            Body::from(data.to_string())
        } else {
            Body::empty()
        };
        let response = self
            .app
            .clone()
            .oneshot(request.body(body).unwrap())
            .await
            .unwrap();
        let status = response.status();
        assert_eq!(response.headers()["cache-control"], "no-store");
        for value in response.headers().get_all("set-cookie") {
            let value = value.to_str().unwrap();
            if !value.starts_with("coinz_session=;") {
                assert!(value.contains("HttpOnly"));
                assert!(value.contains("SameSite=Strict"));
            }
            self.cookie = Some(value.split(';').next().unwrap().to_owned());
        }
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let value: Value = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
        if let Some(token) = value["csrf_token"].as_str() {
            self.csrf = Some(token.into());
        }
        (status, value)
    }
    async fn source(&mut self, name: &str) -> String {
        let (status, value) = self
            .call("POST", "/api/sources", Some(json!({"name":name})))
            .await;
        assert_eq!(status, StatusCode::CREATED, "{value}");
        value["id"].as_str().unwrap().into()
    }
    async fn entry(
        &mut self,
        source: Option<&str>,
        date: &str,
        amount: &str,
        kind: &str,
    ) -> String {
        let (status, value) = self
            .call(
                "POST",
                "/api/transactions",
                Some(entry(source, date, amount, kind)),
            )
            .await;
        assert_eq!(status, StatusCode::CREATED, "{value}");
        value["id"].as_str().unwrap().into()
    }
}

fn entry(source: Option<&str>, date: &str, amount: &str, kind: &str) -> Value {
    json!({ "source_id": source, "date": date, "amount": amount, "kind": kind, "description": "Synthetic entry" })
}

#[tokio::test]
async fn sessions_rotate_persist_and_require_csrf() {
    let h = Harness::new().await;
    let mut anonymous = Client {
        app: h.app.clone(),
        cookie: None,
        csrf: None,
    };
    for path in ["/api/sources", "/api/transactions", "/api/summary"] {
        assert_eq!(
            anonymous.call("GET", path, None).await.0,
            StatusCode::UNAUTHORIZED
        );
    }
    anonymous.call("GET", "/api/session", None).await;
    let old_cookie = anonymous.cookie.clone();
    let login = json!({ "username": "alice", "password": PASSWORD });
    assert_eq!(
        anonymous
            .send("POST", "/api/login", Some(login.clone()), false, ORIGIN)
            .await
            .0,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        anonymous
            .send(
                "POST",
                "/api/login",
                Some(login.clone()),
                true,
                "https://elsewhere.test"
            )
            .await
            .0,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        anonymous
            .call(
                "POST",
                "/api/login",
                Some(json!({ "username": "alice", "password": "wrong" }))
            )
            .await
            .0,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        anonymous.call("POST", "/api/login", Some(login)).await.0,
        StatusCode::OK
    );
    assert_ne!(old_cookie, anonymous.cookie);
    let mut stale = Client {
        app: h.app.clone(),
        cookie: old_cookie,
        csrf: None,
    };
    assert!(stale.call("GET", "/api/session", None).await.1["user"].is_null());
    // A new connection and router use the on-disk session, not process memory.
    let reopened = db::connect(h._directory.path()).await.unwrap();
    anonymous.app = coinz::app(reopened, config()).await.unwrap();
    assert_eq!(
        anonymous.call("GET", "/api/session", None).await.1["user"]["username"],
        "alice"
    );
    let logged_in_cookie = anonymous.cookie.clone();
    assert_eq!(
        anonymous.call("POST", "/api/logout", None).await.0,
        StatusCode::NO_CONTENT
    );
    anonymous.cookie = logged_in_cookie;
    assert_eq!(
        anonymous.call("GET", "/api/sources", None).await.0,
        StatusCode::UNAUTHORIZED
    );
}

#[tokio::test]
async fn manual_tracking_works_without_sources_balances_or_opening_dates() {
    let h = Harness::new().await;
    let mut alice = h.client("alice").await;
    assert_eq!(alice.call("GET", "/api/sources", None).await.1, json!([]));
    // Omitted and explicit-null source are both supported.
    let mut income = entry(None, "2026-11-02", "2000", "income");
    income.as_object_mut().unwrap().remove("source_id");
    assert_eq!(
        alice
            .call("POST", "/api/transactions", Some(income))
            .await
            .0,
        StatusCode::CREATED
    );
    let expense = alice.entry(None, "2026-11-03", "12,34", "expense").await;
    alice.entry(None, "2026-12-01", "999", "income").await;
    alice.entry(None, "2020-01-01", "10", "expense").await;
    let summary = alice
        .call("GET", "/api/summary?month=2026-11", None)
        .await
        .1;
    assert_eq!(summary["income_cents"], "200000");
    assert_eq!(summary["spending_cents"], "1234");
    assert_eq!(summary["net_cents"], "198766");
    assert_eq!(summary["transaction_count"], 2);
    let rows = alice
        .call("GET", "/api/transactions?month=2026-11", None)
        .await
        .1;
    assert!(rows[0]["source_id"].is_null());
    assert!(rows[0]["source_name"].is_null());
    assert_eq!(
        alice
            .call(
                "PUT",
                &format!("/api/transactions/{expense}"),
                Some(entry(None, "2026-11-03", "13.34", "expense"))
            )
            .await
            .0,
        StatusCode::NO_CONTENT
    );
    assert_eq!(
        alice
            .call("GET", "/api/summary?month=2026-11", None)
            .await
            .1["spending_cents"],
        "1334"
    );
    assert_eq!(
        alice
            .call("DELETE", &format!("/api/transactions/{expense}"), None)
            .await
            .0,
        StatusCode::NO_CONTENT
    );
    assert_eq!(
        alice
            .call("GET", "/api/summary?month=2026-11", None)
            .await
            .1["spending_cents"],
        "0"
    );
}

#[tokio::test]
async fn source_metadata_filters_entries_and_can_be_attached_or_removed() {
    let h = Harness::new().await;
    let mut alice = h.client("alice").await;
    let source = alice.source("N26").await;
    let manual = alice.entry(None, "2026-11-01", "20", "expense").await;
    alice
        .entry(Some(&source), "2026-11-01", "10", "expense")
        .await;
    let path = format!("/api/summary?month=2026-11&source_id={source}");
    assert_eq!(
        alice.call("GET", &path, None).await.1["spending_cents"],
        "1000"
    );
    assert_eq!(
        alice
            .call("GET", "/api/summary?month=2026-11", None)
            .await
            .1["spending_cents"],
        "3000"
    );
    assert_eq!(
        alice
            .call(
                "PUT",
                &format!("/api/transactions/{manual}"),
                Some(entry(Some(&source), "2026-11-01", "20", "expense"))
            )
            .await
            .0,
        StatusCode::NO_CONTENT
    );
    assert_eq!(
        alice.call("GET", &path, None).await.1["spending_cents"],
        "3000"
    );
    assert_eq!(
        alice
            .call(
                "PUT",
                &format!("/api/transactions/{manual}"),
                Some(entry(None, "2026-11-01", "20", "expense"))
            )
            .await
            .0,
        StatusCode::NO_CONTENT
    );
    assert_eq!(
        alice.call("GET", &path, None).await.1["spending_cents"],
        "1000"
    );
    let rows = alice
        .call(
            "GET",
            &format!("/api/transactions?month=2026-11&source_id={source}"),
            None,
        )
        .await
        .1;
    assert_eq!(rows[0]["source_name"], "N26");
    assert_eq!(rows.as_array().unwrap().len(), 1);
    let sources = alice.call("GET", "/api/sources", None).await.1;
    assert!(sources[0].get("balance_cents").is_none());
}

#[tokio::test]
async fn sources_and_sourceless_entries_are_isolated_between_users() {
    let h = Harness::new().await;
    let mut alice = h.client("alice").await;
    let mut bob = h.client("bob").await;
    let source = alice.source("N26").await;
    let transaction = alice.entry(None, "2026-11-02", "10", "expense").await;
    let bob_source = bob.source("Erste").await;
    assert_eq!(
        bob.call("GET", "/api/transactions?month=2026-11", None)
            .await
            .1,
        json!([])
    );
    assert_eq!(
        bob.call("GET", "/api/summary?month=2026-11", None).await.1["transaction_count"],
        0
    );
    for path in [
        format!("/api/summary?month=2026-11&source_id={source}"),
        format!("/api/transactions?month=2026-11&source_id={source}"),
    ] {
        assert_eq!(bob.call("GET", &path, None).await.0, StatusCode::NOT_FOUND);
    }
    assert_eq!(
        bob.call(
            "POST",
            "/api/transactions",
            Some(entry(Some(&source), "2026-11-02", "99", "expense"))
        )
        .await
        .0,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        bob.call(
            "PUT",
            &format!("/api/transactions/{transaction}"),
            Some(entry(Some(&bob_source), "2026-11-02", "99", "income"))
        )
        .await
        .0,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        bob.call("DELETE", &format!("/api/transactions/{transaction}"), None)
            .await
            .0,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        alice
            .call("GET", "/api/summary?month=2026-11", None)
            .await
            .1["spending_cents"],
        "1000"
    );
}

#[tokio::test]
async fn invalid_inputs_and_retired_balance_routes_do_not_mutate_data() {
    let h = Harness::new().await;
    let mut alice = h.client("alice").await;
    for body in [
        entry(None, "2026-11-02", "0", "expense"),
        entry(None, "2026-11-02", "-1", "expense"),
        entry(None, "2026-11-02", "12.345", "income"),
        entry(None, "2026-11-31", "1", "expense"),
        entry(None, "2026-11-01", "92233720368547758.08", "income"),
    ] {
        assert_eq!(
            alice.call("POST", "/api/transactions", Some(body)).await.0,
            StatusCode::UNPROCESSABLE_ENTITY
        );
    }
    for (method, path, body) in [
        ("GET", "/api/accounts", None),
        ("POST", "/api/accounts", Some(json!({"name":"Retired"}))),
        ("GET", "/api/accounts/anything/snapshots", None),
        (
            "POST",
            "/api/accounts/anything/snapshots",
            Some(json!({"as_of_date":"2026-11-01","balance":"100"})),
        ),
        ("DELETE", "/api/accounts/anything/snapshots/anything", None),
    ] {
        assert_eq!(
            alice.call(method, path, body).await.0,
            StatusCode::NOT_FOUND
        );
    }
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM transactions")
            .fetch_one(&h.pool)
            .await
            .unwrap(),
        0
    );
}

#[tokio::test]
async fn migration_preserves_sources_entries_and_retired_balance_records() {
    let directory = tempfile::tempdir().unwrap();
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            sqlx::sqlite::SqliteConnectOptions::new()
                .filename(directory.path().join("synthetic.sqlite3"))
                .create_if_missing(true)
                .foreign_keys(true),
        )
        .await
        .unwrap();
    sqlx::raw_sql(include_str!("../migrations/0001_ledger.sql"))
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO users(id,username,password_hash) VALUES('u','synthetic','not-a-login-hash')",
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query("INSERT INTO accounts(id,user_id,name,opening_date,opening_cents) VALUES('a','u','Synthetic N26','2026-11-01',100000)").execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO transactions(id,user_id,account_id,date,description,amount_cents) VALUES('t','u','a','2026-11-03','Synthetic',-1234)").execute(&pool).await.unwrap();
    sqlx::raw_sql(include_str!("../migrations/0002_balance_snapshots.sql"))
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO balance_snapshots(id,account_id,user_id,as_of_date,balance_cents) VALUES('s','a','u','2026-11-03',98766)").execute(&pool).await.unwrap();
    let mut tx = pool.begin().await.unwrap();
    sqlx::raw_sql(include_str!(
        "../migrations/0003_optional_transaction_sources.sql"
    ))
    .execute(&mut *tx)
    .await
    .unwrap();
    tx.commit().await.unwrap();
    assert_eq!(
        sqlx::query_scalar::<_, String>("SELECT name FROM transaction_sources WHERE id='a'")
            .fetch_one(&pool)
            .await
            .unwrap(),
        "Synthetic N26"
    );
    let row: (i64, String, String) = sqlx::query_as(
        "SELECT amount_cents, source_id, description FROM transactions WHERE id='t'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(row, (-1234, "a".into(), "Synthetic".into()));
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT balance_cents FROM balance_snapshots WHERE id='s'")
            .fetch_one(&pool)
            .await
            .unwrap(),
        98766
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT opening_cents FROM legacy_opening_balances WHERE account_id='a'"
        )
        .fetch_one(&pool)
        .await
        .unwrap(),
        100000
    );
    sqlx::query("INSERT INTO transactions(id,user_id,date,description,amount_cents) VALUES('m','u','2026-11-04','Manual',-100)").execute(&pool).await.unwrap();
    assert!(
        sqlx::query("PRAGMA foreign_key_check")
            .fetch_all(&pool)
            .await
            .unwrap()
            .is_empty()
    );
}
