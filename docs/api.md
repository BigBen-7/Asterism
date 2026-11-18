# Credentials API

`services/api` prepares credential issuance and verifies credentials. It holds **no keys**: issuance endpoints return the data and the contract call, and the issuer signs and submits the call themselves. Formats follow [RFC 0003](rfcs/0003-credential-schema-and-vc-mapping.md). Testnet only; unaudited.

## Run

Requires the indexer to be running against the same Postgres (the API reads its `events` table) and these variables (see `.env.example`):

| Variable                  | Meaning                                                                       |
| ------------------------- | ----------------------------------------------------------------------------- |
| `DATABASE_URL`            | Postgres used by the indexer                                                  |
| `REGISTRY_CONTRACT_ID`    | Registry contract id; events are read for this contract                       |
| `CREDENTIALS_CONTRACT_ID` | Credentials contract id; issuance calls target it                             |
| `VERIFY_BASE_URL`         | Base URL of the verifier page used in links (default `http://localhost:3000`) |
| `API_BIND`                | Listen address (default `127.0.0.1:8080`)                                     |

```bash
cargo run -p asterism-api
```

## Endpoints

### `POST /v1/credentials`: prepare a single credential

```json
{
  "issuer": "G...",
  "schema_id": 3,
  "schema_document": {
    "asterism": "credential-schema/1",
    "name": "University degree",
    "version": 1,
    "fields": { "degree": { "type": "string", "required": true } }
  },
  "subject": "G...",
  "claims": { "degree": "BSc" },
  "expires_at": 1893456000
}
```

Validates the schema document and the claims, generates a random salt, and returns:

- `credential`: the record (schema reference, subject, claims, salt). Give it to the holder; the chain never sees the claims.
- `commitment`: hex `SHA-256(JCS(credential))`.
- `call`: the `credentials` contract call to sign and submit, `issue(issuer, schema_id, subject, data_hash, expires_at)` with `data_hash` equal to `commitment`.

The issuer must be registered and authorized for `schema_id` on the credentials contract, or the call will fail with `IssuerNotAuthorized`. After the call succeeds, the returned attestation id is needed to build verification links.

### `POST /v1/credentials/batch`: prepare a batch

Same fields, with `credentials: [{ "subject": "G...", "claims": { ... } }, ...]` (1 to 1000 items) instead of `subject` and `claims`. Returns:

- `root`: hex Merkle root.
- `credentials`: one `{ credential, proof }` per input, in order. `proof` is the hex sibling hashes from leaf to root that the holder keeps.
- `call`: `issue_batch(issuer, schema_id, root, expires_at)`.

Hashing matches the registry: a leaf is `SHA-256(0x00 || JCS(credential))`, a node is `SHA-256(0x01 || smaller || larger)` (children ordered bytewise), and an unpaired node is carried up unchanged.

### `POST /v1/verify`: verify a credential

The body is either `{ "payload": <payload> }` or `{ "link": "<link>" }` (see below). The API checks, in this order:

| Check                       | Meaning                                                                                                                                                  |
| --------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `credential_matches_schema` | The schema document is well formed and the claims satisfy it.                                                                                            |
| `registry`                  | The credential names this API's registry.                                                                                                                |
| `attestation_found`         | The attestation id exists in the indexed events. Stops here if not.                                                                                      |
| `schema_id`                 | The attestation was issued under the credential's schema.                                                                                                |
| `schema_document_hash`      | The schema document hashes to the registered `fields_hash`.                                                                                              |
| `commitment`                | The credential (single) or its Merkle proof (batch) matches on-chain. For a single credential the on-chain subject must also equal the record's subject. |

Response: `{ "valid": bool, "status": "valid" | "expired" | "revoked", "checks": [{ "name", "passed", "detail"? }] }`. `valid` is true only if every check passed and the status is `valid`. `status` is derived the same way as the registry: revoked wins, then expired when `expires_at <= now`. A request that is malformed returns `400`; a credential that simply fails verification returns `200` with `valid: false` and the failed checks.

### `POST /v1/verify/link`: build a link / QR payload

Body: `{ "attestation_id", "schema_document", "credential", "proof"? }`. Returns `{ "link": "<base>/verify#<payload>" }`.

### `GET /health`

Returns `ok`.

## Link and QR payload

```json
{
  "v": 1,
  "attestation_id": 12,
  "schema_document": { ... },
  "credential": { ... },
  "proof": ["<hex>", "..."]
}
```

- `v` is the payload version (currently 1). `proof` is present only for batch credentials.
- The link is `<VERIFY_BASE_URL>/verify#<payload>`, where `<payload>` is the payload's canonical JSON (RFC 8785) as base64url without padding. The payload is in the URL fragment, so browsers do not send it to a server.
- A QR code encodes the link string. Links are typically 700 to 1500 characters, which fits a QR code at a medium error-correction level; very large schema documents or long proofs may need a larger code.
- The payload contains the claims. Whoever holds the link can read them, so share it only with the intended verifier.

## Where chain data comes from

The registry emits `schema_registered` (with `fields_hash`), `issued` and `batch_issued` (with the commitment or root and `expires_at`) and `revoked` events. The indexer stores them decoded, and the API reads that table. Verification therefore reflects what the indexer has ingested: an attestation issued moments ago may not be visible until the indexer catches up.

## Not covered

Signing or submitting transactions, authentication of API callers, rate limiting, and selective disclosure.
