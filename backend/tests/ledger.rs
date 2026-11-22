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
    sqlx::raw_sql(include_str!("../migrations/0004_import_records.sql"))
        .execute(&pool)
        .await
        .unwrap();
    sqlx::raw_sql(include_str!("../migrations/0005_transaction_origin.sql"))
        .execute(&pool)
        .await
        .unwrap();
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM transactions WHERE origin='unknown'")
            .fetch_one(&pool)
            .await
            .unwrap(),
        2
    );
}

const DEMO_CSV: &str = include_str!("../../examples/imports/demo.csv");
const OVERLAP_CSV: &str = include_str!("../../examples/imports/demo-overlap.csv");

fn import_input(source: &str, csv: &str, decisions: Value) -> Value {
    json!({"source_id":source,"csv":csv,"decisions":decisions})
}

async fn import_ok(client: &mut Client, source: &str, csv: &str, decisions: Value) -> Value {
    let (status, result) = client
        .call(
            "POST",
            "/api/imports/commit",
            Some(import_input(source, csv, decisions)),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{result}");
    result
}

#[tokio::test]
async fn imports_preview_repeat_overlap_and_preserve_edits_and_deletions() {
    let h = Harness::new().await;
    let mut alice = h.client("alice").await;
    let source = alice.source("Example import stream").await;
    let (status, preview) = alice
        .call(
            "POST",
            "/api/imports/preview",
            Some(import_input(&source, DEMO_CSV, json!([]))),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{preview}");
    assert_eq!(preview["rows"].as_array().unwrap().len(), 16);
    assert!(
        preview["rows"]
            .as_array()
            .unwrap()
            .iter()
            .all(|r| r["status"] == "new")
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM transactions")
            .fetch_one(&h.pool)
            .await
            .unwrap(),
        0
    );
    assert_eq!(
        import_ok(&mut alice, &source, DEMO_CSV, json!([])).await["added"],
        16
    );
    for (month, spending, net) in [
        ("2026-10", "96024", "143976"),
        ("2026-11", "106379", "133621"),
    ] {
        let summary = alice
            .call("GET", &format!("/api/summary?month={month}"), None)
            .await
            .1;
        assert_eq!(summary["income_cents"], "240000");
        assert_eq!(summary["spending_cents"], spending);
        assert_eq!(summary["net_cents"], net);
        assert_eq!(summary["transaction_count"], 8);
    }
    let repeated = import_ok(&mut alice, &source, DEMO_CSV, json!([])).await;
    assert_eq!(repeated["added"], 0);
    assert_eq!(repeated["known"], 16);
    let overlap = import_ok(&mut alice, &source, OVERLAP_CSV, json!([])).await;
    assert_eq!(overlap["added"], 1);
    assert_eq!(overlap["known"], 3);
    let summary = alice
        .call("GET", "/api/summary?month=2026-11", None)
        .await
        .1;
    assert_eq!(summary["spending_cents"], "107979");
    assert_eq!(summary["net_cents"], "132021");
    // Edits retain provenance; deletion releases the bank row for reimport.
    let rows = alice
        .call("GET", "/api/transactions?month=2026-11", None)
        .await
        .1;
    let edited = rows
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["description"] == "Example cinema")
        .unwrap();
    let edited_id = edited["id"].as_str().unwrap();
    assert_eq!(alice.call("PUT", &format!("/api/transactions/{edited_id}"), Some(json!({"source_id":null,"date":"2026-11-29","description":"My edited note","amount":"18","kind":"expense"}))).await.0, StatusCode::NO_CONTENT);
    let deleted = rows
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["description"] == "Example restaurant")
        .unwrap()["id"]
        .as_str()
        .unwrap();
    assert_eq!(
        alice
            .call("DELETE", &format!("/api/transactions/{deleted}"), None)
            .await
            .0,
        StatusCode::NO_CONTENT
    );
    let changed_file = format!("{}\n", OVERLAP_CSV.replace('\n', "\r\n"));
    let result = import_ok(&mut alice, &source, &changed_file, json!([])).await;
    assert_eq!(result["added"], 1);
    assert_eq!(result["known"], 3);
    assert_eq!(
        alice
            .call("GET", "/api/transactions?month=2026-11", None)
            .await
            .1
            .as_array()
            .unwrap()
            .iter()
            .filter(|r| r["description"] == "My edited note")
            .count(),
        1
    );
}

#[tokio::test]
async fn import_review_matching_conflicts_and_invalid_rows_are_atomic() {
    let h = Harness::new().await;
    let mut alice = h.client("alice").await;
    let source = alice.source("Review example").await;
    let manual = alice.entry(None, "2026-11-01", "3.50", "expense").await;
    let csv = "date,description,amount,external_id\n2026-11-01,Synthetic entry,-3.50,\n";
    let input = import_input(&source, csv, json!([]));
    let preview = alice
        .call("POST", "/api/imports/preview", Some(input.clone()))
        .await
        .1;
    assert_eq!(preview["rows"][0]["status"], "review");
    assert_eq!(preview["rows"][0]["candidates"][0]["id"], manual);
    assert_eq!(
        alice
            .call("POST", "/api/imports/commit", Some(input))
            .await
            .0,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    let matched = import_ok(
        &mut alice,
        &source,
        csv,
        json!([{"record":2,"action":"match","transaction_id":manual}]),
    )
    .await;
    assert_eq!(matched["matched"], 1);
    let rows = alice
        .call("GET", "/api/transactions?month=2026-11", None)
        .await
        .1;
    assert_eq!(rows.as_array().unwrap().len(), 1);
    assert!(rows[0]["source_id"].is_null());
    assert_eq!(
        import_ok(&mut alice, &source, csv, json!([])).await["known"],
        1
    );
    // An overlapping ID-less export requires a decision, even after editing the entry.
    let moved_source = alice.source("Reassigned source").await;
    assert_eq!(alice.call("PUT", &format!("/api/transactions/{manual}"), Some(json!({
        "source_id":moved_source,"date":"2026-11-03","description":"Edited after import","amount":"4.50","kind":"expense"
    }))).await.0, StatusCode::NO_CONTENT);
    let overlap = format!("{csv}2026-11-02,New expense,-2.00,stable-new\n");
    let preview = alice
        .call(
            "POST",
            "/api/imports/preview",
            Some(import_input(&source, &overlap, json!([]))),
        )
        .await
        .1;
    assert_eq!(preview["rows"][0]["status"], "review");
    assert_eq!(preview["rows"][0]["candidates"][0]["id"], manual);
    assert_eq!(
        import_ok(
            &mut alice,
            &source,
            &overlap,
            json!([{"record":2,"action":"keep"}])
        )
        .await["added"],
        2
    );
    let conflict = "date,description,amount,external_id\n2026-11-03,First row,-1.00,first\n2026-11-02,New expense,-3.00,stable-new\n";
    assert_eq!(
        alice
            .call(
                "POST",
                "/api/imports/commit",
                Some(import_input(&source, conflict, json!([])))
            )
            .await
            .0,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT count(*) FROM transactions WHERE description='First row'"
        )
        .fetch_one(&h.pool)
        .await
        .unwrap(),
        0
    );
    assert_eq!(
        import_ok(
            &mut alice,
            &source,
            conflict,
            json!([{"record":3,"action":"skip"}])
        )
        .await["skipped"],
        1
    );
    let invalid = "date,description,amount,external_id\n2026-11-03,Must roll back,-2.00,rollback\n2026-02-30,Bad,2.999,bad\n";
    assert_eq!(
        alice
            .call(
                "POST",
                "/api/imports/commit",
                Some(import_input(&source, invalid, json!([])))
            )
            .await
            .0,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT count(*) FROM transactions WHERE description='Must roll back'"
        )
        .fetch_one(&h.pool)
        .await
        .unwrap(),
        0
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT count(*) FROM import_records WHERE external_id='rollback'"
        )
        .fetch_one(&h.pool)
        .await
        .unwrap(),
        0
    );
    // Repeated external IDs within one file cannot silently merge.
    let repeated_ids =
        "date,description,amount,external_id\n2026-11-03,One,-1,repeat\n2026-11-03,Two,-2,repeat\n";
    let preview = alice
        .call(
            "POST",
            "/api/imports/preview",
            Some(import_input(&source, repeated_ids, json!([]))),
        )
        .await
        .1;
    assert_eq!(preview["rows"][1]["status"], "conflict");
    assert_eq!(
        import_ok(
            &mut alice,
            &source,
            repeated_ids,
            json!([{"record":3,"action":"skip"}])
        )
        .await["added"],
        1
    );
}

#[tokio::test]
async fn imports_enforce_source_match_ownership_csrf_limits_and_concurrent_idempotence() {
    let h = Harness::new().await;
    let mut alice = h.client("alice").await;
    let mut bob = h.client("bob").await;
    let source = alice.source("Private synthetic stream").await;
    let bob_source = bob.source("Bob stream").await;
    for endpoint in ["preview", "commit"] {
        let path = format!("/api/imports/{endpoint}");
        assert_eq!(
            bob.call(
                "POST",
                &path,
                Some(import_input(&source, DEMO_CSV, json!([])))
            )
            .await
            .0,
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            alice
                .send(
                    "POST",
                    &path,
                    Some(import_input(&source, DEMO_CSV, json!([]))),
                    false,
                    ORIGIN
                )
                .await
                .0,
            StatusCode::FORBIDDEN
        );
    }
    let foreign = bob.entry(None, "2026-11-01", "3.50", "expense").await;
    let csv = "date,description,amount,external_id\n2026-11-01,Synthetic entry,-3.50,match\n";
    assert_eq!(
        alice
            .call(
                "POST",
                "/api/imports/commit",
                Some(import_input(
                    &source,
                    csv,
                    json!([{"record":2,"action":"match","transaction_id":foreign}])
                ))
            )
            .await
            .0,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM import_records")
            .fetch_one(&h.pool)
            .await
            .unwrap(),
        0
    );
    let mut parallel = alice.clone();
    let a = import_input(&source, DEMO_CSV, json!([]));
    let b = a.clone();
    let (first, second) = tokio::join!(
        alice.call("POST", "/api/imports/commit", Some(a)),
        parallel.call("POST", "/api/imports/commit", Some(b))
    );
    assert_eq!(first.0, StatusCode::OK, "{}", first.1);
    assert_eq!(second.0, StatusCode::OK, "{}", second.1);
    assert_eq!(
        first.1["added"].as_u64().unwrap() + second.1["added"].as_u64().unwrap(),
        16
    );
    let bob_preview = bob
        .call(
            "POST",
            "/api/imports/preview",
            Some(import_input(&bob_source, DEMO_CSV, json!([]))),
        )
        .await
        .1;
    assert!(
        bob_preview["rows"]
            .as_array()
            .unwrap()
            .iter()
            .all(|r| r["status"] == "new")
    );
    let another_source = alice.source("Separate stream").await;
    assert_eq!(
        import_ok(&mut alice, &another_source, DEMO_CSV, json!([])).await["added"],
        16
    );
    // Valid JSON larger than the ledger's 16 KiB limit reaches the importer.
    let many_rows = format!(
        "date,description,amount,external_id\n{}",
        (0..600)
            .map(|i| format!("2026-11-01,Example purchase {i},-1,id-{i}\n"))
            .collect::<String>()
    );
    assert_eq!(
        alice
            .call(
                "POST",
                "/api/imports/preview",
                Some(import_input(&source, &many_rows, json!([])))
            )
            .await
            .0,
        StatusCode::OK
    );
    let oversized = "x".repeat(256 * 1024 + 1);
    assert_eq!(
        alice
            .call(
                "POST",
                "/api/imports/preview",
                Some(import_input(&source, &oversized, json!([])))
            )
            .await
            .0,
        StatusCode::UNPROCESSABLE_ENTITY
    );
}

#[tokio::test]
async fn mixed_id_imports_review_missing_ids_regardless_of_row_order() {
    let h = Harness::new().await;
    let mut alice = h.client("alice").await;
    for ids in [
        vec!["", "bank-1"],
        vec!["bank-1", ""],
        vec!["", "bank-1", "bank-2"],
        vec!["bank-2", "", "bank-1"],
    ] {
        let source = alice.source("Mixed-ID synthetic stream").await;
        let csv = format!(
            "date,description,amount,external_id\n{}",
            ids.iter()
                .map(|id| format!("2026-11-01,Mixed-ID coffee,-3.50,{id}\n"))
                .collect::<String>()
        );
        let (status, preview) = alice
            .call(
                "POST",
                "/api/imports/preview",
                Some(import_input(&source, &csv, json!([]))),
            )
            .await;
        assert_eq!(status, StatusCode::OK, "{preview}");
        for (row, id) in preview["rows"].as_array().unwrap().iter().zip(&ids) {
            assert_eq!(
                row["status"],
                if id.is_empty() { "review" } else { "new" },
                "unexpected classification for {ids:?}: {preview}"
            );
        }
        assert_eq!(
            alice
                .call(
                    "POST",
                    "/api/imports/commit",
                    Some(import_input(&source, &csv, json!([]))),
                )
                .await
                .0,
            StatusCode::UNPROCESSABLE_ENTITY
        );
        for table in ["transactions", "import_records"] {
            assert_eq!(
                sqlx::query_scalar::<_, i64>(&format!(
                    "SELECT count(*) FROM {table} WHERE source_id=?"
                ))
                .bind(&source)
                .fetch_one(&h.pool)
                .await
                .unwrap(),
                0,
                "failed import must leave no {table}"
            );
        }
        // Distinct stable IDs remain eligible even alongside an ambiguous ID-less row.
        let stable_records: Vec<usize> = ids
            .iter()
            .enumerate()
            .filter(|(_, id)| !id.is_empty())
            .map(|(index, _)| index + 2)
            .collect();
        let (status, result) = alice
            .call(
                "POST",
                "/api/imports/commit",
                Some(json!({"source_id":source,"csv":csv,"selected_records":stable_records})),
            )
            .await;
        assert_eq!(status, StatusCode::OK, "{result}");
        assert_eq!(result["added"], ids.len() - 1);
        assert_eq!(result["excluded"], 1);

        let blank_record = ids.iter().position(|id| id.is_empty()).unwrap() + 2;
        let kept = import_ok(
            &mut alice,
            &source,
            &csv,
            json!([{"record":blank_record,"action":"keep"}]),
        )
        .await;
        assert_eq!(kept["added"], 1);
        assert_eq!(kept["known"], ids.len() - 1);
        let repeated = import_ok(&mut alice, &source, &csv, json!([])).await;
        assert_eq!(repeated["added"], 0);
        assert_eq!(repeated["known"], ids.len());
    }
}

#[tokio::test]
async fn idless_imports_review_identical_rows_and_allow_deleted_rows_again() {
    let h = Harness::new().await;
    let mut alice = h.client("alice").await;
    let source = alice.source("ID-less synthetic stream").await;
    let one = "date,description,amount,external_id\n2026-11-01,ID-less coffee,-3.50,\n";
    let two = format!("{one}2026-11-01,ID-less coffee,-3.50,\n");
    let preview = alice
        .call(
            "POST",
            "/api/imports/preview",
            Some(import_input(&source, &two, json!([]))),
        )
        .await
        .1;
    assert_eq!(preview["rows"][0]["status"], "new");
    assert_eq!(preview["rows"][1]["status"], "review");
    assert_eq!(
        alice
            .call(
                "POST",
                "/api/imports/commit",
                Some(import_input(&source, &two, json!([])))
            )
            .await
            .0,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM transactions")
            .fetch_one(&h.pool)
            .await
            .unwrap(),
        0
    );
    assert_eq!(
        import_ok(
            &mut alice,
            &source,
            &two,
            json!([{"record":3,"action":"keep"}])
        )
        .await["added"],
        2
    );
    let rows = alice
        .call("GET", "/api/transactions?month=2026-11", None)
        .await
        .1;
    for row in rows.as_array().unwrap() {
        assert_eq!(
            alice
                .call(
                    "DELETE",
                    &format!("/api/transactions/{}", row["id"].as_str().unwrap()),
                    None
                )
                .await
                .0,
            StatusCode::NO_CONTENT
        );
    }
    let preview = alice
        .call(
            "POST",
            "/api/imports/preview",
            Some(import_input(&source, one, json!([]))),
        )
        .await
        .1;
    assert_eq!(preview["rows"][0]["status"], "new");
    assert_eq!(
        import_ok(&mut alice, &source, one, json!([])).await["added"],
        1
    );
}

#[tokio::test]
async fn selected_imports_leave_exclusions_available_and_recheck_selected_rows() {
    let h = Harness::new().await;
    let mut alice = h.client("alice").await;
    let source = alice.source("Selected synthetic stream").await;
    let csv = "date,description,amount,external_id\n2026-11-01,One,-1.25,one\n2026-11-02,Two,-2.50,two\n2026-02-30,Invalid,1.234,bad\n";
    let input = |records: Value| json!({"source_id":source,"csv":csv,"selected_records":records,"decisions":[]});
    let (status, first) = alice
        .call("POST", "/api/imports/commit", Some(input(json!([2]))))
        .await;
    assert_eq!(status, StatusCode::OK, "{first}");
    assert_eq!(first["added"], 1);
    assert_eq!(first["excluded"], 2);
    assert_eq!(first["spending_cents"], "125");
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM import_records")
            .fetch_one(&h.pool)
            .await
            .unwrap(),
        1
    );
    let second = alice
        .call("POST", "/api/imports/commit", Some(input(json!([3]))))
        .await;
    assert_eq!(second.0, StatusCode::OK, "{}", second.1);
    assert_eq!(second.1["added"], 1);
    assert_eq!(second.1["spending_cents"], "250");
    assert_eq!(second.1["known"], 1);
    assert_eq!(
        alice
            .call("POST", "/api/imports/commit", Some(input(json!([2, 3]))))
            .await
            .1["added"],
        0
    );
    for invalid in [json!([4]), json!([2, 2]), json!([999])] {
        assert_eq!(
            alice
                .call("POST", "/api/imports/commit", Some(input(invalid)))
                .await
                .0,
            StatusCode::UNPROCESSABLE_ENTITY
        );
    }
    let mut bad = input(json!([]));
    bad["decisions"] = json!([{"record":2,"action":"keep"}]);
    assert_eq!(
        alice.call("POST", "/api/imports/commit", Some(bad)).await.0,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    let manual = alice.entry(None, "2026-11-03", "3", "expense").await;
    let ambiguous = "date,description,amount,external_id\n2026-11-03,Synthetic entry,-3,manual-match\n2026-11-04,Another,-4,new\n";
    let partial = json!({"source_id":source,"csv":ambiguous,"selected_records":[3],"decisions":[]});
    assert_eq!(
        alice
            .call("POST", "/api/imports/commit", Some(partial))
            .await
            .1["added"],
        1
    );
    let matched = json!({"source_id":source,"csv":ambiguous,"selected_records":[2],"decisions":[{"record":2,"action":"match","transaction_id":manual}]});
    assert_eq!(
        alice
            .call("POST", "/api/imports/commit", Some(matched))
            .await
            .1["matched"],
        1
    );
    let entries = alice
        .call("GET", "/api/transactions?month=2026-11", None)
        .await
        .1;
    let entry = entries
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["id"] == manual)
        .unwrap();
    assert_eq!(entry["origin"], "manual");
    assert_eq!(entry["csv_linked"], true);
    assert!(
        entries
            .as_array()
            .unwrap()
            .iter()
            .filter(|r| r["id"] != manual)
            .all(|r| r["origin"] == "csv")
    );
}

#[tokio::test]
async fn inline_fields_provenance_and_bulk_deletion_enforce_ownership_and_atomicity() {
    let h = Harness::new().await;
    let mut alice = h.client("alice").await;
    let mut bob = h.client("bob").await;
    let source = alice.source("Owned stream").await;
    let foreign_source = bob.source("Foreign stream").await;
    let manual = alice.entry(None, "2026-11-01", "1.25", "expense").await;
    let foreign = bob.entry(None, "2026-11-01", "2.50", "expense").await;
    let path = format!("/api/transactions/{manual}");
    for input in [
        json!({"field":"description","value":"Edited"}),
        json!({"field":"amount","value":"2.34","kind":"income"}),
        json!({"field":"date","value":"2026-11-02"}),
        json!({"field":"source","value":source}),
    ] {
        assert_eq!(
            alice.call("PATCH", &path, Some(input)).await.0,
            StatusCode::NO_CONTENT
        );
    }
    let row = alice
        .call("GET", "/api/transactions?month=2026-11", None)
        .await
        .1[0]
        .clone();
    assert_eq!(row["description"], "Edited");
    assert_eq!(row["amount_cents"], "234");
    assert_eq!(row["date"], "2026-11-02");
    assert_eq!(row["source_id"], source);
    assert_eq!(row["origin"], "manual");
    assert_eq!(row["csv_linked"], false);
    assert_eq!(
        bob.call(
            "PATCH",
            &path,
            Some(json!({"field":"description","value":"Forbidden"}))
        )
        .await
        .0,
        StatusCode::NOT_FOUND
    );
    for bad in [
        json!({"field":"source","value":foreign_source}),
        json!({"field":"amount","value":"1.234","kind":"expense"}),
        json!({"field":"date","value":"2026-02-30"}),
        json!({"field":"origin","value":"csv"}),
    ] {
        assert!(!alice.call("PATCH", &path, Some(bad)).await.0.is_success());
    }
    assert_eq!(
        alice
            .send(
                "PATCH",
                &path,
                Some(json!({"field":"description","value":"No CSRF"})),
                false,
                ORIGIN
            )
            .await
            .0,
        StatusCode::FORBIDDEN
    );
    import_ok(&mut alice, &source, DEMO_CSV, json!([])).await;
    let rows = alice
        .call("GET", "/api/transactions?month=2026-11", None)
        .await
        .1;
    let imported = rows
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["origin"] == "csv")
        .unwrap()["id"]
        .as_str()
        .unwrap();
    let evidence_path = format!("/api/transactions/{imported}/provenance");
    let evidence = alice.call("GET", &evidence_path, None).await;
    assert_eq!(evidence.0, StatusCode::OK);
    assert_eq!(evidence.1[0]["source_name"], "Owned stream");
    assert_eq!(
        bob.call("GET", &evidence_path, None).await.0,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        alice
            .call(
                "POST",
                "/api/transactions/bulk-delete",
                Some(json!({"ids":[manual,foreign]}))
            )
            .await
            .0,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        alice
            .call("GET", "/api/transactions?month=2026-11", None)
            .await
            .1
            .as_array()
            .unwrap()
            .len(),
        9
    );
    for ids in [json!([]), json!([manual, manual]), json!(["missing"])] {
        assert!(
            !alice
                .call(
                    "POST",
                    "/api/transactions/bulk-delete",
                    Some(json!({"ids":ids}))
                )
                .await
                .0
                .is_success()
        );
    }
    assert_eq!(
        alice
            .send(
                "POST",
                "/api/transactions/bulk-delete",
                Some(json!({"ids":[manual]})),
                false,
                ORIGIN
            )
            .await
            .0,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        alice
            .call(
                "POST",
                "/api/transactions/bulk-delete",
                Some(json!({"ids":[manual,imported]}))
            )
            .await
            .0,
        StatusCode::NO_CONTENT
    );
    assert_eq!(
        import_ok(&mut alice, &source, DEMO_CSV, json!([])).await["added"],
        1
    );
    assert_eq!(
        bob.call("GET", "/api/transactions?month=2026-11", None)
            .await
            .1
            .as_array()
            .unwrap()
            .len(),
        1
    );
}

#[tokio::test]
async fn deleting_all_imported_entries_allows_full_reimport_and_migration_clears_old_tombstones() {
    let h = Harness::new().await;
    let mut alice = h.client("alice").await;
    let source = alice.source("Synthetic reimport").await;
    import_ok(&mut alice, &source, DEMO_CSV, json!([])).await;
    let ids: Vec<String> = sqlx::query_scalar("SELECT id FROM transactions")
        .fetch_all(&h.pool)
        .await
        .unwrap();
    assert_eq!(
        alice
            .call(
                "POST",
                "/api/transactions/bulk-delete",
                Some(json!({"ids":ids}))
            )
            .await
            .0,
        StatusCode::NO_CONTENT
    );
    assert_eq!(
        import_ok(&mut alice, &source, DEMO_CSV, json!([])).await["added"],
        16
    );
    // Recreate the old deletion behavior in this wholly synthetic temporary database.
    sqlx::query("DROP TRIGGER forget_deleted_import_evidence")
        .execute(&h.pool)
        .await
        .unwrap();
    sqlx::query("DELETE FROM transactions")
        .execute(&h.pool)
        .await
        .unwrap();
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT count(*) FROM import_records WHERE transaction_id IS NULL"
        )
        .fetch_one(&h.pool)
        .await
        .unwrap(),
        16
    );
    sqlx::raw_sql(include_str!(
        "../migrations/0006_reimport_deleted_entries.sql"
    ))
    .execute(&h.pool)
    .await
    .unwrap();
    assert_eq!(
        import_ok(&mut alice, &source, DEMO_CSV, json!([])).await["added"],
        16
    );
}

#[tokio::test]
async fn workspace_pagination_totals_dates_search_and_dashboard_are_owned() {
    let h = Harness::new().await;
    let mut alice = h.client("alice").await;
    let mut bob = h.client("bob").await;
    let source = alice.source("Report fixture").await;
    import_ok(&mut alice, &source, DEMO_CSV, json!([])).await;
    bob.entry(None, "2026-11-01", "9999", "expense").await;
    let (status, first) = alice.call("GET", "/api/entries?page_size=5", None).await;
    assert_eq!(status, StatusCode::OK, "{first}");
    assert_eq!(first["entries"].as_array().unwrap().len(), 5);
    assert_eq!(first["pages"], 4);
    assert_eq!(first["totals"]["transaction_count"], 16);
    assert_eq!(first["totals"]["spending_cents"], "202403");
    let second = alice
        .call("GET", "/api/entries?page_size=5&page=2", None)
        .await
        .1;
    assert!(second["entries"].as_array().unwrap().iter().all(|r| {
        !first["entries"]
            .as_array()
            .unwrap()
            .iter()
            .any(|a| a["id"] == r["id"])
    }));
    let november = alice
        .call("GET", "/api/entries?start=2026-11-01&end=2026-11-30", None)
        .await
        .1;
    assert_eq!(november["totals"]["spending_cents"], "106379");
    let day = alice
        .call("GET", "/api/entries?start=2026-11-01&end=2026-11-01", None)
        .await
        .1;
    assert_eq!(day["totals"]["transaction_count"], 1);
    assert!(
        day["entries"]
            .as_array()
            .unwrap()
            .iter()
            .all(|r| r["date"] == "2026-11-01")
    );
    let coffee = alice.call("GET", "/api/entries?q=COFFEE", None).await.1;
    assert!(coffee["totals"]["transaction_count"].as_u64().unwrap() > 0);
    assert!(coffee["entries"].as_array().unwrap().iter().all(|r| {
        r["description"]
            .as_str()
            .unwrap()
            .to_lowercase()
            .contains("coffee")
    }));
    let dashboard = alice
        .call("GET", "/api/dashboard?month=2026-11", None)
        .await
        .1;
    assert_eq!(dashboard["monthly"]["spending_cents"], "106379");
    assert_eq!(dashboard["yearly"]["spending_cents"], "202403");
    assert_eq!(dashboard["months"].as_array().unwrap().len(), 12);
    assert_eq!(dashboard["latest"].as_array().unwrap().len(), 5);
    assert_eq!(
        bob.call("GET", "/api/entries", None).await.1["totals"]["transaction_count"],
        1
    );
    for path in [
        "/api/entries?start=2026-02-30",
        "/api/entries?start=2026-12-01&end=2026-01-01",
        "/api/entries?page=0",
        "/api/entries?page_size=101",
        "/api/dashboard?month=2026-13",
    ] {
        assert_eq!(
            alice.call("GET", path, None).await.0,
            StatusCode::UNPROCESSABLE_ENTITY,
            "{path}"
        );
    }
    assert_eq!(
        alice
            .call("GET", "/api/dashboard?month=9999-12", None)
            .await
            .0,
        StatusCode::OK
    );
}

#[tokio::test]
async fn import_history_removal_preserves_edits_matches_and_rechecks_overlaps() {
    let h = Harness::new().await;
    let mut alice = h.client("alice").await;
    let mut bob = h.client("bob").await;
    let source = alice.source("Batch fixture").await;
    let manual = alice.entry(None, "2026-11-01", "3.50", "expense").await;
    let csv = "date,description,amount,external_id\n2026-11-01,Synthetic entry,-3.50,manual\n2026-10-01,Unchanged,-1,one\n2026-11-02,Edited,-2,two\n2026-11-03,Shared,-3,three\n2026-11-04,Deleted,-4,four\n";
    let input = json!({"source_id":source,"csv":csv,"filename":"/fake/path/batch.csv","decisions":[{"record":2,"action":"match","transaction_id":manual}]});
    assert_eq!(
        alice
            .call("POST", "/api/imports/commit", Some(input))
            .await
            .0,
        StatusCode::OK
    );
    let list = alice.call("GET", "/api/imports/history", None).await.1;
    let batch = &list["batches"][0];
    assert_eq!(batch["filename"], "batch.csv");
    assert_eq!(batch["added"], 4);
    assert_eq!(batch["matched"], 1);
    let id = batch["id"].as_str().unwrap();
    let path = format!("/api/imports/history/{id}");
    let removal = format!("{path}/remove");
    assert_eq!(bob.call("GET", &path, None).await.0, StatusCode::NOT_FOUND);
    assert_eq!(
        bob.call("GET", "/api/imports/history", None).await.1["batches"],
        json!([])
    );
    assert_eq!(
        bob.call("POST", &removal, Some(json!({"token":"no"})))
            .await
            .0,
        StatusCode::NOT_FOUND
    );
    let original = alice.call("GET", &path, None).await.1;
    assert_eq!(original["removable"], 4);
    let ids:Vec<(String,String)>=sqlx::query_as("SELECT id,description FROM transactions WHERE user_id=(SELECT id FROM users WHERE username='alice')").fetch_all(&h.pool).await.unwrap();
    let edited = &ids.iter().find(|(_, desc)| desc == "Edited").unwrap().0;
    let deleted = &ids.iter().find(|(_, desc)| desc == "Deleted").unwrap().0;
    alice
        .call(
            "PATCH",
            &format!("/api/transactions/{edited}"),
            Some(json!({"field":"description","value":"User revision"})),
        )
        .await;
    alice
        .call("DELETE", &format!("/api/transactions/{deleted}"), None)
        .await;
    let overlap = "date,description,amount,external_id\n2026-11-03,Shared,-3,three\n";
    import_ok(&mut alice, &source, overlap, json!([])).await;
    assert_eq!(
        alice
            .call("POST", &removal, Some(json!({"token":original["token"]})))
            .await
            .0,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    let current = alice.call("GET", &path, None).await.1;
    assert_eq!(current["removable_entries"].as_array().unwrap().len(), 2);
    assert_eq!(current["edited_entries"][0]["description"], "User revision");
    assert_eq!(current["edited_entries"][0]["id"], *edited);
    assert!(
        current["removable_entries"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entry| entry["description"] == "Shared" && entry["overlapping"] == true)
    );
    assert_eq!(current["removable"], 2);
    assert_eq!(current["edited"], 1);
    assert_eq!(current["overlapping"], 1);
    assert_eq!(current["already_deleted"], 1);
    assert_eq!(
        alice
            .send(
                "POST",
                &removal,
                Some(json!({"token":current["token"]})),
                false,
                ORIGIN
            )
            .await
            .0,
        StatusCode::FORBIDDEN
    );
    let result = alice
        .call("POST", &removal, Some(json!({"token":current["token"]})))
        .await;
    assert_eq!(result.0, StatusCode::OK, "{}", result.1);
    assert_eq!(result.1["removed"], 2);
    let current = alice.call("GET", &path, None).await.1;
    assert_eq!(current["removable"], 0);
    assert_eq!(current["batch"]["removed"], 2);
    assert_eq!(
        alice
            .call(
                "POST",
                &removal,
                Some(json!({"token":current["token"],"selected_edited_ids":[edited]}))
            )
            .await
            .1["removed"],
        1
    );
    let remaining = alice.call("GET", "/api/entries", None).await.1;
    assert_eq!(remaining["totals"]["transaction_count"], 1);
    assert!(
        remaining["entries"]
            .as_array()
            .unwrap()
            .iter()
            .any(|r| r["id"] == manual)
    );
    let preview = alice
        .call(
            "POST",
            "/api/imports/preview",
            Some(import_input(&source, csv, json!([]))),
        )
        .await
        .1;
    assert_eq!(preview["rows"][1]["status"], "new");
    assert_eq!(preview["rows"][2]["status"], "new");
    assert_eq!(preview["rows"][3]["status"], "new");
}

#[tokio::test]
async fn history_records_are_atomic_and_idless_repeats_preserve_creator_and_edits() {
    let h = Harness::new().await;
    let mut alice = h.client("alice").await;
    let source = alice.source("History regression").await;
    let csv = "date,description,amount,external_id\n2026-11-01,ID-less purchase,-1.25,\n";
    import_ok(&mut alice, &source, csv, json!([])).await;
    let batch = alice.call("GET", "/api/imports/history", None).await.1["batches"][0]["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let id = alice.call("GET", "/api/entries", None).await.1["entries"][0]["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let path = format!("/api/imports/history/{batch}");
    let token = alice.call("GET", &path, None).await.1["token"].clone();
    // Identical saves are not edits; changing and reverting still is an edit.
    alice
        .call(
            "PATCH",
            &format!("/api/transactions/{id}"),
            Some(json!({"field":"description","value":"ID-less purchase"})),
        )
        .await;
    assert_eq!(alice.call("GET", &path, None).await.1["token"], token);
    for description in ["Changed", "ID-less purchase"] {
        alice
            .call(
                "PATCH",
                &format!("/api/transactions/{id}"),
                Some(json!({"field":"description","value":description})),
            )
            .await;
    }
    assert_eq!(alice.call("GET", &path, None).await.1["edited"], 1);
    import_ok(&mut alice, &source, csv, json!([])).await;
    let review = alice.call("GET", &path, None).await.1;
    assert_eq!(review["overlapping"], 1);
    assert_eq!(review["edited"], 1);
    assert_eq!(
        alice
            .call(
                "POST",
                &format!("{path}/remove"),
                Some(json!({"token":review["token"]}))
            )
            .await
            .1["removed"],
        0
    );
    let invalid = "date,description,amount,external_id\n2026-11-02,Valid before failure,-2,ok\n2026-02-30,Invalid,-3,bad\n";
    assert_eq!(
        alice
            .call(
                "POST",
                "/api/imports/commit",
                Some(import_input(&source, invalid, json!([])))
            )
            .await
            .0,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    assert_eq!(
        alice.call("GET", "/api/imports/history", None).await.1["batches"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert_eq!(
        alice.call("GET", "/api/entries", None).await.1["totals"]["transaction_count"],
        1
    );
    let reviewed = alice.call("GET", &path, None).await.1;
    assert_eq!(
        alice
            .call(
                "POST",
                &format!("{path}/remove"),
                Some(json!({"token":reviewed["token"],"selected_edited_ids":[id]}))
            )
            .await
            .1["removed"],
        1
    );
}

#[tokio::test]
async fn import_removal_only_includes_individually_selected_edited_entries() {
    let h = Harness::new().await;
    let mut alice = h.client("alice").await;
    let mut bob = h.client("bob").await;
    let source = alice.source("Individual edit fixture").await;
    let csv = "date,description,amount,external_id\n2026-11-01,First edited,-1,one\n2026-11-02,Second edited,-2,two\n2026-11-03,Third edited,-3,three\n2026-11-04,Unchanged,-4,four\n2026-11-05,Shared,-5,five\n";
    import_ok(&mut alice, &source, csv, json!([])).await;
    let batch = alice.call("GET", "/api/imports/history", None).await.1["batches"][0]["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let path = format!("/api/imports/history/{batch}");
    let removal = format!("{path}/remove");
    let rows = alice.call("GET", "/api/entries", None).await.1;
    let id_for = |description: &str| {
        rows["entries"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["description"] == description)
            .unwrap()["id"]
            .as_str()
            .unwrap()
            .to_owned()
    };
    let first = id_for("First edited");
    let second = id_for("Second edited");
    let third = id_for("Third edited");
    let unchanged = id_for("Unchanged");
    let shared = id_for("Shared");
    for id in [&first, &second, &third] {
        assert_eq!(
            alice
                .call(
                    "PATCH",
                    &format!("/api/transactions/{id}"),
                    Some(json!({"field":"amount","value":"9.99","kind":"expense"}))
                )
                .await
                .0,
            StatusCode::NO_CONTENT
        );
    }
    import_ok(
        &mut alice,
        &source,
        "date,description,amount,external_id\n2026-11-05,Shared,-5,five\n",
        json!([]),
    )
    .await;
    let foreign = bob.entry(None, "2026-11-01", "1", "expense").await;
    let manual = alice.entry(None, "2026-11-01", "1", "expense").await;
    let review = alice.call("GET", &path, None).await.1;
    assert_eq!(review["edited_entries"].as_array().unwrap().len(), 3);
    assert_eq!(review["edited_entries"][0]["id"], third);
    assert_eq!(review["edited_entries"][0]["amount_cents"], "-999");
    assert!(
        review["removable_entries"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entry| entry["id"] == shared && entry["overlapping"] == true)
    );
    for ids in [
        json!([first, foreign]),
        json!([first, manual]),
        json!([first, shared]),
        json!([first, unchanged]),
        json!([first, first]),
        json!(["missing"]),
    ] {
        assert_eq!(
            alice
                .call(
                    "POST",
                    &removal,
                    Some(json!({"token":review["token"],"selected_edited_ids":ids}))
                )
                .await
                .0,
            StatusCode::UNPROCESSABLE_ENTITY
        );
        assert_eq!(
            alice.call("GET", &path, None).await.1["removable"],
            2,
            "invalid selection must not delete unchanged entries"
        );
    }
    let large_selection: Vec<_> = (0..1000)
        .map(|n| format!("00000000-0000-0000-0000-{n:012}"))
        .collect();
    assert_eq!(
        alice
            .call(
                "POST",
                &removal,
                Some(json!({"token":review["token"],"selected_edited_ids":large_selection}))
            )
            .await
            .0,
        StatusCode::UNPROCESSABLE_ENTITY,
        "valid-sized batches reach selection validation, rather than a 16 KiB body rejection"
    );
    let result = alice
        .call(
            "POST",
            &removal,
            Some(json!({"token":review["token"],"selected_edited_ids":[first,third]})),
        )
        .await;
    assert_eq!(result.0, StatusCode::OK, "{}", result.1);
    assert_eq!(result.1["removed"], 4);
    let review = alice.call("GET", &path, None).await.1;
    assert_eq!(review["removable_entries"], json!([]));
    assert_eq!(review["edited_entries"].as_array().unwrap().len(), 1);
    assert_eq!(review["edited_entries"][0]["id"], second);
    assert_eq!(review["overlapping"], 0);
    assert_eq!(review["already_deleted"], 4);
    assert_eq!(
        alice.call("GET", "/api/entries", None).await.1["totals"]["transaction_count"],
        2
    );
}

#[tokio::test]
async fn dense_overlaps_keep_one_creator_and_do_not_block_batch_removal() {
    let h = Harness::new().await;
    let mut alice = h.client("alice").await;
    let source = alice.source("Dense overlapping synthetic exports").await;
    let csv = |ids: &[u32]| {
        format!(
            "date,description,amount,external_id\n{}",
            ids.iter()
                .map(|id| format!("2026-11-01,Purchase {id},-1,id-{id}\n"))
                .collect::<String>()
        )
    };
    let first = csv(&[1, 2, 3]);
    import_ok(&mut alice, &source, &first, json!([])).await;
    let creator: String = sqlx::query_scalar("SELECT id FROM import_batches WHERE source_id=?")
        .bind(&source)
        .fetch_one(&h.pool)
        .await
        .unwrap();
    import_ok(&mut alice, &source, &csv(&[2, 3, 4]), json!([])).await;
    let later: String =
        sqlx::query_scalar("SELECT id FROM import_batches WHERE source_id=? AND added=1")
            .bind(&source)
            .fetch_one(&h.pool)
            .await
            .unwrap();
    for ids in [&[4, 3, 2, 1][..], &[2, 1, 4, 3][..]] {
        import_ok(&mut alice, &source, &csv(ids), json!([])).await;
    }
    let path = format!("/api/imports/history/{creator}");
    let review = alice.call("GET", &path, None).await.1;
    assert_eq!(review["removable"], 3);
    assert_eq!(review["overlapping"], 3);
    assert!(
        review["removable_entries"]
            .as_array()
            .unwrap()
            .iter()
            .all(|entry| entry["overlapping"] == true)
    );
    // A later import owns only its new addition, not the overlapping original entries.
    let later_review = alice
        .call("GET", &format!("/api/imports/history/{later}"), None)
        .await
        .1;
    assert_eq!(later_review["removable"], 1);
    assert_eq!(
        later_review["removable_entries"][0]["description"],
        "Purchase 4"
    );
    assert_eq!(
        alice
            .call(
                "POST",
                &format!("{path}/remove"),
                Some(json!({"token":review["token"]}))
            )
            .await
            .1["removed"],
        3
    );
    let remaining = alice.call("GET", "/api/entries", None).await.1;
    assert_eq!(remaining["totals"]["transaction_count"], 1);
    assert_eq!(remaining["entries"][0]["description"], "Purchase 4");
    let later_review = alice
        .call("GET", &format!("/api/imports/history/{later}"), None)
        .await
        .1;
    assert_eq!(later_review["removable"], 1);
    let preview = alice
        .call(
            "POST",
            "/api/imports/preview",
            Some(import_input(&source, &first, json!([]))),
        )
        .await
        .1;
    assert!(
        preview["rows"]
            .as_array()
            .unwrap()
            .iter()
            .all(|row| row["status"] == "new")
    );
    assert_eq!(
        import_ok(&mut alice, &source, &first, json!([])).await["added"],
        3
    );
    assert_eq!(
        alice.call("GET", &path, None).await.1["removable"],
        0,
        "reimport creates new entries owned by the new import"
    );
    let later_path = format!("/api/imports/history/{later}");
    let later_review = alice.call("GET", &later_path, None).await.1;
    assert_eq!(
        alice
            .call(
                "POST",
                &format!("{later_path}/remove"),
                Some(json!({"token":later_review["token"]}))
            )
            .await
            .1["removed"],
        1
    );
    assert_eq!(
        alice.call("GET", "/api/entries", None).await.1["totals"]["transaction_count"],
        3,
        "removing the later import must not delete newly reimported entries"
    );
}
