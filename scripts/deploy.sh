#!/usr/bin/env bash
set -euo pipefail

NETWORK=${1:-testnet}
WASM=${2:-target/wasm32-unknown-unknown/release/depthdrip_contract.wasm}

echo "Deploying $WASM to $NETWORK"

if ! command -v soroban >/dev/null 2>&1; then
  echo "soroban CLI not found. Install from https://soroban.stellar.org/"
  exit 1
fi

# This script assumes soroban CLI is configured with the target network and an account.
soroban contract deploy --wasm $WASM --network $NETWORK
