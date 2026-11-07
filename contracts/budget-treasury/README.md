# budget-treasury

Reference implementation of the open [Budget Execution Events](../../docs/budget-execution-events.md)
schema (`truway-budget-execution/1`), declared in the contract meta as `event_schema`.

Budget execution as a contract. An organisation's funds sit in this contract and leave it only
as payments against an approved budget line, approved by a second person, with the accounting
code and the hash of the supporting document on every payment.

Nothing in the contract is specific to one country: charts of accounts are plain symbols
(`CO_PUC`, `IFRS`, `PE_PCGE`, ...) and the treasury asset is any SEP-41 token (USDC on Stellar
in production).

## Roles

| Role | Propose | Approve or reject |
|---|---|---|
| `Admin` | Any action | Yes |
| `Approver` | No | Yes |
| `Operator` | Payments only | Only to withdraw their own proposal |

A proposal is never approved by the member who proposed it. Truway deploys the contract but is
not a member: it cannot move funds, change budgets or upgrade the code.

## Interface

| Function | What it does |
|---|---|
| `__constructor(admins, token)` | Creates the treasury with the organisation's admins and its asset |
| `fund(from, project, source, amount)` | Moves funds in, earmarked for a project by a funding source |
| `propose(proposer, action)` | Opens a proposal: `SetBudget`, `Pay`, `AddMember`, `RemoveMember`, `Upgrade` |
| `approve(approver, id)` | Approves and executes a proposal |
| `reject(member, id)` | Rejects a pending proposal |
| `line`, `funds(project, source)`, `receipt_use`, `proposal`, `role`, `members`, `token` | Reads |

## Rules enforced on chain

| Rule | Error |
|---|---|
| The approver is not the proposer | `SelfApproval` (2) |
| A payment never exceeds what is left in its budget line | `BudgetExceeded` (5) |
| A payment never exceeds what its funding source earmarked for the project | `InsufficientFunds` (6) |
| A supporting document backs only one executed payment | `ReceiptAlreadyUsed` (13) |
| A payment books to the chart of its budget line | `ChartMismatch` (10) |
| A budget is never set below what was already paid | `BudgetBelowPaid` (9) |
| The treasury always keeps an admin and two members able to approve | `QuorumLost` (11) |
| Only members act, each within their role | `Unauthorized` (1) |

Error codes are part of the interface and are never renumbered.

## Events

`PaymentExecuted` carries project, line, funding source, recipient, amount, chart, account, receipt hash,
reference, proposer, approver, the line's paid and budget totals and what is left of the source. Together with `BudgetSet`
and `ProjectFunded` it is enough to rebuild the funder's execution report and the general ledger
without trusting Truway's database.

## Tests

`cargo test -p budget-treasury` covers every rule above, funding by source, payments within budget,
concurrent proposals that only exceed the line together, roles and quorum.
