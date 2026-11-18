#!/usr/bin/env bash
# Build the attestation registry and deploy it to Stellar TESTNET.
# Testnet only: any other network is refused. See docs/deploy-testnet.md.
set -euo pipefail

cd "$(dirname "$0")/.."

# Optional local overrides (never committed).
if [ -f .env ]; then
  set -a
  # shellcheck disable=SC1091
  . ./.env
  set +a
fi

NETWORK="${STELLAR_NETWORK:-testnet}"
RPC_URL="${STELLAR_RPC_URL:-https://soroban-testnet.stellar.org}"
PASSPHRASE="${STELLAR_NETWORK_PASSPHRASE:-Test SDF Network ; September 2015}"
SOURCE="${STELLAR_SOURCE_ACCOUNT:-asterism-deployer}"
WASM="target/wasm32v1-none/release/attestation_registry.wasm"

if [ "$NETWORK" != "testnet" ]; then
  echo "error: only testnet is supported (STELLAR_NETWORK=$NETWORK)" >&2
  exit 1
fi
if [ "$PASSPHRASE" != "Test SDF Network ; September 2015" ]; then
  echo "error: STELLAR_NETWORK_PASSPHRASE is not the testnet passphrase" >&2
  exit 1
fi

if ! command -v stellar >/dev/null 2>&1; then
  echo "error: the Stellar CLI ('stellar') is not installed" >&2
  exit 1
fi

# Create and fund the deployer identity on first use. The secret key stays in
# the Stellar CLI's own key store, not in this repository.
if ! stellar keys address "$SOURCE" >/dev/null 2>&1; then
  echo "Creating and funding testnet identity '$SOURCE' via friendbot..." >&2
  stellar keys generate "$SOURCE" --network testnet --fund
fi

echo "Building contract..." >&2
stellar contract build --package attestation-registry

echo "Deploying to testnet as '$SOURCE'..." >&2
CONTRACT_ID="$(stellar contract deploy \
  --wasm "$WASM" \
  --source-account "$SOURCE" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$PASSPHRASE")"

echo "Contract id: $CONTRACT_ID"
