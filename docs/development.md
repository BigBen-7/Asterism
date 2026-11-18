# Local development

Build and test Asterism from a clean clone. Everything runs on testnet or locally; there is no Docker.

## Prerequisites

| Tool                                                                     | Used for                                                                                                                   |
| ------------------------------------------------------------------------ | -------------------------------------------------------------------------------------------------------------------------- |
| Rust, via `rustup`                                                       | Contracts and services. `rust-toolchain.toml` pins the version and adds the `wasm32v1-none` target and `rustfmt`/`clippy`. |
| [Stellar CLI](https://developers.stellar.org/docs/tools/cli) (`stellar`) | Building contracts (`stellar contract build`) and deploying to testnet.                                                    |
| Node 22+ and [pnpm](https://pnpm.io)                                     | TypeScript packages and apps (`packageManager` is pinned in `package.json`).                                               |
| [just](https://github.com/casey/just)                                    | The single command runner for the repo.                                                                                    |
| PostgreSQL (native, running locally)                                     | Indexer development and tests. `psql`, `createdb`, `pg_isready` must be on your `PATH`.                                    |

## First run

```bash
git clone https://github.com/Chain-Explain/Asterism.git
cd Asterism
cp .env.example .env     # optional; never commit .env
just setup               # shows the active Rust toolchain and runs `pnpm install`
just build
just test
just lint
```

`just` with no arguments lists every recipe.

## Common commands

| Command               | What it does                                                                                     |
| --------------------- | ------------------------------------------------------------------------------------------------ |
| `just setup`          | Shows the pinned Rust toolchain and installs Node dependencies.                                  |
| `just build`          | Builds the contracts (`stellar contract build`), the Rust workspace and the TS packages.         |
| `just test`           | Runs `cargo test --workspace` and the TypeScript tests.                                          |
| `just lint`           | `cargo fmt --check`, `cargo clippy -D warnings`, ESLint, Prettier and type checks.               |
| `just fmt`            | Formats Rust and TypeScript.                                                                     |
| `just db-up`          | Creates the local `asterism` database (Postgres must already be running).                        |
| `just db-down`        | Drops the local `asterism` database.                                                             |
| `just bench`          | Prints per-operation contract resource usage (see [benchmarks.md](benchmarks.md)).               |
| `just deploy-testnet` | Builds and deploys the registry to Stellar testnet (see [deploy-testnet.md](deploy-testnet.md)). |

## Database and the indexer

The indexer's integration tests need a running Postgres and `DATABASE_URL` (the default in `.env.example` is `postgres://localhost:5432/asterism`):

```bash
just db-up
export DATABASE_URL=postgres://localhost:5432/asterism
cargo test -p asterism-indexer
```

The tests create and drop throwaway databases, so they do not touch your `asterism` database. See [services/indexer/README.md](../services/indexer/README.md) to run the indexer itself.

## Before opening a PR

Run `just lint` and `just test`, follow [CONTRIBUTING.md](../CONTRIBUTING.md), and link the issue with `Closes #N`. Use testnet keys only and never commit secrets.
