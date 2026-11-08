# Proof of Spend

**An open standard for budget execution on Stellar.**

**Identifier:** `proof-of-spend/1` · **Status:** Draft 2 · **Date:** 2026-11-07 · **License:** Apache-2.0 ·
**Reference implementation:** [`contracts/budget-treasury`](../contracts/budget-treasury) · **Authors:** Truway

A payment on Stellar proves that money moved. Proof of Spend proves what it bought: against which
approved budget line, from which funder, booked to which account, backed by which document, and
approved by whom. Any contract can implement it. Anyone can read it from the ledger and rebuild a
budget execution report or a general ledger without access to the organisation's database or its
software vendor.

## 1. Why a standard, not another platform

Public budgets and restricted grants are executed by many organisations on many systems. Funders,
auditors, oversight bodies and citizens cannot compare or verify execution without asking each of
those systems. In October 2026, public officials at a regional workshop for Colombia's National
Development Plan asked for "an interoperable system that links the formulation, budget allocation,
contracting, execution and evaluation of projects funded with public resources". Interoperability
starts with a shared, public format. Proof of Spend is that format for the execution step.

## 2. Scope

**Covered:** budget lines per project and funding source, each with an accounting code and a
currency; expenses paid outside the ledger and payments made on it, each with its accounting code
and the hash of its supporting document; the two-person approval behind every change; money held on
the ledger, earmarked by funding source.

**Not covered yet:** procurement and contracting, payments to individual beneficiaries (which need
selective disclosure), milestones and impact.

## 3. Conformance

A contract conforms to `proof-of-spend/1` when it:

1. declares the contract meta entry `event_schema = proof-of-spend/1`;
2. emits the events in section 5 with exactly these names, topics and fields;
3. enforces every invariant in section 6.

Readers must ignore events from contracts that do not declare the meta entry.

## 4. Identifiers and units

| Field | Type | Meaning |
|---|---|---|
| `project`, `line`, `source` | `BytesN<32>` | SHA-256 of an opaque identifier chosen by the organisation (for example a UUID). A budget line is identified by `(project, line, source)` |
| `reference`, `payment_ref`, `payee_ref` | `BytesN<32>` | SHA-256 of an off-chain reference: the payment's id, the external payment evidence, the payee's tax id. Never the identifier itself |
| `chart` | `Symbol` | Chart of accounts, from the registry in section 8 |
| `account` | `Symbol` | Account code within `chart` |
| `currency` | `Symbol` | Currency of the budget line, e.g. `COP` or `USDC` |
| `amount`, `budget`, `paid`, `funds` | `i128` | Fixed point with 7 decimals in the line's currency: 45,000 COP is `450000000000` |
| `receipt_hash` | `BytesN<32>` | SHA-256 of the supporting document's bytes. Required |
| `proposal_id`, `id` | `u64` | Proposal that authorised the change |
| `proposer`, `approver`, `to`, `from` | `Address` | Stellar accounts or contracts |

## 5. Events

Encoding follows Soroban's `#[contractevent]` default: topics are the event name as a `Symbol`
followed by the fields marked as topics; data is a map of the remaining fields keyed by name.

### Execution

| Event | Topics | Data | Emitted when |
|---|---|---|---|
| `budget_set` | `project` | `line`, `source`, `chart`, `account`, `currency`, `budget`, `proposal_id`, `proposer`, `approver` | An approved proposal creates or changes a budget line |
| `expense_recorded` | `project` | `line`, `source`, `proposal_id`, `amount`, `currency`, `chart`, `account`, `receipt_hash`, `payment_ref`, `payee_ref`, `proposer`, `approver`, `line_paid`, `line_budget` | An approved expense paid outside the ledger (bank transfer, cash) is recorded. No funds move |
| `payment_executed` | `project` | `line`, `source`, `proposal_id`, `to`, `amount`, `currency`, `chart`, `account`, `receipt_hash`, `reference`, `proposer`, `approver`, `line_paid`, `line_budget`, `source_funds` | An approved payment leaves the treasury on the ledger |
| `project_funded` | `project` | `source`, `from`, `amount`, `funds` | Money enters the treasury, earmarked for a project by a source |

### Governance

| Event | Topics | Data | Emitted when |
|---|---|---|---|
| `proposal_created` | `id` | `proposer`, `kind` (`budget`, `expense`, `pay`, `add_mbr`, `rm_mbr`, `upgrade`) | A member opens a proposal |
| `proposal_executed` | `id` | `kind`, `proposer`, `approver` | Any proposal is approved and executed |
| `proposal_rejected` | `id` | `rejected_by` | A proposal is rejected or withdrawn |
| `member_added` | `member` | `role` | A member is added or their role changes |
| `member_removed` | `member` | | A member is removed |
| `contract_upgraded` | `proposal_id` | `wasm_hash`, `proposer`, `approver` | Right before the code is replaced: readers must treat later events as coming from new code |

## 6. Invariants

A conforming contract MUST guarantee:

1. Every budget change, expense and payment was proposed by one member and approved by a different one.
2. Membership and code changes are approved by an admin.
3. For every line `(project, line, source)`, `paid <= budget` at all times, and a budget is never set below what was paid.
4. An expense or payment is charged to exactly one line, so money from one source never pays for another.
5. Every expense and payment carries a non-zero `receipt_hash`, and a `receipt_hash` backs at most one executed expense or payment per contract.
6. An expense or payment books to the `chart` of its line; a payment's line is in the treasury asset, and never exceeds the funds its source holds for the project.
7. The treasury always keeps at least one admin and two members able to approve.
8. Every amount is greater than zero.

"Two members" means two distinct addresses. Binding addresses to real people (identity checks) is the
implementer's responsibility and outside the ledger.

## 7. Reading the record

### Retention

Stellar RPC providers keep events for a limited window (about 7 days by default). Readers that need
history must ingest events as they are published, or read them from a historical archive. The
current state of every line can always be read from the contract (`line`, `funds`).

### Budget execution report

For a project and period: the latest `budget_set` per `(line, source)` gives the budget; the sum of
`expense_recorded.amount` and `payment_executed.amount` per `(line, source)` gives what was spent;
`budget - paid` is what remains. Grouping by `source` gives the report each funder asks for.
`line_paid` and `line_budget` in every expense and payment let a reader check running totals without
replaying history.

### General ledger

Each `expense_recorded` and `payment_executed` is one journal entry: debit `account` in `chart`;
credit the account where the organisation booked the money (bank, cash or the treasury asset); the
support is `receipt_hash`. Each `project_funded` debits the treasury asset and credits the liability
or income account set by the organisation's policy. Mapping hashes to names and tax ids, and
accounts to local statements, happens off chain.

### Verifying one record

Find the event by transaction hash or reference, compute the SHA-256 of the supporting document, and
compare it with `receipt_hash`. No account and no permission are needed.

## 8. Chart registry (draft)

| `chart` | Chart of accounts |
|---|---|
| `CO_PUC` | Colombia, Plan Único de Cuentas |
| `CO_CGN` | Colombia, public sector Catálogo General de Cuentas (reserved) |
| `PE_PCGE` | Peru, Plan Contable General Empresarial |
| `MX_SAT` | Mexico, código agrupador del SAT |
| `BR_SPED` | Brazil, plano de contas referencial |
| `SV_NIIF` | El Salvador, entity chart under IFRS for SMEs |
| `IFRS` | Generic IFRS classification |

To add a chart, open a pull request against this file.

## 9. Privacy

Events never carry personal data. Payees appear only as hashes or addresses; names, tax ids and
documents stay with the organisation, which decides what to publish. Payments to individual
beneficiaries need selective disclosure and are out of scope for version 1.

## 10. Versioning

Breaking changes produce a new major identifier (`proof-of-spend/2`). New information is added as
new events, never by changing the fields of an existing one. Error codes of the reference
implementation are never renumbered.

## Changelog

- **Draft 2 (2026-11-07):** renamed from Budget Execution Events (`truway-budget-execution/1`) to
  Proof of Spend (`proof-of-spend/1`); budget lines per funding source with a currency;
  `expense_recorded` for expenses paid outside the ledger; `proposal_executed` and
  `contract_upgraded`; required supporting document; admin approval for membership and code changes.
- **Draft 1 (2026-11-07):** first version.
