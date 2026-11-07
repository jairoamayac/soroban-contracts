# budget-treasury

Reference implementation of [Proof of Spend](../../docs/proof-of-spend.md) (`proof-of-spend/1`),
declared in the contract meta as `event_schema`.

Budget execution as a contract. Budget lines are set per project, line and funding source, each with
an accounting code and a currency. Expenses paid outside the ledger are recorded against them, and
money held on the ledger is paid against them, always with a supporting document and a second
person's approval.

## Roles

| Role | Propose | Approve or reject |
|---|---|---|
| `Admin` | Any action | Any proposal |
| `Approver` | No | Budgets, expenses and payments |
| `Operator` | Expenses and payments | Only to withdraw their own proposal |

A proposal is never approved by the member who proposed it. Membership and code changes need an
admin approver. Truway deploys the contract but is not a member.

## Interface

| Function | What it does |
|---|---|
| `__constructor(admins, token, token_symbol)` | Creates the treasury with the organisation's admins and its asset |
| `fund(from, project, source, amount)` | Moves funds in, earmarked for a project by a funding source |
| `propose(proposer, action)` | `SetBudget`, `RecordExpense`, `Pay`, `AddMember`, `RemoveMember`, `Upgrade` |
| `approve(approver, id)` | Approves and executes a proposal |
| `reject(member, id)` | Rejects a pending proposal or withdraws one's own |
| `line(project, line, source)`, `funds(project, source)`, `receipt_use`, `proposal`, `role`, `members`, `token`, `token_symbol` | Reads |

## Rules enforced on chain

| Rule | Error |
|---|---|
| Only members act, each within their role | `Unauthorized` (1) |
| The approver is not the proposer | `SelfApproval` (2) |
| An expense or payment never exceeds what is left in its line for its funder | `BudgetExceeded` (5) |
| A payment never exceeds the funds its source holds for the project | `InsufficientFunds` (6) |
| A budget is never set below what was spent | `BudgetBelowPaid` (9) |
| An expense or payment books to the chart of its line | `ChartMismatch` (10) |
| The treasury keeps an admin and two members able to approve, also when a role changes | `QuorumLost` (11) |
| A supporting document backs only one expense or payment | `ReceiptAlreadyUsed` (13) |
| Every expense and payment has a supporting document | `ReceiptRequired` (14) |
| Payments use lines in the treasury asset | `CurrencyMismatch` (15) |

Error codes are part of the interface and are never renumbered.

## Known limitations (planned before mainnet)

Membership and code changes need one admin approver, not all of them; there is no key recovery, no
pause and no proposal expiry yet. See the threat model in the roadmap.

## Tests

`cargo test -p budget-treasury` covers every rule above, the address that authorises each call,
budgets per funder, expenses that move no funds, payments, roles and quorum.
