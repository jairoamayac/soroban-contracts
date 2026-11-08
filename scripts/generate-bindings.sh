#!/usr/bin/env bash
# Generates the TypeScript client for budget-treasury from the release WASM
# into bindings/typescript/budget-treasury.
#
# Usage: scripts/generate-bindings.sh
#
# The client is network-agnostic: pass the contract id, RPC URL and network
# passphrase when you create it (see deployments/<network>.json).
# Never edit the generated files by hand; run this script again instead.
set -euo pipefail
cd "$(dirname "$0")/.."

./scripts/build.sh > /dev/null

out=bindings/typescript/budget-treasury
stellar contract bindings typescript \
  --wasm target/wasm32v1-none/release/budget_treasury.wasm \
  --output-dir "$out" \
  --overwrite

echo "Generated $out from commit $(git rev-parse --short HEAD)."
