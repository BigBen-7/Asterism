//! Integration tests against a local Postgres (`just db-up`, `DATABASE_URL` set).
//! `#[sqlx::test]` creates a throwaway database per test and applies the migrations.

use asterism_indexer::{get_cursor, set_cursor, Cursor, DEFAULT_CURSOR};
use sqlx::PgPool;

#[sqlx::test(migrations = "./migrations")]
async fn migrations_create_the_tables(pool: PgPool) -> Result<(), sqlx::Error> {
    for table in ["events", "indexer_cursor"] {
        let exists: bool = sqlx::query_scalar("SELECT to_regclass($1) IS NOT NULL")
            .bind(table)
            .fetch_one(&pool)
            .await?;
        assert!(exists, "table {table} should exist");
    }
    Ok(())
}

#[sqlx::test(migrations = "./migrations")]
async fn events_are_unique_by_event_id(pool: PgPool) -> Result<(), sqlx::Error> {
    let insert = "INSERT INTO events (event_id, contract_id, ledger, ledger_closed_at, tx_hash, topics, data)
                  VALUES ($1, 'C1', 10, 1000, 'abc', '[\"issued\"]'::jsonb, '{}'::jsonb)";
    sqlx::query(insert)
        .bind("0000000010-0000000001")
        .execute(&pool)
        .await?;
    let duplicate = sqlx::query(insert)
        .bind("0000000010-0000000001")
        .execute(&pool)
        .await;
    assert!(duplicate.is_err());

    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM events")
        .fetch_one(&pool)
        .await?;
    assert_eq!(count, 1);
    Ok(())
}

#[sqlx::test(migrations = "./migrations")]
async fn cursor_roundtrip_and_update(pool: PgPool) -> Result<(), asterism_indexer::Error> {
    assert_eq!(get_cursor(&pool, DEFAULT_CURSOR).await?, None);

    let first = Cursor {
        ledger: 100,
        rpc_cursor: Some("0000000100-0000000002".to_string()),
    };
    set_cursor(&pool, DEFAULT_CURSOR, &first).await?;
    assert_eq!(get_cursor(&pool, DEFAULT_CURSOR).await?, Some(first));

    let second = Cursor {
        ledger: 150,
        rpc_cursor: None,
    };
    set_cursor(&pool, DEFAULT_CURSOR, &second).await?;
    assert_eq!(get_cursor(&pool, DEFAULT_CURSOR).await?, Some(second));
    Ok(())
}
