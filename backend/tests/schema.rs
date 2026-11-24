use coinz::db;
use sqlx::{
    SqlitePool,
    migrate::{MigrateError, Migration},
    sqlite::SqliteConnectOptions,
};
use std::borrow::Cow;

#[test]
fn embedded_migrations_use_lf_for_stable_checksums() {
    for migration in sqlx::migrate!("./migrations").iter() {
        assert!(
            !migration.sql.contains('\r'),
            "Migration {} must use LF: SQLx hashes raw bytes.",
            migration.version
        );
    }
}

#[tokio::test]
async fn fresh_schema_reopens_without_losing_data_or_creating_retired_tables() {
    let directory = tempfile::tempdir().unwrap();
    let pool = db::connect(directory.path()).await.unwrap();
    sqlx::query("INSERT INTO users(id,username,password_hash) VALUES('u','synthetic','unused-synthetic-hash')").execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO transactions(id,user_id,date,description,amount_cents,origin) VALUES('t','u','2026-11-01','Synthetic source-free entry',-1234,'manual')").execute(&pool).await.unwrap();
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM _sqlx_migrations")
            .fetch_one(&pool)
            .await
            .unwrap(),
        1
    );
    pool.close().await;

    let reopened = db::connect(directory.path()).await.unwrap();
    let entry: (String, i64, Option<String>) =
        sqlx::query_as("SELECT description,amount_cents,source_id FROM transactions WHERE id='t'")
            .fetch_one(&reopened)
            .await
            .unwrap();
    assert_eq!(entry, ("Synthetic source-free entry".into(), -1234, None));
    assert_eq!(sqlx::query_scalar::<_,i64>("SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name IN ('accounts','balance_snapshots','legacy_opening_balances')").fetch_one(&reopened).await.unwrap(),0);
    assert!(
        sqlx::query("PRAGMA foreign_key_check")
            .fetch_all(&reopened)
            .await
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        sqlx::query_scalar::<_, String>("PRAGMA integrity_check")
            .fetch_one(&reopened)
            .await
            .unwrap(),
        "ok"
    );
}

#[tokio::test]
async fn incompatible_history_is_rejected_without_resetting_existing_data() {
    let directory = tempfile::tempdir().unwrap();
    let options = SqliteConnectOptions::new()
        .filename(directory.path().join("coinz.sqlite3"))
        .create_if_missing(true);
    let pool = SqlitePool::connect_with(options.clone()).await.unwrap();
    let mut old = sqlx::migrate!("./migrations");
    old.migrations = Cow::Owned(vec![Migration::new(
        1,
        "synthetic previous baseline".into(),
        sqlx::migrate::MigrationType::Simple,
        "CREATE TABLE synthetic_marker (value TEXT); INSERT INTO synthetic_marker VALUES ('preserve me');".into(),
        false,
    )]);
    old.run(&pool).await.unwrap();
    pool.close().await;

    let error = db::connect(directory.path()).await.unwrap_err();
    assert!(matches!(
        error.downcast_ref::<MigrateError>(),
        Some(MigrateError::VersionMismatch(1))
    ));
    let unchanged = SqlitePool::connect_with(options).await.unwrap();
    assert_eq!(
        sqlx::query_scalar::<_, String>("SELECT value FROM synthetic_marker")
            .fetch_one(&unchanged)
            .await
            .unwrap(),
        "preserve me"
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='users'"
        )
        .fetch_one(&unchanged)
        .await
        .unwrap(),
        0
    );
}
