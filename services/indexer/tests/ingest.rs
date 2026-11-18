//! Ingestion, restart and backfill tests against local Postgres, with an
//! in-memory event source standing in for the RPC.

use asterism_indexer::{
    get_cursor, ingest_page,
    rpc::{EventSource, EventsPage, Position, RpcEvent},
    set_cursor, sync, Cursor, Error, SyncConfig, DEFAULT_CURSOR,
};
use serde_json::json;
use sqlx::PgPool;

fn event(n: i64) -> RpcEvent {
    RpcEvent {
        id: format!("{n:010}-0000000000"),
        ledger: n,
        ledger_closed_at: "2026-11-18T17:21:49Z".to_string(),
        contract_id: "CREGISTRY".to_string(),
        tx_hash: format!("tx{n}"),
        topic_json: vec![
            json!({ "symbol": "issued" }),
            json!({ "u64": n.to_string() }),
        ],
        value_json: json!({ "map": [] }),
    }
}

fn page(range: std::ops::RangeInclusive<i64>) -> EventsPage {
    let events: Vec<RpcEvent> = range.map(event).collect();
    EventsPage {
        cursor: events.last().map(|e| e.id.clone()),
        events,
        latest_ledger: 1_000,
        oldest_ledger: 1,
    }
}

/// Serves `pages` in order. Position `Ledger(_)` returns the first page;
/// `Cursor(c)` returns the page after the one whose cursor is `c`.
/// `fail_at` makes the request for that page index fail (a crash or RPC outage).
struct FakeSource {
    pages: Vec<EventsPage>,
    fail_at: Option<usize>,
}

impl EventSource for FakeSource {
    async fn get_events(
        &self,
        position: &Position,
        _contract_id: &str,
        _limit: u32,
    ) -> Result<EventsPage, Error> {
        let index = match position {
            Position::Ledger(_) => 0,
            Position::Cursor(c) => self
                .pages
                .iter()
                .position(|p| p.cursor.as_deref() == Some(c))
                .map_or(self.pages.len(), |i| i + 1),
        };
        if self.fail_at == Some(index) {
            return Err(Error::Rpc {
                code: -1,
                message: "simulated outage".to_string(),
            });
        }
        Ok(self.pages.get(index).cloned().unwrap_or(EventsPage {
            events: vec![],
            cursor: None,
            latest_ledger: 1_000,
            oldest_ledger: 1,
        }))
    }
}

fn config() -> SyncConfig {
    SyncConfig {
        contract_id: "CREGISTRY".to_string(),
        page_limit: 3,
    }
}

async fn stored_ledgers(pool: &PgPool) -> Vec<i64> {
    sqlx::query_scalar("SELECT ledger FROM events ORDER BY event_id")
        .fetch_all(pool)
        .await
        .unwrap_or_default()
}

#[sqlx::test(migrations = "./migrations")]
async fn ingest_stores_events_and_cursor(pool: PgPool) -> Result<(), Error> {
    let p = page(1..=3);
    let inserted = ingest_page(&pool, DEFAULT_CURSOR, &p.events, "0000000003-0000000000").await?;
    assert_eq!(inserted, 3);
    assert_eq!(stored_ledgers(&pool).await, vec![1, 2, 3]);
    assert_eq!(
        get_cursor(&pool, DEFAULT_CURSOR).await?,
        Some(Cursor {
            ledger: 3,
            rpc_cursor: Some("0000000003-0000000000".to_string())
        })
    );

    let (topics, closed_at): (serde_json::Value, i64) =
        sqlx::query_as("SELECT topics, ledger_closed_at FROM events WHERE ledger = 1")
            .fetch_one(&pool)
            .await?;
    assert_eq!(topics[0], json!({ "symbol": "issued" }));
    assert_eq!(closed_at, 1_795_022_509);
    Ok(())
}

#[sqlx::test(migrations = "./migrations")]
async fn duplicate_delivery_is_idempotent(pool: PgPool) -> Result<(), Error> {
    let p = page(1..=3);
    assert_eq!(
        ingest_page(&pool, DEFAULT_CURSOR, &p.events, "c3").await?,
        3
    );
    assert_eq!(
        ingest_page(&pool, DEFAULT_CURSOR, &p.events, "c3").await?,
        0
    );
    assert_eq!(stored_ledgers(&pool).await, vec![1, 2, 3]);
    Ok(())
}

#[sqlx::test(migrations = "./migrations")]
async fn out_of_order_delivery_keeps_all_events_and_the_latest_cursor(
    pool: PgPool,
) -> Result<(), Error> {
    let first = page(1..=3);
    let second = page(4..=6);
    ingest_page(&pool, DEFAULT_CURSOR, &second.events, "c6").await?;
    ingest_page(&pool, DEFAULT_CURSOR, &first.events, "c3").await?;

    assert_eq!(stored_ledgers(&pool).await, vec![1, 2, 3, 4, 5, 6]);
    assert_eq!(
        get_cursor(&pool, DEFAULT_CURSOR).await?,
        Some(Cursor {
            ledger: 6,
            rpc_cursor: Some("c6".to_string())
        })
    );
    Ok(())
}

#[sqlx::test(migrations = "./migrations")]
async fn sync_reads_all_pages(pool: PgPool) -> Result<(), Error> {
    let source = FakeSource {
        pages: vec![page(1..=3), page(4..=6), page(7..=8)],
        fail_at: None,
    };
    let report = sync(&pool, &source, &config(), DEFAULT_CURSOR, Some(1)).await?;
    assert_eq!((report.fetched, report.inserted), (8, 8));
    assert_eq!(stored_ledgers(&pool).await, (1..=8).collect::<Vec<_>>());

    // Caught up: another run fetches nothing.
    let report = sync(&pool, &source, &config(), DEFAULT_CURSOR, Some(1)).await?;
    assert_eq!((report.fetched, report.inserted), (0, 0));
    Ok(())
}

#[sqlx::test(migrations = "./migrations")]
async fn restart_mid_batch_resumes_without_gaps_or_duplicates(pool: PgPool) -> Result<(), Error> {
    let pages = vec![page(1..=3), page(4..=6), page(7..=9)];

    // The RPC fails while fetching the third page.
    let crashing = FakeSource {
        pages: pages.clone(),
        fail_at: Some(2),
    };
    assert!(sync(&pool, &crashing, &config(), DEFAULT_CURSOR, Some(1))
        .await
        .is_err());
    assert_eq!(stored_ledgers(&pool).await, (1..=6).collect::<Vec<_>>());
    assert_eq!(
        get_cursor(&pool, DEFAULT_CURSOR)
            .await?
            .and_then(|c| c.rpc_cursor),
        Some("0000000006-0000000000".to_string())
    );

    // After the restart, with no start ledger given, it resumes from the cursor.
    let healthy = FakeSource {
        pages,
        fail_at: None,
    };
    let report = sync(&pool, &healthy, &config(), DEFAULT_CURSOR, None).await?;
    assert_eq!((report.fetched, report.inserted), (3, 3));
    assert_eq!(stored_ledgers(&pool).await, (1..=9).collect::<Vec<_>>());
    Ok(())
}

#[sqlx::test(migrations = "./migrations")]
async fn failed_page_is_stored_atomically(pool: PgPool) -> Result<(), Error> {
    // The second event has a closing time Postgres cannot parse, so the
    // whole page, including the first event, must roll back.
    let mut p = page(1..=2);
    p.events[1].ledger_closed_at = "not a timestamp".to_string();
    assert!(ingest_page(&pool, DEFAULT_CURSOR, &p.events, "c2")
        .await
        .is_err());
    assert!(stored_ledgers(&pool).await.is_empty());
    assert_eq!(get_cursor(&pool, DEFAULT_CURSOR).await?, None);
    Ok(())
}

#[sqlx::test(migrations = "./migrations")]
async fn backfill_from_a_given_ledger_does_not_move_the_main_cursor(
    pool: PgPool,
) -> Result<(), Error> {
    let live = FakeSource {
        pages: vec![page(7..=9)],
        fail_at: None,
    };
    sync(&pool, &live, &config(), DEFAULT_CURSOR, Some(7)).await?;

    let history = FakeSource {
        pages: vec![page(1..=3), page(4..=6), page(7..=9)],
        fail_at: None,
    };
    let report = sync(&pool, &history, &config(), "backfill", Some(1)).await?;
    assert_eq!((report.fetched, report.inserted), (9, 6));
    assert_eq!(stored_ledgers(&pool).await, (1..=9).collect::<Vec<_>>());
    assert_eq!(
        get_cursor(&pool, DEFAULT_CURSOR).await?.map(|c| c.ledger),
        Some(9)
    );
    Ok(())
}

#[sqlx::test(migrations = "./migrations")]
async fn fresh_start_without_a_start_ledger_is_an_error(pool: PgPool) -> Result<(), Error> {
    let source = FakeSource {
        pages: vec![page(1..=3)],
        fail_at: None,
    };
    let result = sync(&pool, &source, &config(), DEFAULT_CURSOR, None).await;
    assert!(matches!(result, Err(Error::NoStartPoint)));
    assert!(stored_ledgers(&pool).await.is_empty());
    Ok(())
}

#[sqlx::test(migrations = "./migrations")]
async fn set_cursor_can_reset_a_cursor_backwards(pool: PgPool) -> Result<(), Error> {
    let p = page(1..=3);
    ingest_page(&pool, DEFAULT_CURSOR, &p.events, "c3").await?;
    let reset = Cursor {
        ledger: 1,
        rpc_cursor: None,
    };
    set_cursor(&pool, DEFAULT_CURSOR, &reset).await?;
    assert_eq!(get_cursor(&pool, DEFAULT_CURSOR).await?, Some(reset));
    Ok(())
}
