# Architecture

How the contracts in this repository fit with the Truway platform and with anyone who reads the
record. The decision behind this design is [ADR 0003](adr/0003-members-sign-proof-of-spend.md).

## Components

```
Organisation members (admins, approvers, field operators)
  │  each signs with their own key
  ▼
Truway platform (closed source)
  │  builds transactions, stores documents, maps hashes to names
  │  deploys contracts but is never a member
  ▼
budget-treasury  (one instance per organisation)        ── Proof of Spend events ──►  Readers
  members · budget lines · funds · proposals · receipts                               funders, auditors,
                                                                                      oversight bodies,
                                                                                      public explorer
```

| Component | What it does |
|---|---|
| [`budget-treasury`](../contracts/budget-treasury) | Reference implementation of [Proof of Spend](proof-of-spend.md). Holds an organisation's members and roles, budget lines per project, line and funding source, funds per project and source, proposals, and the supporting documents already used. Every change is proposed by one member and approved by another |
| [`truway-common`](../packages/common) | The storage TTL policy shared by every contract |
| Truway platform | Builds each transaction for the member who must sign it, stores supporting documents and computes their SHA-256 hash, maps hashed identifiers back to projects, payees and funders, and indexes events. It holds no role in any treasury |
| Readers | Read events and contract state from the network, or from an indexer, without an account |
| Legacy contracts | `fund-tracker`, `expense-anchor` and `sbt-badge` follow the server-attested model of [ADR 0002](adr/0002-server-attested-records.md), now superseded. They get no new deployments and will be removed once the platform no longer calls them |

## Trust model

**Who signs.** Each member signs their own proposals and approvals. The platform prepares the
transaction and the member's key signs it, through an embedded signer, so members never handle
seed phrases. On testnet, while per-person signing is being built, the platform holds demo keys
that stand in for demo members; they are never used on mainnet.

**What a record proves.**

- Two distinct members of the organisation's treasury proposed and approved it.
- It fits within the budget its funding source assigned to that line, and a payment within the
  funds that source holds.
- It books to the chart of accounts of its line.
- Its supporting document hashes to `receipt_hash`, and that document backs no other record in
  the same treasury.

**What it does not prove.**

- That an address belongs to a given person or organisation. Identity checks happen off the
  ledger.
- That a supporting document is genuine, or that an expense paid outside the ledger was really
  paid. `payment_ref` links to the external payment evidence for auditors.

**What each party can do.**

| Party | Can | Cannot |
|---|---|---|
| Admin | Propose anything; approve any proposal of another member | Approve their own proposal |
| Approver | Approve or reject budgets, expenses and payments | Propose; approve membership or code changes |
| Operator | Propose expenses and payments | Approve anything |
| Truway | Deploy a treasury for an organisation | Move funds, approve, change members or upgrade |
| Anyone | Fund a project; read every record | Change any record |

## Life of an expense paid outside the ledger

1. A field operator pays a supplier by bank transfer or cash and captures the invoice at the
   moment of payment. The platform stores the file and computes its SHA-256 hash.
2. The operator proposes `RecordExpense` with the line, funding source, amount, account code,
   `receipt_hash` and hashed references to the payment and the payee.
3. The contract checks the operator's role, that the document was never used, that the line
   exists in the same chart and that the amount fits in what is left.
4. A second member approves. The contract checks everything again, adds the amount to the line,
   marks the document as used and emits `expense_recorded` and `proposal_executed`.
5. The indexer stores the events. The public explorer shows the record in plain language, with
   links to the transaction on a block explorer and to the standard.
6. Anyone holding the invoice can hash it and compare the result with `receipt_hash`.

## Life of a payment on the ledger

1. A funder, or the organisation, calls `fund` to move the treasury asset (for example USDC) into
   the contract, earmarked for a project by a funding source. The contract emits
   `project_funded`.
2. A member proposes `Pay` with the destination address, the line, the source and the
   supporting document. On top of the expense checks, the line must be in the treasury asset and
   the source must hold enough funds for the project.
3. A second member approves. The contract transfers the amount to the destination, updates the
   line and the source's funds, and emits `payment_executed` and `proposal_executed`.

## Upgrades

An admin proposes `Upgrade(wasm_hash)` and another admin approves it. The contract emits
`contract_upgraded` and then replaces its code, so readers know that later events come from new
code. The constructor does not run again, and storage keys must stay compatible: variants of
`DataKey` are never renamed or removed without a migration.

## Limits and planned work

- Members are capped at 32 (`MAX_MEMBERS`), because they live in instance storage, which every
  call loads.
- Membership and code changes need one admin approver besides the proposer. Approval by several
  admins, a timelock on upgrades, proposal expiry and a pause are planned before mainnet.
- RPC providers keep events for about 7 days, so the platform runs an indexer. Contract state
  (`line`, `funds`, `receipt_use`, `proposal`) can always be read directly.
