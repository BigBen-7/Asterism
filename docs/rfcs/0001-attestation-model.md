# RFC 0001: Attestation data model

- Status: Proposed
- Issue: #35 (0.1)
- Related: RFC 0002 (state archival and TTL), CLAUDE.md section 4.1

## Summary

This RFC defines the registry's core records (schema, attestation), how revocation and expiry work, whether batches share the single-record type, and what lives on-chain versus off-chain.

## Goals and non-goals

Goals: a minimal model that credentials and reputation can produce into, and funding and governance can read from, without any module depending on another.

Non-goals: payload formats (modules define them), TTL policy (RFC 0002), credential-specific rules such as issuer allow-lists (Phase 1).

## Decisions

### 1. Only commitments live on-chain

The registry stores hashes, never payloads. The attestation payload (the actual claim, any personal data) stays off-chain with the issuer or subject, and the chain holds a 32-byte commitment. This keeps storage minimal, keeps personal data off a public ledger, and fits Soroban entry-size and rent costs.

The hash function is SHA-256 (available as a host function on Soroban).

### 2. Schemas

```
Schema {
    name:        String,       // human label, max 64 bytes
    version:     u32,
    fields_hash: BytesN<32>,   // SHA-256 of the off-chain schema definition
    revocable:   bool,         // fixed at registration, cannot change
    creator:     Address,
}
```

- Schema ids are `u64`, assigned from a monotonically increasing counter stored in instance storage. Counters are simpler and cheaper than deriving ids by hashing, and ids are never reused.
- Schemas are immutable once registered. A change is a new schema (a new id, a higher `version`).
- `register_schema` requires the creator's authorization (`require_auth`).
- Anyone can register a schema. Trust in a schema comes from who issues under it, not from registration.

### 3. Attestations: one record type, two commitment kinds

```
Commitment = Single(BytesN<32>)   // data_hash of one claim
           | Batch(BytesN<32>)    // Merkle root over many claims

Attestation {
    schema_id:  u64,
    issuer:     Address,
    subject:    Option<Address>,   // Some for Single, None for Batch
    commitment: Commitment,
    issued_at:  u64,               // ledger timestamp
    expires_at: Option<u64>,
    revoked_at: Option<u64>,
}
```

Attestation ids are also `u64` from a counter. Single vs. batch is decided as follows:

- One record type with a `Commitment` enum, so there is a single storage layout, one `get`, one `revoke` and one event shape.
- A **batch** is one registry entry whose commitment is a Merkle root. Subjects live in the leaves, so `subject` is `None`. A leaf is `SHA-256(0x00 || leaf_bytes)` and interior nodes are `SHA-256(0x01 || left || right)` (domain separation against second-preimage attacks). Leaf bytes are defined by the module that issues the batch.
- `verify_inclusion(attestation_id, leaf, proof)` checks a proof against the stored root and the attestation's status. Proof length is bounded (max 32 levels) so there is no unbounded loop.
- The Merkle root is what makes issuing thousands of credentials one write.

### 4. Issuance and authorization

- `issue` and `issue_batch` require the issuer's authorization (`require_auth`). The issuer is recorded in the attestation.
- The registry does not decide _who may issue under a schema_. Any address may issue; consumers choose which issuers to trust. (Credential-specific issuer registration is a Phase 1 concern.)
- `issued_at` is the ledger timestamp at issuance. `expires_at`, if set, must be strictly in the future.

### 5. Revocation

- Revocation sets `revoked_at` and is never a delete, so history stays verifiable.
- Only the **issuer** may revoke, and only when the schema is `revocable`. Revoking twice is an error.
- Batches are revoked as a whole. Revoking one leaf of a batch is out of scope for v0; an issuer who needs per-item revocation should issue single attestations. (Open question below.)

### 6. Expiry and status

`status(attestation, now)` is a pure function:

1. `Revoked` if `revoked_at` is set,
2. else `Expired` if `expires_at <= now`,
3. else `Valid`.

Expired records are not deleted; they remain readable. Archival of old records is governed by RFC 0002.

### 7. Events

Every state change emits an event so the indexer can rebuild state from events alone: `schema_registered`, `issued`, `batch_issued`, `revoked`. Events carry ids and the commitment, not payloads.

### 8. Errors

A single `#[contracterror]` enum with stable numeric codes: `SchemaNotFound`, `AttestationNotFound`, `NameTooLong`, `InvalidExpiry`, `NotRevocable`, `AlreadyRevoked`, `NotIssuer`, `InvalidProof`, `ProofTooLong`. Codes are append-only.

## Operations (for issues 0.4 to 0.8)

`register_schema`, `get_schema`, `issue`, `issue_batch`, `get`, `status`, `verify` (single), `verify_inclusion` (batch), `revoke`, `extend_ttl`.

## Alternatives considered

- **Separate record types for single and batch.** Rejected: doubles storage layouts, events and API surface for no gain.
- **Hash-derived ids.** Rejected for now: counters are cheaper and simpler; collisions are impossible.
- **Per-leaf revocation inside batches.** Deferred: it needs a second keyed store (attestation id plus leaf hash) and its rent policy.
- **Storing payloads on-chain.** Rejected: cost, privacy, and entry size limits.

## Open questions

1. Per-leaf revocation for batches: add later as a separate `revoke_leaf` keyed by `(attestation_id, leaf_hash)`, or never?
2. Should schema creators be able to restrict issuers? Deferred to the credentials module.
3. Counter contention under heavy concurrent issuance: check in benchmarks (issue 0.9).

## Impact on other issues

0.3 implements these types in `asterism-types`; 0.4 to 0.7 implement the operations; 0.8 follows RFC 0002.
