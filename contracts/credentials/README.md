# credentials

Credentials module (Phase 1). Depends only on the attestation registry.

Implemented: issuer registration and per-schema issuer authorization (`register_issuer`, `authorize_issuer`, `deauthorize_issuer`, `is_authorized`). The model and storage layout are documented in `src/lib.rs`; the off-chain schema and credential format is in [RFC 0003](../../docs/rfcs/0003-credential-schema-and-vc-mapping.md).

Issuing, revoking, verifying and batch issuance come in later issues.

Testnet only. Unaudited.
