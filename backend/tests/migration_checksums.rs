use coinz::db;
use sqlx::{
    SqlitePool,
    migrate::{MigrateError, Migration},
    sqlite::SqliteConnectOptions,
};
use std::borrow::Cow;

async fn historical_database(directory: &std::path::Path, edited: bool) -> SqlitePool {
    let pool = SqlitePool::connect_with(
        SqliteConnectOptions::new()
            .filename(directory.join("coinz.sqlite3"))
            .create_if_missing(true),
    )
    .await
    .unwrap();
    let mut historical = sqlx::migrate!("./migrations");
    historical.migrations = Cow::Owned(
        historical
            .iter()
            .filter(|m| m.version <= 8)
            .map(|m| {
                // Simulate the original LF checkout, independently of current checkout settings.
                let mut sql = m.sql.replace("\r\n", "\n");
                if edited && m.version == 8 {
                    sql.push_str("\n-- Synthetic alteration of an applied migration.\n");
                }
                Migration::new(
                    m.version,
                    m.description.clone(),
                    m.migration_type,
                    sql.into(),
                    m.no_tx,
                )
            })
            .collect(),
    );
    historical.run(&pool).await.unwrap();
    pool
}

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
async fn original_lf_database_upgrades_without_rewriting_applied_checksums() {
    let directory = tempfile::tempdir().unwrap();
    let historical = historical_database(directory.path(), false).await;
    sqlx::query("INSERT INTO users(id,username,password_hash) VALUES('synthetic-owner','synthetic-owner','unused-synthetic-hash')")
        .execute(&historical).await.unwrap();
    sqlx::query("INSERT INTO import_profiles(id,user_id,name,layout) VALUES('synthetic-profile','synthetic-owner','Synthetic saved profile','{}')")
        .execute(&historical).await.unwrap();
    let checksum: Vec<u8> =
        sqlx::query_scalar("SELECT checksum FROM _sqlx_migrations WHERE version=8")
            .fetch_one(&historical)
            .await
            .unwrap();
    historical.close().await;
    let upgraded = db::connect(directory.path()).await.unwrap();
    let retained: Vec<u8> =
        sqlx::query_scalar("SELECT checksum FROM _sqlx_migrations WHERE version=8")
            .fetch_one(&upgraded)
            .await
            .unwrap();
    assert_eq!(checksum, retained);
    let profile: String =
        sqlx::query_scalar("SELECT name FROM import_profiles WHERE id='synthetic-profile'")
            .fetch_one(&upgraded)
            .await
            .unwrap();
    assert_eq!(profile, "Synthetic saved profile");
    let classification_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name='classifications')",
    )
    .fetch_one(&upgraded)
    .await
    .unwrap();
    assert!(classification_exists);
    upgraded.close().await;
}

#[tokio::test]
async fn altered_migration_still_fails_validation() {
    let directory = tempfile::tempdir().unwrap();
    historical_database(directory.path(), true)
        .await
        .close()
        .await;
    let error = db::connect(directory.path()).await.unwrap_err();
    assert!(matches!(
        error.downcast_ref::<MigrateError>(),
        Some(MigrateError::VersionMismatch(8))
    ));
}
