# Indexer

Persists registry contract events into Postgres. Belongs to Phase 0 (issues 0.11-0.13); see ROADMAP.md.

This crate is the scaffold (0.11): SQLx, the schema, migrations and the ingestion cursor. Event ingestion (0.12) and backfill/resume (0.13) come next.

## Schema

`migrations/0001_init.sql` creates:

- `events`: one row per contract event, unique by RPC `event_id` so ingestion is idempotent.
- `indexer_cursor`: where ingestion stopped, so it can resume after a restart.

## Run

Requires a local Postgres (no Docker) and `DATABASE_URL` (see `.env.example`).

```bash
just db-up
export DATABASE_URL=postgres://localhost:5432/asterism
cargo run -p asterism-indexer      # applies migrations
cargo test -p asterism-indexer     # integration tests against Postgres
```
