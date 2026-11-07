# Deployments

One manifest per network. A manifest is the source of truth for which contract instances
belong to this repository.

| File | Network |
|---|---|
| [testnet.json](testnet.json) | Stellar testnet |
| [mainnet.json](mainnet.json) | Stellar mainnet (public network) |

Fields per contract instance:

| Field | Meaning |
|---|---|
| `contract_id` | Deployed contract address (`C...`) |
| `wasm_hash` | SHA-256 of the installed WASM; rebuild `source_commit` to check it |
| `source_commit` | Commit of this repository the WASM was built from |
| `deployed_at` | Deployment date (ISO 8601) |
| `deployer` | Account that paid for the deployment. It is not a member of the treasury |
| `admins` | Admin addresses passed to the constructor |
| `token`, `token_symbol` | Asset the treasury holds and its code |
| `event_schema` | Standard declared in the contract meta |
| `purpose`, `evidence` | Optional: why the instance exists, and transactions that show it working |

`scripts/deploy.sh` writes these fields and moves the previous instance under the same name to
`retired`, with `superseded_by`.

**Policy:** only instances deployed under Truway-controlled keys are listed. Instances under
`retired`, and any deployed before this repository existed, must not be used.
