# Storage and TTL

## Policy

Defined once in [`packages/common/src/ttl.rs`](../packages/common/src/ttl.rs) and applied by every
contract:

| Constant | Value |
|---|---|
| `BUMP_THRESHOLD` | 30 days (518,400 ledgers) |
| `BUMP_TO` | 120 days (2,073,600 ledgers) |

Every write extends the entry it touches and the contract instance, which also extends the
contract code. Reads extend nothing.

## Layout of `budget-treasury`

Keys are the variants of `DataKey` in
[`storage.rs`](../contracts/budget-treasury/src/storage.rs).

| Key | Storage | Value | Why there |
|---|---|---|---|
| `Token`, `TokenSymbol` | instance | Treasury asset and its code | Read by every payment |
| `Members` | instance | `Map<Address, Role>`, at most 32 entries | Read by every call; the cap keeps the instance small |
| `NextId` | instance | Next proposal id | Read by every proposal |
| `Proposal(id)` | persistent | `Proposal` | One entry per proposal, kept as the approval record |
| `Line(LineKey)` | persistent | `BudgetLine` | One entry per project, line and funding source |
| `Funds(FundKey)` | persistent | `i128` | One entry per project and funding source |
| `Receipt(hash)` | persistent | Proposal id that used the document | Enforces single use of each supporting document |

## Entries that are not written again

Persistent entries that nobody writes for 120 days are archived, not deleted. Since protocol 23
an archived entry is restored automatically when a transaction needs it: simulation adds the
restore to the footprint and the caller pays the rent. An archived persistent entry is never
read as absent, so a used supporting document cannot be reused after its `Receipt` entry is
archived. To keep entries live without
writing, extend them with `stellar contract extend --key-xdr <KEY> --durability persistent`.

Events are not storage: RPC providers keep them for about 7 days. History lives in the indexer,
and current state can always be read from the contract.

## Changing storage keys

Variant names and their field types are part of the on-ledger encoding. In a deployed contract,
never rename, reorder the fields of, or remove a `DataKey` variant without a migration that runs
in the upgraded code. Add new variants instead.
