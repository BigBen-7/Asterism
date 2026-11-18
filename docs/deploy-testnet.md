# Deploying the registry to Stellar testnet

Testnet only. The script refuses any other network, and mainnet is not supported.

## Confirmed endpoints

Checked against the live testnet RPC (`getNetwork`, `getHealth`) and the Stellar CLI 25.2.0:

| Item               | Value                                                                  |
| ------------------ | ---------------------------------------------------------------------- |
| RPC URL            | `https://soroban-testnet.stellar.org`                                  |
| Network passphrase | `Test SDF Network ; September 2015`                                    |
| Friendbot          | `https://friendbot.stellar.org` (the RPC reports it as `friendbotUrl`) |

The testnet RPC reported a `ledgerRetentionWindow` of 120960 ledgers (about 7 days) when checked. This is why the indexer must persist every event from the first deployment (CLAUDE.md section 6 item 3). Re-check before relying on it.

## Prerequisites

- Rust toolchain from `rust-toolchain.toml` (includes the `wasm32v1-none` target)
- [Stellar CLI](https://developers.stellar.org/docs/tools/cli) (`stellar`)

## Deploy

```bash
cp .env.example .env   # optional; the defaults already point at testnet
just deploy-testnet
```

The script (`scripts/deploy-testnet.sh`):

1. Refuses to run unless `STELLAR_NETWORK` is `testnet` and the passphrase is the testnet one.
2. Creates the identity named by `STELLAR_SOURCE_ACCOUNT` (default `asterism-deployer`) with `stellar keys generate --network testnet --fund` if it does not exist. Friendbot funds it. The secret key lives in the Stellar CLI's key store, never in this repository.
3. Builds the contract with `stellar contract build --package attestation-registry`.
4. Deploys `target/wasm32v1-none/release/attestation_registry.wasm` with `stellar contract deploy` and prints `Contract id: C...`.

## Configuration

All values are in `.env.example`. `.env` is git-ignored. Do not put secret keys in any file in this repository; use testnet identities only.

## Not covered

- Mainnet deployment.
- Contract upgrades and initialization beyond what the registry needs (it has no constructor arguments).
