# Events

## `budget-treasury`

The events of `budget-treasury` are specified by the open standard
[Proof of Spend](proof-of-spend.md), section 5: names, topics, data fields, units and when each
one is emitted. The tests `events_follow_the_standard` and `governance_events_follow_the_standard`
in [`test.rs`](../contracts/budget-treasury/src/test.rs) check the emitted events against that
section.

| Group | Events |
|---|---|
| Execution | `project_funded`, `budget_set`, `expense_recorded`, `payment_executed` |
| Governance | `proposal_created`, `proposal_executed`, `proposal_rejected`, `member_added`, `member_removed`, `contract_upgraded` |

Readers must ignore events from contracts whose meta does not declare
`event_schema = proof-of-spend/1`.

## Legacy contracts

Superseded by [ADR 0003](adr/0003-members-sign-proof-of-spend.md). Listed for readers of
existing records only.

| Contract | Event | Topics |
|---|---|---|
| `fund-tracker` | `FundAllocated` | `("fund_allocated", project_id)` |
| `expense-anchor` | `ExpenseAnchored` | `("expense_anchored", project_id)` |
| `sbt-badge` | `BadgeMinted` | `("badge_minted", badge_type)` |
| `sbt-badge` | `BadgeRevoked` | `("badge_revoked",)` |
