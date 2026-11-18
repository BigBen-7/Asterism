# Architecture

Asterism is a small public-goods stack on Stellar (Soroban, Rust) built around one shared **attestation registry**. This page describes the layers and the one rule that keeps the modules independent. What exists today is called out explicitly; later phases are only named.

## Layers and data flow

```
Issuers/Apps --> contracts (Soroban) --> events --> indexer (Rust) --> Postgres --> API (Rust) --> SDK (TS) --> Web apps
                                                                          ^
                                          GitHub App (webhooks) ----------+   (Phase 2)
```

| Layer     | Location                | Role                                                                                                                                                      | Status (Phase 0)                                        |
| --------- | ----------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------- |
| Contracts | `contracts/`            | Soroban smart contracts. `attestation-registry` stores schemas and attestations and emits an event per change.                                            | Registry implemented; other contracts are later phases. |
| Types     | `crates/asterism-types` | Shared `no_std` types (schema, attestation, errors) used by contracts and services.                                                                       | Implemented                                             |
| Indexer   | `services/indexer`      | Reads registry events from the Stellar RPC and persists them in Postgres. The RPC keeps only a limited window of events, so this is the permanent record. | Ingestion, resume and backfill implemented              |
| API       | `services/api`          | Serves indexed data and issuance/verification endpoints.                                                                                                  | Placeholder (Phase 1)                                   |
| SDK       | `packages/sdk`          | TypeScript client for apps.                                                                                                                               | Scaffold only                                           |
| Web apps  | `apps/`                 | Next.js frontends per module and the docs site.                                                                                                           | Placeholders                                            |

On-chain, the registry stores only commitments (hashes), never payloads. The data model is in [RFC 0001](rfcs/0001-attestation-model.md); storage lifetime is in [RFC 0002](rfcs/0002-state-archival-and-ttl.md).

## The attestation layer

An attestation is a statement made by an issuer about a subject under a registered schema. The registry supports single attestations and batches (one Merkle root covering many claims), revocation by the issuer, expiry, and TTL extension by anyone. See the crate docs in `contracts/attestation-registry/src/lib.rs` for the storage layout.

## Module dependency rule

Each module depends only on the shared attestation layer, never on another module. Every module must be usable and releasable on its own.

```
credentials ---+
               +--> attestation-registry <--+-- funding   (reads attestations for Sybil resistance)
reputation ----+                            +-- governance (reads attestations for voting eligibility/weight)
```

- **Credentials** and **reputation** _produce_ attestations.
- **Funding** and **governance** _consume_ attestations.

For example, funding may read a reputation attestation, but it does so through the registry, not by calling the reputation module.

## Phases

Work is strictly sequential by phase (see [ROADMAP.md](../ROADMAP.md)). Phase 0 is the registry, indexer and tooling; credentials, reputation, funding and governance follow in Phases 1 to 4.

## Testnet only

All deployment targets Stellar testnet. Contracts are unaudited. See [deploy-testnet.md](deploy-testnet.md).
