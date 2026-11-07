# Budget Execution Events

**Schema identifier:** `truway-budget-execution/1` · **Status:** Draft 1 · **Date:** 2026-11-07 ·
**License:** Apache-2.0 · **Reference implementation:** [`contracts/budget-treasury`](../contracts/budget-treasury)

An open event schema for executing earmarked money on Stellar: funding by source, approved budget
lines, and payments that carry their accounting classification and proof of spend. Any contract can
implement it. Anyone can read it from the ledger and rebuild a budget execution report or a general
ledger without access to the organisation's database or its software vendor.

## 1. Why a schema, not another platform

Public budgets and restricted grants are executed by many organisations on many systems. Funders,
auditors, oversight bodies and citizens cannot compare or verify execution without asking each of
those systems. In October 2026, public officials at a regional workshop for Colombia's National
Development Plan asked for "an interoperable system that links the formulation, budget allocation,
contracting, execution and evaluation of projects funded with public resources". Interoperability
starts with a shared, public format. This document is that format for the execution step; the
contract in this repository is its first implementation.

## 2. Scope

**Covered:** money earmarked by funding source, approved budget lines with an accounting code,
payments against those lines with their accounting code and the hash of their supporting document,
and the two-person approval behind every change.

**Not covered yet:** procurement and contracting, payments to individual beneficiaries (which need
selective disclosure), milestones and impact. These are candidates for later versions.

## 3. Conformance

A contract conforms to `truway-budget-execution/1` when it:

1. declares the contract meta entry `event_schema = truway-budget-execution/1`;
2. emits the events in section 5 with exactly these names, topics and fields;
3. enforces every invariant in section 6.

Readers must ignore events from contracts that do not declare the meta entry.

## 4. Identifiers and units

| Field | Type | Meaning |
|---|---|---|
| `project`, `line`, `source`, `reference` | `BytesN<32>` | SHA-256 of an opaque identifier chosen by the organisation (for example a UUID). Never personal data |
| `chart` | `Symbol` | Chart of accounts, from the registry in section 8 |
| `account` | `Symbol` | Account code within `chart` |
| `amount`, `budget`, `paid`, `funds` | `i128` | Smallest unit of the contract's asset (`token()`); 7 decimals for Stellar assets |
| `receipt_hash` | `BytesN<32>` | SHA-256 of the supporting document's bytes. All zeros means "support pending" |
| `proposal_id` | `u64` | Proposal that authorised the change |
| `proposer`, `approver`, `to`, `from` | `Address` | Stellar accounts or contracts |

## 5. Events

Encoding follows Soroban's `#[contractevent]` default: topics are the event name as a `Symbol`
followed by the fields marked as topics; data is a map of the remaining fields keyed by name.

### `project_funded`

Emitted when money enters, earmarked for a project by a funding source.

| Topics | Data |
|---|---|
| `project` | `source`, `from`, `amount`, `funds` (what this source now holds for this project) |

### `budget_set`

Emitted when an approved proposal creates or changes a budget line.

| Topics | Data |
|---|---|
| `project` | `line`, `chart`, `account`, `budget`, `proposal_id`, `proposer`, `approver` |

### `payment_executed`

Emitted when an approved payment leaves the treasury. It carries everything an accountant needs to
book it and a funder needs to trace it.

| Topics | Data |
|---|---|
| `project` | `line`, `source`, `proposal_id`, `to`, `amount`, `chart`, `account`, `receipt_hash`, `reference`, `proposer`, `approver`, `line_paid`, `line_budget`, `source_funds` |

### `proposal_created`, `proposal_rejected`

| Event | Topics | Data |
|---|---|---|
| `proposal_created` | `id` | `proposer`, `kind` (`budget`, `pay`, `add_mbr`, `rm_mbr`, `upgrade`) |
| `proposal_rejected` | `id` | `rejected_by` |

### `member_added`, `member_removed`

| Event | Topics | Data |
|---|---|---|
| `member_added` | `member` | `role` |
| `member_removed` | `member` | |

## 6. Invariants

A conforming contract MUST guarantee:

1. Every budget change and every payment was proposed by one member and approved by a different one.
2. For every line, `paid <= budget` at all times, and a budget is never set below what was paid.
3. For every `(project, source)`, funds never go below zero, and each payment debits exactly one source.
4. A non-zero `receipt_hash` backs at most one executed payment per contract.
5. A payment's `chart` equals the `chart` of its budget line.
6. Every amount is greater than zero.

## 7. Rebuilding reports from events

### Budget execution report

For a project and period: the latest `budget_set` per line gives the budget; the sum of
`payment_executed.amount` per line gives what was paid; `budget - paid` is what remains. Grouping
the same payments by `source` gives the report each funder asks for. `line_paid` and `line_budget`
in every payment let a reader check its running totals without replaying history.

### General ledger

Each `payment_executed` is one journal entry: debit `account` in `chart`; credit the account where
the organisation books the treasury asset; the third party is `to`; the support is `receipt_hash`.
Each `project_funded` debits the treasury asset and credits the liability or income account set by
the organisation's accounting policy. Mapping addresses to names and tax ids, and accounts to local
statements, happens off chain in a chart pack.

### Verifying one payment

Find the `payment_executed` event by transaction hash or `reference`, compute the SHA-256 of the
supporting document, and compare it with `receipt_hash`. No account and no permission are needed.

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

Events never carry personal data. Recipients appear as addresses; names, tax ids and documents stay
with the organisation, which decides what to publish. Payments to individual beneficiaries need
selective disclosure and are out of scope for version 1.

## 10. Versioning

Breaking changes produce a new major identifier (`truway-budget-execution/2`). New information is
added as new events, never by changing the fields of an existing one. Error codes of the reference
implementation are never renumbered.
