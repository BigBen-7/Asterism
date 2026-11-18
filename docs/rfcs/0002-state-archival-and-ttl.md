# RFC 0002: State archival and TTL policy

- Status: Proposed
- Issue: #36 (0.2)
- Related: RFC 0001 (data model), issue 0.8 (implementation)

## Summary

Soroban contract state expires unless its TTL (time to live, counted in ledgers) is extended. This RFC decides which registry data uses which storage type, who pays to keep records alive, and what happens to records that expire.

## Confirmed facts

Checked against the current Stellar developer docs ([State Archival](https://developers.stellar.org/docs/learn/fundamentals/contract-development/storage/state-archival), [Choosing the right storage type](https://developers.stellar.org/docs/build/guides/storage/choosing-the-right-storage), [Storage strategies in production contracts](https://developers.stellar.org/docs/build/guides/storage/storage-strategies), [Fees, resource limits, metering](https://developers.stellar.org/docs/learn/fundamentals/fees-resource-limits-metering)):

- There are three storage types: **temporary**, **persistent** and **instance**.
- Persistent and temporary entries each have their own TTL and must be extended individually. Instance storage is one ledger entry with one shared TTL; extending it also extends the contract instance itself.
- `extend_ttl(threshold, extend_to)` on persistent/temporary storage extends an entry to `extend_to` ledgers only if its remaining TTL is below `threshold`.
- On expiry, **persistent and instance entries are archived** (they can be restored); **temporary entries are deleted permanently**.
- Rent is charged when an entry is created or grows, and when its TTL is extended.
- TTL logic can be tested with the TTL getter plus ledger-sequence manipulation in the test environment.
- Mainnet per-transaction limits recorded in the docs at the time of writing: 400,000,000 CPU instructions, 40 MiB memory, 400 footprint entries (200 disk-read, 200 write entries), serialized contract-data entry max 64 KiB, key max 250 bytes. Validators can change these; re-check with `stellar network settings` before relying on them (feeds issue 0.9).

## Not yet confirmed (`needs-verification`)

These were not established by the sources above and must be confirmed (for example against the CAP text or by running on testnet) before issue 0.8 is merged:

1. Whether any account, not only the contract, can extend or restore a given entry's TTL via the footprint operations (this RFC assumes yes; see Decision 3).
2. The network's maximum TTL (the cap on `extend_to`) and the minimum TTL for new persistent entries.
3. Exact rent cost per entry size and ledger, to size the default TTL constants.

## Decisions

### 1. Storage layout

| Data                       | Storage type | Rationale                                                      |
| -------------------------- | ------------ | -------------------------------------------------------------- |
| Admin/config, id counters  | Instance     | Small, read on most calls, shared TTL extended on every write. |
| Schemas (by `u64` id)      | Persistent   | Must outlive any attestation that references them.             |
| Attestations (by `u64` id) | Persistent   | Long-lived records; archival is recoverable.                   |

Temporary storage is not used for registry records, because deletion is permanent and attestations must stay verifiable.

### 2. Who pays

- **Issuer pays on write.** Registering a schema or issuing an attestation sets the entry's TTL to a default (`DEFAULT_TTL_LEDGERS`) as part of the same transaction, so rent is paid by whoever created the record.
- **Reads do not extend TTL.** Read functions (`get`, `status`, `verify`) stay side-effect free so they can be simulated and called without fees.
- **Anyone may extend.** `extend_ttl(kind, id)` is a public, permissionless function, so an issuer, a subject, a verifier or a keeper service can pay to keep a record alive. It requires no authorization and only ever increases a TTL. (Depends on confirmed item 1 above for extension by third parties outside the contract.)
- **Instance TTL** is extended on every state-changing call using the same threshold and extension values.

### 3. Constants (initial, tunable after benchmarks)

Ledgers close roughly every 5 seconds, about 17,280 per day.

- `TTL_THRESHOLD`: 30 days (about 518,400 ledgers)
- `DEFAULT_TTL_LEDGERS` (extend-to): 180 days (about 3,110,400 ledgers)

These values are placeholders until confirmed item 2 (network maximum) and issue 0.9 (cost benchmarks) are done. They must be below the network maximum.

### 4. Expiry versus archival

Two different things, kept separate:

- **Logical expiry** (`expires_at` in RFC 0001) is a property of the claim. Expired attestations stay readable and report `Expired`.
- **Archival** is a storage event. Archived persistent entries are not lost; they can be restored by a restore operation (paid by whoever restores). Verifiers and the SDK must treat "archived" as "restore, then read", and the SDK should surface that clearly.

Records past logical expiry are not actively deleted. Keepers need not extend them, so they may archive naturally; if a verifier needs one later, it can restore it.

### 5. Indexer implication

The indexer persists every event from the first deployment (RPC event retention is limited; see issue 0.12). This means an archived record's history is still available off-chain even if the on-chain entry has to be restored.

## Implementation (issue 0.8)

- `extend_ttl(kind, id)` (`kind` is `Schema` or `Attestation`) and `extend_instance_ttl()` are public and need no authorization.
- Both use `extend_ttl(TTL_THRESHOLD, DEFAULT_TTL_LEDGERS)`: nothing happens while more than `TTL_THRESHOLD` ledgers remain, and otherwise the TTL is set to exactly `DEFAULT_TTL_LEDGERS`. A TTL is never shortened and never exceeds that bound.
- A missing record (never created, or archived) returns `SchemaNotFound` / `AttestationNotFound`. An archived record must be restored first.
- The constants are still the placeholders from Decision 3. Confirmed items 1 to 3 above remain open and `needs-verification`; the contract does not depend on them beyond `DEFAULT_TTL_LEDGERS` being at or below the network maximum.

## Alternatives considered

- **Contract pays all rent from a fee pool.** Rejected for Phase 0: adds fund-handling logic and an economic design the project does not need yet.
- **Temporary storage with periodic re-issuance.** Rejected: permanent deletion breaks revocation history.
- **Extending TTL on reads.** Rejected: turns every verification into a paid write.

## Impact on other issues

- 0.4, 0.5, 0.7: set `DEFAULT_TTL_LEDGERS` on creation.
- 0.8: implements `extend_ttl`, TTL tests, and resolves the unconfirmed items above.
- 0.9: records rent and resource costs and fixes the final constants.
