# credentials

Credentials module (Phase 1). Depends only on the attestation registry.

Implemented: issuer registration and per-schema issuer authorization (`register_issuer`, `authorize_issuer`, `deauthorize_issuer`, `is_authorized`), and `issue`, `issue_batch`, `revoke`, `status`, `verify`, `verify_inclusion`. The model and storage layout are documented in `src/lib.rs`; the off-chain schema and credential format is in [RFC 0003](../../docs/rfcs/0003-credential-schema-and-vc-mapping.md).

Issuing, revoking and verifying credentials (single and Merkle batch) go through the registry and require an authorized issuer; see `src/lib.rs`.

Testnet only. Unaudited.
