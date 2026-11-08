# Bindings

Generated TypeScript client for `budget-treasury`, in
[`typescript/budget-treasury`](typescript/budget-treasury). It is produced from the release WASM
by `scripts/generate-bindings.sh` (or `make bindings`) and must never be edited by hand. CI fails
if it is out of date.

The client is network-agnostic. Pass the contract id, RPC URL and network passphrase from
[`deployments/`](../deployments/) when you create it:

```ts
import { Client } from "budget-treasury";

const treasury = new Client({
  contractId: "C...",
  rpcUrl: "https://soroban-testnet.stellar.org",
  networkPassphrase: "Test SDF Network ; September 2015",
});
// Reads are simulated, so they need no signature.
const { result } = await treasury.line({ project, line: lineId, source });
```

The legacy contracts have no client.
