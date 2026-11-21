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
    async fn account(&mut self, opening: &str) -> String {
        let (status, value) = self.call("POST", "/api/accounts", Some(json!({ "name": "N26", "opening_date": "2026-11-01", "opening_balance": opening }))).await;
        assert_eq!(status, StatusCode::CREATED, "{value}");
        value["id"].as_str().unwrap().into()
    }
    async fn entry(&mut self, account: &str, date: &str, amount: &str, kind: &str) -> String {
        let (status, value) = self
            .call(
                "POST",
                "/api/transactions",
                Some(entry(account, date, amount, kind)),
            )
            .await;
        assert_eq!(status, StatusCode::CREATED, "{value}");
        value["id"].as_str().unwrap().into()
    }
}

fn entry(account: &str, date: &str, amount: &str, kind: &str) -> Value {
    json!({ "account_id": account, "date": date, "amount": amount, "kind": kind, "description": "Synthetic entry" })
}

#[tokio::test]
async fn balances_month_boundaries_edits_and_deletes_are_consistent() {
    let h = Harness::new().await;
    let mut alice = h.client("alice").await;
    let account = alice.account("1000.00").await;
    alice.entry(&account, "2026-11-02", "2000", "income").await;
    let expense = alice
        .entry(&account, "2026-11-03", "12,34", "expense")
        .await;
    let (_, accounts) = alice.call("GET", "/api/accounts", None).await;
    assert_eq!(accounts[0]["balance_cents"], "298766");
    let (_, summary) = alice.call("GET", "/api/summary?month=2026-11", None).await;
    assert_eq!(summary["income_cents"], "200000");
    assert_eq!(summary["spending_cents"], "1234");
    assert_eq!(summary["transaction_count"], 2);
    assert_eq!(
        alice
            .call(
                "PUT",
                &format!("/api/transactions/{expense}"),
                Some(entry(&account, "2026-11-03", "13.34", "expense"))
            )
            .await
            .0,
        StatusCode::NO_CONTENT
    );
    let (_, accounts) = alice.call("GET", "/api/accounts", None).await;
    assert_eq!(accounts[0]["balance_cents"], "298666");
    alice.entry(&account, "2026-11-30", "100", "expense").await;
    alice.entry(&account, "2026-12-01", "4000", "income").await;
    let (_, summary) = alice.call("GET", "/api/summary?month=2026-11", None).await;
    assert_eq!(summary["spending_cents"], "11334");
    assert_eq!(summary["income_cents"], "200000");
    let (_, rows) = alice
        .call("GET", "/api/transactions?month=2026-11", None)
        .await;
    assert_eq!(rows.as_array().unwrap().len(), 3);
    assert_eq!(rows[0]["date"], "2026-11-30");
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
        "10000"
    );
}

#[tokio::test]
async fn private_records_and_totals_stay_with_their_owner() {
    let h = Harness::new().await;
    let mut alice = h.client("alice").await;
    let mut bob = h.client("bob").await;
    let account = alice.account("1000").await;
    let transaction = alice.entry(&account, "2026-11-02", "10", "expense").await;
    assert_eq!(bob.call("GET", "/api/accounts", None).await.1, json!([]));
    assert_eq!(
        bob.call("GET", "/api/transactions?month=2026-11", None)
            .await
            .1,
        json!([])
    );
    assert_eq!(
        bob.call("GET", "/api/summary?month=2026-11", None).await.1["income_cents"],
        "0"
    );
    for path in [
        format!("/api/transactions?month=2026-11&account_id={account}"),
        format!("/api/summary?month=2026-11&account_id={account}"),
    ] {
        assert_eq!(bob.call("GET", &path, None).await.0, StatusCode::NOT_FOUND);
    }
    let body = entry(&account, "2026-11-02", "99", "expense");
    assert_eq!(
        bob.call("POST", "/api/transactions", Some(body.clone()))
            .await
            .0,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        bob.call(
            "PUT",
            &format!("/api/transactions/{transaction}"),
            Some(body)
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
    let own = bob.account("0").await;
    assert_eq!(
        bob.call(
            "PUT",
            &format!("/api/transactions/{transaction}"),
            Some(entry(&own, "2026-11-02", "99", "income"))
        )
        .await
        .0,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        alice.call("GET", "/api/accounts", None).await.1[0]["balance_cents"],
        "99000"
    );
}

#[tokio::test]
async fn validation_rejects_bad_input_and_rolls_back_overflows() {
    let h = Harness::new().await;
    let mut alice = h.client("alice").await;
    let account = alice.account("0").await;
    for body in [
        entry(&account, "2026-11-02", "0", "expense"),
        entry(&account, "2026-11-02", "-1", "expense"),
        entry(&account, "2026-11-02", "12.345", "income"),
        entry(&account, "2026-10-31", "1", "expense"),
        entry(&account, "2026-11-31", "1", "expense"),
    ] {
        assert_eq!(
            alice.call("POST", "/api/transactions", Some(body)).await.0,
            StatusCode::UNPROCESSABLE_ENTITY
        );
    }
    assert_eq!(
        alice
            .call("GET", "/api/transactions?month=2026-11", None)
            .await
            .1,
        json!([])
    );
    let huge = alice.account("92233720368547758.07").await;
    assert_eq!(
        alice
            .call(
                "POST",
                "/api/transactions",
                Some(entry(&huge, "2026-11-02", "0.01", "income"))
            )
            .await
            .0,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM transactions")
            .fetch_one(&h.pool)
            .await
            .unwrap(),
        0
    );
    assert_eq!(
        alice
            .call("GET", "/api/summary?month=not-a-month", None)
            .await
            .0,
        StatusCode::UNPROCESSABLE_ENTITY
    );
}

#[tokio::test]
async fn sessions_rotate_persist_expire_and_require_csrf() {
    let h = Harness::new().await;
    let mut anonymous = Client {
        app: h.app.clone(),
        cookie: None,
        csrf: None,
    };
    for path in ["/api/accounts", "/api/transactions", "/api/summary"] {
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
        anonymous.call("GET", "/api/accounts", None).await.0,
        StatusCode::UNAUTHORIZED
    );
}

#[tokio::test]
async fn concurrent_writes_cannot_commit_an_overflow() {
    let h = Harness::new().await;
    let mut alice = h.client("alice").await;
    let account = alice.account("92233720368547758.06").await;
    let mut other_request = alice.clone();
    let first = entry(&account, "2026-11-02", "0.01", "income");
    let second = first.clone();
    let (a, b) = tokio::join!(
        alice.call("POST", "/api/transactions", Some(first)),
        other_request.call("POST", "/api/transactions", Some(second))
    );
    let mut statuses = [a.0.as_u16(), b.0.as_u16()];
    statuses.sort();
    assert_eq!(statuses, [201, 422]);
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM transactions")
            .fetch_one(&h.pool)
            .await
            .unwrap(),
        1
    );
}
