# fund-tracker

> **Legacy.** Superseded by [`budget-treasury`](../budget-treasury) and
> [ADR 0003](../../docs/adr/0003-members-sign-proof-of-spend.md). No new deployments. Kept until
> the Truway platform no longer calls it.

Records the budget allocated to each project. Every allocation is an attested, timestamped
entry written by the contract admin.

## Interface

| Function | Auth | Description |
|---|---|---|
| `initialize(admin)` | `admin` | Sets the admin. Fails with `already_initialized` on a second call. |
| `allocate(caller, project_id, amount_xlm)` | admin | Records or replaces the allocation of `project_id`. `caller` must be the admin. |
| `get_allocation(project_id) -> Option<FundAllocation>` | none | Reads an allocation. |
| `upgrade(new_wasm_hash)` | admin | Replaces the code in place, keeping the contract ID and storage. |

## Storage

| Key | Storage | Value |
|---|---|---|
| `Admin` | instance | `Address` |
| `Allocation(project_id)` | persistent | `FundAllocation { project_id, organization, amount_xlm, allocated_at }` |

- `organization` is the attesting signer (the admin), not the foundation. See
  [ADR 0002](../../docs/adr/0002-server-attested-records.md).
- `amount_xlm` is in stroops (1 unit = 10,000,000 stroops).
- `allocated_at` is the ledger timestamp.

## Events

| Event | Topics | Data |
|---|---|---|
| `FundAllocated` | `("fund_allocated", project_id)` | `organization`, `amount_xlm`, `allocated_at` |

## Errors

| Panic message | When |
|---|---|
| `already_initialized` | `initialize` called twice |
| `not_initialized` | Any write before `initialize` |
| `not_admin` | `caller` is not the admin |
| `invalid_amount` | `amount_xlm <= 0` |

## Notes

- Re-allocating a project overwrites the previous value and emits a new event, so budget
  changes stay visible in the event history.
- `project_id` is a `Symbol` built from the last 12 characters of the project UUID. Moving to
  full UUIDs is on the roadmap (audit finding M-07).
