# RFC 0003: Credential schema format and optional W3C Verifiable Credentials mapping

- Status: Proposed
- Issue: 1.1 (credentials phase)
- Related: RFC 0001 (attestation model), CLAUDE.md sections 4.1 and 10.3

## Summary

This RFC defines what a credential schema is off-chain, how a credential's claims are committed to on-chain through the attestation registry, and an optional, one-way mapping to the W3C Verifiable Credentials (VC) data model so credentials can be consumed by VC tooling.

## Goals and non-goals

Goals: a small, deterministic schema and credential format that fits the registry's "only commitments live on-chain" rule (RFC 0001, decision 1); a privacy-preserving way to verify a single claim; an optional VC mapping that does not change the on-chain format.

Non-goals: issuer registration and authorization (the `credentials` contract), API and SDK shapes, selective-disclosure proofs (zero-knowledge or BBS+), and DID method design.

## Decisions

### 1. The registry already fixes the on-chain side

A credential schema is registered with the registry's `register_schema(name, version, fields_hash, revocable)`. `fields_hash` is the SHA-256 of the **schema document** defined here. Everything else about the schema lives off-chain, so the registry needs no changes. A schema change is a new registry schema with a higher `version` (RFC 0001, decision 2).

### 2. Schema document

A schema document is a JSON object:

```json
{
  "asterism": "credential-schema/1",
  "name": "University degree",
  "version": 1,
  "description": "Awarded on completion of a degree programme.",
  "fields": {
    "degree": { "type": "string", "required": true },
    "programme": { "type": "string", "required": true },
    "graduated": { "type": "date", "required": true },
    "honours": { "type": "string", "required": false }
  }
}
```

Rules:

- `asterism` is the document format version, so the format itself can evolve.
- `name` and `version` must equal the values passed to `register_schema`. Tools reject a document whose `name` is longer than the registry's 64-byte limit.
- `fields` maps field names to `{ type, required }`. Field names are unique strings. Allowed types: `string`, `integer`, `boolean`, `date` (an ISO 8601 `YYYY-MM-DD` string), `datetime` (an RFC 3339 string) and `address` (a Stellar account or contract strict-key string).
- No nested objects or arrays in v1. A flat field set keeps verification and the VC mapping simple. Nesting is an open question below.
- Unknown top-level keys are rejected, so two parties hashing "the same" schema cannot disagree because of extra keys.

### 3. Canonicalization and hashing

All documents that are hashed are first serialized with the JSON Canonicalization Scheme ([RFC 8785](https://www.rfc-editor.org/rfc/rfc8785), JCS): UTF-8, keys sorted, no insignificant whitespace, deterministic number formatting. Then:

```
fields_hash = SHA-256( JCS(schema document) )
```

The schema document should be published at a stable location (for example IPFS, a project site, or alongside the issuer's metadata). The hash is what makes any copy checkable, so the location is not trusted.

### 4. Credentials and their commitments

A **credential record** is a JSON object, kept off-chain by the issuer and the holder:

```json
{
  "schema": { "registry": "<registry contract id>", "id": 7 },
  "subject": "G...",
  "claims": { "degree": "BSc", "programme": "Computer Science", "graduated": "2026-07-15" },
  "salt": "<32 random bytes, hex>"
}
```

- `claims` must satisfy the schema document: every required field present, types respected, no unknown fields.
- `salt` is 32 random bytes from a secure source, unique per credential. Without it, a verifier could guess low-entropy claims by hashing candidates against the on-chain commitment.
- `subject` is the holder's Stellar address and is part of what is hashed, so a credential cannot be replayed for another holder.

Commitments, using the registry's existing rules:

- **Single credential:** `data_hash = SHA-256( JCS(credential record) )`, passed to `issue`. The registry's `subject` equals `subject` in the record.
- **Batch:** the Merkle leaf bytes are `JCS(credential record)`. The registry hashes leaves as `SHA-256(0x00 || leaf_bytes)` (RFC 0001, decision 3), so the issuer does not add a prefix. The batch attestation has no on-chain `subject`; the subject lives only in the record.

To verify, a holder presents the credential record (and, for a batch, the Merkle proof). The verifier checks the claims against the schema document (whose hash matches the registered `fields_hash`), recomputes the commitment, and asks the registry for the attestation status (valid, expired or revoked).

### 5. Privacy

The chain holds only the commitment and the issuer, schema, timestamps and (for single credentials) subject address. Claim values never go on-chain. Because the record includes a salt, publishing a commitment reveals nothing about the claims.

Revealing a credential reveals all its claims: v1 has no selective disclosure. Issuers wanting a holder to prove one fact (for example "over 18" without a birth date) should define a schema for that single fact. Selective disclosure is a later extension.

### 6. Optional W3C Verifiable Credentials mapping

The mapping is optional and one-way: Asterism credentials can be _exported_ as VC Data Model 2.0 documents so VC tooling can read them. The VC form does not replace the on-chain commitment, and verification of an exported VC still ends at the registry.

| Asterism                                       | VC Data Model 2.0 field                                                   |
| ---------------------------------------------- | ------------------------------------------------------------------------- |
| `claims` (field name and value)                | `credentialSubject` properties                                            |
| `subject`                                      | `credentialSubject.id`, as a `did:pkh` identifier for the Stellar address |
| registry `issuer` address                      | `issuer`, as a `did:pkh` identifier                                       |
| registry `issued_at`                           | `validFrom`                                                               |
| registry `expires_at`                          | `validUntil`                                                              |
| schema document and `fields_hash`              | `credentialSchema` (`id` is the document location, plus the hash)         |
| revocable schema and attestation id            | `credentialStatus`, pointing at the registry attestation                  |
| `salt`                                         | a custom property inside the proof section; it is never part of `claims`  |
| commitment (`data_hash` or Merkle root + path) | `proof`, using a custom `AsterismRegistryProof` type                      |

Notes:

- The `proof` is not a signature over the VC. It is a pointer: registry contract id, attestation id and, for batches, the Merkle proof. A verifier that does not know `AsterismRegistryProof` cannot verify the credential and must treat it as unverified, not as invalid.
- Which `did:pkh` namespace to use for Stellar (the CAIP-2 chain identifiers for the Stellar public and test networks) is `needs-verification`: confirm against the current CAIP and `did:pkh` specifications before the exporter is written.
- Whether to also define a JSON-LD context for the custom terms, or use `@vocab`, is an open question.
- Importing arbitrary third-party VCs is out of scope.

### 7. Where the logic lives

- Schema and record validation and JCS hashing are implemented once as a pure library and exposed through the TypeScript SDK (issue 1.7) and the Rust API (1.5, 1.6). The contracts never see claims.
- The `credentials` contract (issues 1.2 to 1.4) adds issuer registration and authorization on top of the registry. It does not need to understand the schema document.

## Alternatives considered

- **Hash claims without a salt.** Rejected: low-entropy claims (dates, grades) can be guessed from the public commitment.
- **Store one field hash per claim and reveal individually.** Deferred: it allows selective disclosure but multiplies leaves and complicates the model. A Merkle tree over fields could be added later without breaking v1 records, by using a new `asterism` format version.
- **Make W3C VCs the primary format.** Rejected: it brings in JSON-LD processing and signature suites that the registry does not need, and VC proofs would duplicate the on-chain commitment. A one-way export gets interoperability at much lower cost.
- **A custom canonical form instead of JCS.** Rejected: JCS is a published standard with implementations in the languages used here.

## Open questions

1. Nested objects and arrays in `fields`: add in a later schema format version, or never?
2. Which `did:pkh` namespace identifies Stellar accounts (`needs-verification`)?
3. Should the exporter emit a JSON-LD context, or use `@vocab`?
4. Where should schema documents be published by default?
5. Is a per-credential `validUntil` that differs from the registry `expires_at` ever needed? This RFC says no: the registry value is authoritative.

## Impact on other issues

1.2 to 1.4 follow RFC 0001 and do not depend on this RFC's details. 1.5 to 1.7 implement validation, canonicalization and commitments as described here. 1.8 and 1.9 build the issuer and verifier interfaces on those.
