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

#[tokio::test]
async fn optional_source_migration_preserves_import_history_and_evidence() {
    let directory = tempfile::tempdir().unwrap();
    let pool = SqlitePool::connect_with(
        SqliteConnectOptions::new()
            .filename(directory.path().join("coinz.sqlite3"))
            .create_if_missing(true)
            .foreign_keys(true),
    )
    .await
    .unwrap();
    let mut migrator = sqlx::migrate!("./migrations");
    migrator.migrations = Cow::Owned(
        migrator
            .iter()
            .filter(|m| m.version < 12)
            .map(|m| {
                Migration::new(
                    m.version,
                    m.description.clone(),
                    m.migration_type,
                    m.sql.clone(),
                    m.no_tx,
                )
            })
            .collect(),
    );
    migrator.run(&pool).await.unwrap();
    sqlx::query("INSERT INTO users(id,username,password_hash) VALUES('u','upgrade-fixture','unused-synthetic-hash')").execute(&pool).await.unwrap();
    sqlx::query(
        "INSERT INTO transaction_sources(id,user_id,name) VALUES('s','u','Synthetic source')",
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query("INSERT INTO transactions(id,user_id,source_id,date,description,amount_cents,origin,revision) VALUES('t','u','s','2026-11-01','Preserved synthetic entry',-1234,'csv',3)").execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO import_batches(id,user_id,source_id,filename,added,interpretation) VALUES('b','u','s','synthetic.csv',1,'{}')").execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO import_batch_entries(batch_id,user_id,transaction_id,role,revision) VALUES('b','u','t','added',2)").execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO import_records(user_id,source_id,file_hash,record_number,external_id,date,description,amount_cents,transaction_id,disposition,interpretation) VALUES('u','s','synthetic-hash',2,'bank-id','2026-11-01','Preserved evidence',-1234,'t','accepted','{}')").execute(&pool).await.unwrap();
    pool.close().await;
    let updated = db::connect(directory.path()).await.unwrap();
    let evidence: (String, String, String, i64) =
        sqlx::query_as("SELECT source_id,scope_id,description,amount_cents FROM import_records")
            .fetch_one(&updated)
            .await
            .unwrap();
    assert_eq!(
        evidence,
        ("s".into(), "s".into(), "Preserved evidence".into(), -1234)
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT revision FROM import_batch_entries")
            .fetch_one(&updated)
            .await
            .unwrap(),
        2
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT revision FROM transactions")
            .fetch_one(&updated)
            .await
            .unwrap(),
        3
    );
    assert_eq!(
        sqlx::query_scalar::<_, String>("SELECT interpretation FROM import_batches")
            .fetch_one(&updated)
            .await
            .unwrap(),
        "{}"
    );
    assert!(
        sqlx::query("PRAGMA foreign_key_check")
            .fetch_all(&updated)
            .await
            .unwrap()
            .is_empty()
    );
    sqlx::query("DELETE FROM transactions WHERE id='t'")
        .execute(&updated)
        .await
        .unwrap();
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM import_records")
            .fetch_one(&updated)
            .await
            .unwrap(),
        0
    );
}
