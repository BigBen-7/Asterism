# Asterism

An open-source public-goods stack on the **Stellar network** (Soroban smart contracts, Rust), built around one shared **attestation registry**.

> **Status: Phase 0 (Foundation). Testnet only. Unaudited.**
> Nothing here is audited. Modules that hold or move funds (funding, governance) must not be used with real value.

Phase 0 builds the shared foundation: the attestation registry contract, the Postgres-backed event indexer, deployment tooling and docs. The four modules below are not started; each begins only after the previous phase is released. See [ROADMAP.md](ROADMAP.md).

![CI Rust](https://img.shields.io/badge/ci--rust-pending-lightgrey) ![CI TS](https://img.shields.io/badge/ci--ts-pending-lightgrey) ![License: MIT](https://img.shields.io/badge/license-MIT-blue)

## Modules

| #   | Module            | Role                                                                                              | Phase |
| --- | ----------------- | ------------------------------------------------------------------------------------------------- | ----- |
| 1   | Credentials       | Issue and verify tamper-proof credentials, with revocation. Produces attestations.                | 1     |
| 2   | Reputation        | Turn open-source contribution activity into non-transferable attestations. Produces attestations. | 2     |
| 3   | Quadratic funding | Round-based grant funding with quadratic matching and Sybil resistance. Consumes attestations.    | 3     |
| 4   | Governance        | Proposals, voting, timelock, optional gasless voting. Consumes attestations.                      | 4     |

Each module depends only on the attestation registry, never on another module. See [docs/architecture.md](docs/architecture.md) and [ROADMAP.md](ROADMAP.md).

## Quickstart

Prerequisites: Rust (see `rust-toolchain.toml`), Stellar CLI, Node 22+, pnpm, [just](https://github.com/casey/just), and a local PostgreSQL. Full instructions are in [docs/development.md](docs/development.md).

```bash
just setup
just lint
just test
```

## Documentation

- [Architecture](docs/architecture.md)
- [Local development](docs/development.md)
- [Credentials quickstart](docs/credentials-quickstart.md)
- [Credentials API](docs/api.md)
- [Deploying to testnet](docs/deploy-testnet.md)
- [Contract benchmarks](docs/benchmarks.md)
- [Design RFCs](docs/rfcs/README.md)

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). Report vulnerabilities as described in [SECURITY.md](SECURITY.md).

## License

[MIT](LICENSE)
