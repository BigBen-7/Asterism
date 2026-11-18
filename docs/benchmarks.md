# Registry resource benchmarks

Measures the CPU instructions and memory used by each `attestation-registry` operation and compares them with the network's per-transaction limits.

## Reproduce

```bash
just bench
```

This runs `contracts/attestation-registry/tests/bench.rs` in the Soroban test environment and prints a Markdown table (CPU instructions, memory bytes per operation). Paste the output into "Results" below.

## Network limits

Per-transaction limits as recorded in the Stellar docs when RFC 0002 was written ([Fees, resource limits, metering](https://developers.stellar.org/docs/learn/fundamentals/fees-resource-limits-metering)):

| Resource                   | Limit                           |
| -------------------------- | ------------------------------- |
| CPU instructions           | 400,000,000                     |
| Memory                     | 40 MiB                          |
| Footprint entries          | 400 (200 disk-read, 200 write)  |
| Serialized entry size      | 64 KiB (key max 250 bytes)      |

Validators can change these. Confirm the current values before relying on them:

```bash
stellar network settings --network testnet
```

## Ledger entries per operation

Derived from the storage layout in `contracts/attestation-registry/src/lib.rs` (reads, writes include the contract instance and its counters). These are counted from the code, not measured.

| Operation                                  | Persistent reads | Persistent writes | Instance writes |
| ------------------------------------------ | ---------------- | ----------------- | --------------- |
| `register_schema`                          | 0                | 1                 | 1 (counter)     |
| `get_schema`, `get`, `status`, `verify`    | 1                | 0                 | 0               |
| `issue`, `issue_batch`                     | 1 (schema)       | 1                 | 1 (counter)     |
| `revoke`                                   | 2 (attestation, schema) | 1          | 0               |
| `verify_inclusion`                         | 1                | 0                 | 0               |
| `extend_ttl`                               | 1 (existence)    | 0 (TTL bump only) | 0               |

All are far below the 200 / 200 entry limits.

## Results

Not yet recorded. Run `just bench` on a machine with the Rust toolchain and paste the table here, then flag any operation above 50% of the CPU or memory limit in the section below.

| Operation | CPU instructions | Memory bytes |
| --------- | ---------------- | ------------ |
| _pending_ |                  |              |

## Operations near a limit

_To be filled in from the results._ `verify_inclusion` at the maximum proof depth (`MAX_PROOF_LEN` = 32) is the operation most likely to be expensive, because it hashes once per level.

## Out of scope

Optimising the contract.
