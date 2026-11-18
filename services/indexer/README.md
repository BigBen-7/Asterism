# Indexer

Persists registry contract events into Postgres. Belongs to Phase 0 (issues 0.11-0.13); see ROADMAP.md.

## How it works

- `migrations/0001_init.sql` creates `events` (one row per contract event, unique by RPC event id) and `indexer_cursor` (where ingestion stopped).
- Events come from the Soroban RPC `getEvents` method, filtered to the registry contract and requested with `xdrFormat: "json"`, so topics and values arrive decoded.
- Each page of events and the advanced cursor are written in **one transaction**. A crash leaves either the whole page and its cursor, or neither.
- Rows are keyed by the RPC event id and inserted with `ON CONFLICT DO NOTHING`, so duplicate delivery is harmless. The cursor only moves forward, so an older page delivered late adds its events but does not rewind it.

## Event retention (confirmed)

The RPC keeps only a limited window of events. `getHealth` on the public testnet RPC reported `ledgerRetentionWindow: 120960` ledgers (about 7 days at roughly 5 seconds per ledger) when this was written. A `getEvents` request with a `startLedger` outside the retained range fails with JSON-RPC error `-32600` (`startLedger must be within the ledger range: <oldest> - <latest>`).

Consequences:

- Events older than the window cannot be recovered from the RPC. **The indexer must be running, with a start point inside the window, from the contract's deployment**, and it must persist everything it reads.
- On a fresh database there is no saved cursor, so `INDEXER_START_LEDGER` (the deployment ledger) is required. The indexer refuses to start without it rather than guess, because starting later would silently skip events.
- Resuming uses the saved RPC cursor. Whether the RPC accepts a cursor older than its window is not confirmed; if the indexer is down for longer than the window, plan to treat the gap as lost.

Re-check the window with `curl -s -X POST https://soroban-testnet.stellar.org -H 'Content-Type: application/json' -d '{"jsonrpc":"2.0","id":1,"method":"getHealth"}'`.

## Run

Requires a local Postgres (no Docker) and the variables in `.env.example` (`DATABASE_URL`, `REGISTRY_CONTRACT_ID`, `INDEXER_START_LEDGER`).

```bash
just db-up
cargo run -p asterism-indexer                      # follow the registry; resumes from the saved cursor after a restart
cargo run -p asterism-indexer -- backfill <ledger> # load history from <ledger> once, then exit
cargo test -p asterism-indexer                     # integration tests against Postgres
```

`backfill` uses its own cursor (`backfill`), so it never moves the main cursor, and rerunning it resumes where it stopped. Overlap with already-stored events is skipped.

## Limits

Chain reorganisations are not handled beyond what the RPC provides. The API (issue-side reads of this data) is a separate service.
