set shell := ["bash", "-euo", "pipefail", "-c"]

db_name := "asterism"

default:
    @just --list

# Install dependencies and toolchain pieces
setup:
    rustup show active-toolchain
    pnpm install

build: contracts-build
    cargo build --workspace
    pnpm build

test: contracts-test
    pnpm test

lint:
    cargo fmt --all --check
    cargo clippy --workspace --all-targets -- -D warnings
    pnpm lint
    pnpm typecheck

fmt:
    cargo fmt --all
    pnpm fmt

contracts-build:
    stellar contract build

contracts-test:
    cargo test --workspace

# Print per-operation resource usage of the registry as a Markdown table (see docs/benchmarks.md)
bench:
    cargo test -p attestation-registry --test bench -- --ignored --nocapture

# Build and deploy the registry to Stellar TESTNET (never mainnet); see docs/deploy-testnet.md
deploy-testnet:
    scripts/deploy-testnet.sh

# Create the local dev database on the native Postgres (starts nothing; Postgres must be running)
db-up:
    pg_isready -q
    psql -lqt | cut -d'|' -f1 | grep -qw {{db_name}} || createdb {{db_name}}
    @echo "Database '{{db_name}}' ready"

# Drop the local dev database
db-down:
    dropdb --if-exists {{db_name}}
