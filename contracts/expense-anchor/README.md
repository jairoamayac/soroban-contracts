# expense-anchor

> **Legacy.** Superseded by [`budget-treasury`](../budget-treasury) and
> [ADR 0003](../../docs/adr/0003-members-sign-proof-of-spend.md). No new deployments. Kept until
> the Truway platform no longer calls it.

Anchors approved expenses and donor reports as immutable records. Each record carries the
SHA-256 hash of its receipt, so anyone holding the file can prove it has not been altered.

## Interface

| Function | Auth | Description |
|---|---|---|
| `initialize(admin)` | `admin` | Sets the admin. Fails with `already_initialized` on a second call. |
| `anchor(caller, expense_id, project_id, amount_xlm, receipt_hash)` | admin | Anchors an expense. `caller` must be the admin. |
| `get_expense(expense_id) -> Option<AnchoredExpense>` | none | Reads an anchored expense. |
| `upgrade(new_wasm_hash)` | admin | Replaces the code in place, keeping the contract ID and storage. |

## Storage

| Key | Storage | Value |
|---|---|---|
| `Admin` | instance | `Address` |
| `Expense(expense_id)` | persistent | `AnchoredExpense { expense_id, project_id, submitted_by, amount_xlm, receipt_hash, anchored_at }` |

- `submitted_by` is the attesting signer (the admin). See
  [ADR 0002](../../docs/adr/0002-server-attested-records.md).
- `receipt_hash` is the SHA-256 digest of the receipt file, exactly 32 bytes.
- `amount_xlm` is in stroops. Zero is accepted: reports with no spending are anchored with 0.

## Events

| Event | Topics | Data |
|---|---|---|
| `ExpenseAnchored` | `("expense_anchored", project_id)` | `expense_id`, `submitted_by`, `amount_xlm`, `receipt_hash`, `anchored_at` |

## Errors

| Panic message | When |
|---|---|
| `already_initialized` | `initialize` called twice |
| `not_initialized` | Any write before `initialize` |
| `not_admin` | `caller` is not the admin |
| `invalid_amount` | `amount_xlm < 0` |
| `invalid_receipt_hash` | `receipt_hash` is not 32 bytes |
| `expense_already_exists` | `expense_id` is already anchored with different data |

## Notes

- Records are immutable. Anchoring the same `expense_id` again with identical data is a no-op
  that emits no event, so retries are safe.
- `expense_id` and `project_id` are `Symbol`s built from the last 12 characters of a UUID. A
  collision fails instead of overwriting (audit finding M-07).
