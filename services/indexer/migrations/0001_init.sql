-- Events emitted by the registry contract, persisted as they arrive.
-- RPC event retention is limited, so every event must be stored from the first deployment.
CREATE TABLE events (
    id               BIGSERIAL PRIMARY KEY,
    -- Unique RPC event id; makes ingestion idempotent (backfill and restart safe).
    event_id         TEXT        NOT NULL UNIQUE,
    contract_id      TEXT        NOT NULL,
    ledger           BIGINT      NOT NULL,
    ledger_closed_at BIGINT      NOT NULL,
    tx_hash          TEXT        NOT NULL,
    -- Decoded event topics (first topic is the event name, e.g. "issued") and data.
    topics           JSONB       NOT NULL,
    data             JSONB       NOT NULL,
    inserted_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX events_contract_ledger_idx ON events (contract_id, ledger);

-- Where ingestion stopped, so the indexer can resume after a restart.
CREATE TABLE indexer_cursor (
    name        TEXT PRIMARY KEY,
    ledger      BIGINT      NOT NULL,
    -- Opaque RPC pagination cursor, if the last response provided one.
    rpc_cursor  TEXT,
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);
