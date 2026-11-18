//! Asterism indexer: persists registry events into Postgres.
//!
//! This is the scaffold from issue 0.11: schema, migrations and the cursor.
//! Event ingestion (0.12) and backfill (0.13) build on it.

use sqlx::{postgres::PgPoolOptions, PgPool};

/// Migrations from `services/indexer/migrations`, embedded at build time.
pub static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");

/// Name of the cursor row used by the main ingestion loop.
pub const DEFAULT_CURSOR: &str = "registry";

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("missing environment variable {0}")]
    MissingEnv(&'static str),
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("migration error: {0}")]
    Migrate(#[from] sqlx::migrate::MigrateError),
}

/// Where ingestion stopped.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cursor {
    pub ledger: i64,
    pub rpc_cursor: Option<String>,
}

/// Connects to Postgres.
pub async fn connect(database_url: &str) -> Result<PgPool, Error> {
    Ok(PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await?)
}

/// Applies all pending migrations.
pub async fn migrate(pool: &PgPool) -> Result<(), Error> {
    MIGRATOR.run(pool).await?;
    Ok(())
}

/// Reads the saved cursor, or `None` if ingestion has not started.
pub async fn get_cursor(pool: &PgPool, name: &str) -> Result<Option<Cursor>, Error> {
    let row = sqlx::query_as::<_, (i64, Option<String>)>(
        "SELECT ledger, rpc_cursor FROM indexer_cursor WHERE name = $1",
    )
    .bind(name)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|(ledger, rpc_cursor)| Cursor { ledger, rpc_cursor }))
}

/// Saves the cursor, replacing any previous value for `name`.
pub async fn set_cursor(pool: &PgPool, name: &str, cursor: &Cursor) -> Result<(), Error> {
    sqlx::query(
        "INSERT INTO indexer_cursor (name, ledger, rpc_cursor) VALUES ($1, $2, $3)
         ON CONFLICT (name) DO UPDATE
         SET ledger = EXCLUDED.ledger, rpc_cursor = EXCLUDED.rpc_cursor, updated_at = now()",
    )
    .bind(name)
    .bind(cursor.ledger)
    .bind(&cursor.rpc_cursor)
    .execute(pool)
    .await?;
    Ok(())
}
