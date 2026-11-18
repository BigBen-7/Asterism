//! Asterism indexer: persists registry events into Postgres.
//!
//! Ingestion (issue 0.12) reads `getEvents` pages from the RPC and stores each
//! page and the advanced cursor in one transaction. Because events are keyed
//! by their RPC id, delivery can be duplicated or out of order without
//! creating duplicates. Resume and backfill (issue 0.13) are the same loop
//! started from the saved cursor or from a given ledger.

pub mod rpc;

use rpc::{EventSource, EventsPage, Position, RpcEvent};
use sqlx::{postgres::PgPoolOptions, PgConnection, PgPool};

/// Migrations from `services/indexer/migrations`, embedded at build time.
pub static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");

/// Cursor name used by the main ingestion loop.
pub const DEFAULT_CURSOR: &str = "registry";
/// Cursor name used by `backfill`, so backfilling never moves the main cursor.
pub const BACKFILL_CURSOR: &str = "backfill";

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("missing environment variable {0}")]
    MissingEnv(&'static str),
    #[error("invalid value for {name}: {value}")]
    InvalidEnv { name: &'static str, value: String },
    #[error("no saved cursor and no start ledger: set INDEXER_START_LEDGER to the ledger the contract was deployed in")]
    NoStartPoint,
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("migration error: {0}")]
    Migrate(#[from] sqlx::migrate::MigrateError),
    #[error("RPC request failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("RPC error {code}: {message}")]
    Rpc { code: i64, message: String },
    #[error("RPC response had neither a result nor an error")]
    EmptyRpcResponse,
}

/// Where ingestion stopped.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cursor {
    pub ledger: i64,
    pub rpc_cursor: Option<String>,
}

/// Settings for one sync run.
#[derive(Debug, Clone)]
pub struct SyncConfig {
    pub contract_id: String,
    /// Events requested per RPC call.
    pub page_limit: u32,
}

/// What a sync run did.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SyncReport {
    pub fetched: u64,
    /// Events that were new (not already stored).
    pub inserted: u64,
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

/// Saves the cursor, replacing any previous value for `name`. Use this to
/// reset a cursor deliberately; ingestion itself only moves cursors forward.
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

/// Stores one event. Returns `true` if it was new, `false` if its RPC id was already stored.
async fn insert_event(conn: &mut PgConnection, event: &RpcEvent) -> Result<bool, Error> {
    let result = sqlx::query(
        "INSERT INTO events (event_id, contract_id, ledger, ledger_closed_at, tx_hash, topics, data)
         VALUES ($1, $2, $3, EXTRACT(EPOCH FROM $4::text::timestamptz)::bigint, $5, $6, $7)
         ON CONFLICT (event_id) DO NOTHING",
    )
    .bind(&event.id)
    .bind(&event.contract_id)
    .bind(event.ledger)
    .bind(&event.ledger_closed_at)
    .bind(&event.tx_hash)
    .bind(sqlx::types::Json(&event.topic_json))
    .bind(sqlx::types::Json(&event.value_json))
    .execute(conn)
    .await?;
    Ok(result.rows_affected() > 0)
}

/// Moves the cursor forward. A page that is older than the saved cursor
/// (out-of-order delivery) leaves the cursor where it is.
async fn advance_cursor(
    conn: &mut PgConnection,
    name: &str,
    ledger: i64,
    rpc_cursor: &str,
) -> Result<(), Error> {
    sqlx::query(
        "INSERT INTO indexer_cursor (name, ledger, rpc_cursor) VALUES ($1, $2, $3)
         ON CONFLICT (name) DO UPDATE
         SET ledger = EXCLUDED.ledger, rpc_cursor = EXCLUDED.rpc_cursor, updated_at = now()
         WHERE indexer_cursor.ledger <= EXCLUDED.ledger",
    )
    .bind(name)
    .bind(ledger)
    .bind(rpc_cursor)
    .execute(conn)
    .await?;
    Ok(())
}

/// Persists a page of events and advances the cursor in one transaction, so a
/// crash leaves either the whole page and its cursor or neither. Returns the
/// number of events that were new.
pub async fn ingest_page(
    pool: &PgPool,
    cursor_name: &str,
    events: &[RpcEvent],
    rpc_cursor: &str,
) -> Result<u64, Error> {
    let mut tx = pool.begin().await?;
    let mut inserted = 0;
    for event in events {
        if insert_event(&mut tx, event).await? {
            inserted += 1;
        }
    }
    let ledger = events.iter().map(|e| e.ledger).max().unwrap_or(0);
    advance_cursor(&mut tx, cursor_name, ledger, rpc_cursor).await?;
    tx.commit().await?;
    Ok(inserted)
}

/// Reads events until the source has nothing more, persisting page by page.
///
/// Starts from the saved cursor `cursor_name` if there is one (resume), else
/// from `from_ledger`. With neither it fails with [`Error::NoStartPoint`]
/// rather than guessing, because starting later than the contract's
/// deployment would silently skip events.
pub async fn sync<S: EventSource>(
    pool: &PgPool,
    source: &S,
    config: &SyncConfig,
    cursor_name: &str,
    from_ledger: Option<i64>,
) -> Result<SyncReport, Error> {
    let mut position = match get_cursor(pool, cursor_name).await? {
        Some(Cursor {
            rpc_cursor: Some(cursor),
            ..
        }) => Position::Cursor(cursor),
        Some(Cursor { ledger, .. }) => Position::Ledger(ledger),
        None => Position::Ledger(from_ledger.ok_or(Error::NoStartPoint)?),
    };

    let mut report = SyncReport::default();
    loop {
        let EventsPage { events, cursor, .. } = source
            .get_events(&position, &config.contract_id, config.page_limit)
            .await?;
        let Some(last) = events.last() else {
            return Ok(report);
        };
        let next = cursor.unwrap_or_else(|| last.id.clone());
        report.fetched += events.len() as u64;
        report.inserted += ingest_page(pool, cursor_name, &events, &next).await?;
        position = Position::Cursor(next);
    }
}
