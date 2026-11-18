# API

Prepares credential issuance (single and batch) and verifies credentials against the indexer's Postgres tables. Phase 1; see [docs/api.md](../../docs/api.md) for endpoints, the verification payload and the link / QR format.

```bash
cargo run -p asterism-api    # needs DATABASE_URL, REGISTRY_CONTRACT_ID, CREDENTIALS_CONTRACT_ID
cargo test -p asterism-api   # no database required
```
