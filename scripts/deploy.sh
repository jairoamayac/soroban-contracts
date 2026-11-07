#!/usr/bin/env bash
# Deploys one budget-treasury instance and records it in
# deployments/<network>.json.
#
# Usage:
#   scripts/deploy.sh <testnet|mainnet> \
#     --source <cli-identity> \
#     --admin <G...> --admin <G...> [--admin <G...>] \
#     --token <C...> --symbol <ASSET_CODE> \
#     [--name <manifest-key>]
#
# * <cli-identity> is a key in the Stellar CLI keystore of the person
#   deploying. Keys are never read from this repository.
# * The deployer does not become a member: only the --admin addresses do, and
#   at least two distinct admins are required.
# * --token is the asset contract the treasury holds (e.g. the USDC SAC) and
#   --symbol its code, which payment budget lines must use.
# * Mainnet needs CONFIRM_MAINNET=yes in the environment.
# * The working tree must be clean, so the manifest points at the exact source.
#
# An existing entry under the same name is moved to "retired".
set -euo pipefail
cd "$(dirname "$0")/.."

usage() { sed -n '2,22p' "$0" >&2; }

network="${1:-}"
shift || true
case "$network" in
  testnet | mainnet) ;;
  *) usage; exit 1 ;;
esac

source_id="" token="" symbol="" name="budget-treasury"
admins=()
while [ $# -gt 0 ]; do
  case "$1" in
    --source) source_id="$2"; shift 2 ;;
    --admin) admins+=("$2"); shift 2 ;;
    --token) token="$2"; shift 2 ;;
    --symbol) symbol="$2"; shift 2 ;;
    --name) name="$2"; shift 2 ;;
    *) echo "Unknown option: $1" >&2; usage; exit 1 ;;
  esac
done

if [ -z "$source_id" ] || [ -z "$token" ] || [ -z "$symbol" ] || [ "${#admins[@]}" -lt 2 ]; then
  usage
  exit 1
fi
if [ "$network" = mainnet ] && [ "${CONFIRM_MAINNET:-}" != yes ]; then
  echo "Refusing to deploy to mainnet without CONFIRM_MAINNET=yes." >&2
  exit 1
fi
if [ -n "$(git status --porcelain)" ]; then
  echo "The working tree has uncommitted changes. Commit first." >&2
  exit 1
fi

./scripts/build.sh
wasm=target/wasm32v1-none/release/budget_treasury.wasm
schema=$(stellar contract info meta --wasm "$wasm" 2>/dev/null | sed -n 's/.*event_schema: //p')
if [ -z "$schema" ]; then
  echo "The WASM does not declare event_schema in its contract meta." >&2
  exit 1
fi

wasm_hash=$(stellar contract upload --wasm "$wasm" \
  --source-account "$source_id" --network "$network")
admins_json="[$(printf '"%s",' "${admins[@]}" | sed 's/,$//')]"
contract_id=$(stellar contract deploy --wasm-hash "$wasm_hash" \
  --source-account "$source_id" --network "$network" \
  -- --admins "$admins_json" --token "$token" --token_symbol "$symbol")

python3 - "deployments/$network.json" "$name" <<PY
import json, sys
path, name = sys.argv[1], sys.argv[2]
with open(path) as f:
    manifest = json.load(f)
entry = {
    "contract_id": "$contract_id",
    "wasm_hash": "$wasm_hash",
    "source_commit": "$(git rev-parse --short HEAD)",
    "deployed_at": "$(date -u +%F)",
    "deployer": "$(stellar keys address "$source_id")",
    "admins": json.loads('$admins_json'),
    "token": "$token",
    "token_symbol": "$symbol",
    "event_schema": "$schema",
}
old = manifest["contracts"].get(name)
if old and old.get("contract_id"):
    old["superseded_by"] = entry["contract_id"]
    manifest.setdefault("retired", {})[f"{name}-{old.get('source_commit')}"] = old
manifest["contracts"][name] = entry
with open(path, "w") as f:
    json.dump(manifest, f, indent=2)
    f.write("\n")
PY

echo "Deployed $name to $network: $contract_id"
echo "Recorded in deployments/$network.json. Review and commit it."
