# Internal review: budget-treasury

| | |
|---|---|
| **Date** | 2026-11-07 |
| **Scope** | `contracts/budget-treasury` v0.3 (commit `ca2e196`), `packages/common`, build and CI |
| **Fixes** | v0.4, same date |
| **Type** | Internal code and architecture review. Not an independent audit. |

## Summary

`budget-treasury` v0.3 enforced its main invariants: two distinct signers on every change,
budgets per line and funding source, single use of supporting documents, and quorum on
membership changes. The review found one high severity issue at construction, an unbounded
member list, validation that ran only at execution for membership changes, and no test that the
emitted events match the standard. All four are fixed in v0.4. Four design limits remain open and
are planned before mainnet.

The testnet demo instance (`CAZUXWWW…P3YJF`, v0.3) was deployed with two distinct admins, so
BT-01 does not affect it.

## Findings

| ID | Severity | Status | Finding |
|---|---|---|---|
| BT-01 | High | Fixed | A treasury could be created locked |
| BT-02 | Medium | Fixed | Unbounded member list in instance storage |
| BT-03 | Low | Fixed | Membership proposals validated only at execution |
| BT-04 | Low | Fixed | Event conformance with the standard not tested |
| BT-05 | Info | Fixed | SDK behind the network protocol, deprecated upgrade call |
| BT-06 | Info | Fixed | Tooling drift: CLI versions, stub scripts |
| BT-07 | Medium | Open, planned | One admin approver is enough for membership and code changes |
| BT-08 | Low | Open, planned | Proposals never expire |
| BT-09 | Low | Open, planned | No pause |
| BT-10 | Info | Accepted | Anyone can fund under any funding source label |
| BT-11 | Info | Accepted | The asset code is given at construction |
| BT-12 | Info | Accepted | An expense paid outside the ledger is attested by two members, not proven |
| BT-13 | Info | Open | The upgrade path is not exercised by unit tests |
| BT-14 | Info | Documented | One member can veto any proposal they could decide |
| BT-15 | Medium | Fixed | The declared testnet instance ran v0.3, and the v0.4 instance was not recorded |
| BT-16 | Low | In progress | Builds were not verifiable by explorers (SEP-55) |

## Details

### BT-01: A treasury could be created locked

The constructor only rejected an empty admin list. With a single admin, or the same address
listed twice, the treasury had one member able to approve. Since a proposer can never approve
their own proposal, no proposal could ever pass, including the one that would add a second
member.

**Fix.** The constructor requires at least two distinct admins and fails with `QuorumLost` (11)
otherwise. Tests: `constructor_needs_two_admins`, `constructor_counts_distinct_admins`.

### BT-02: Unbounded member list in instance storage

Members live in a map in instance storage, which every call loads. Each new member made every
call more expensive, and the instance entry is bounded by the network's maximum entry size.

**Fix.** At most 32 members (`MAX_MEMBERS`), enforced at construction and when adding a member,
with `TooManyMembers` (16). Test: `membership_is_capped`. If organisations need larger field
teams, members can move to one persistent entry each, with admin and approver counts in the
instance.

### BT-03: Membership proposals validated only at execution

Budgets, expenses and payments were validated when proposed and again when approved. Membership
changes were validated only when approved, so proposals that could never pass (removing an
unknown member, leaving the treasury without quorum) were stored and announced with
`proposal_created`.

**Fix.** `propose` runs the same validation as execution for every action, and execution checks
again because state may change in between. Tests: `quorum_cannot_be_lost_by_removal_or_role_change`,
`quorum_is_checked_again_when_a_proposal_is_approved`, `removing_an_unknown_member_is_refused`.

### BT-04: Event conformance with the standard not tested

The tests counted events but did not check their names or fields. A renamed struct field would
have silently broken the published standard for every reader.

**Fix.** `events_follow_the_standard` and `governance_events_follow_the_standard` check every
event's name and data fields against section 5 of `docs/proof-of-spend.md`, and the exact
content of `expense_recorded`, `payment_executed` and `proposal_executed`.

### BT-05: SDK behind the network protocol

The workspace pinned `soroban-sdk` 23. Mainnet runs protocol 29 and testnet protocol 30. The SDK
major version tracks the protocol, and version 30 is only a release candidate.

**Fix.** `soroban-sdk` 29.0.1 for every contract. The release WASM declares protocol 29 in its
environment meta, so it deploys on mainnet and on testnet. `update_current_contract_wasm`,
deprecated in this SDK, is replaced by `update_current_contract`.

### BT-06: Tooling drift

CI installed Stellar CLI 23 while builds and deployments used CLI 27. `scripts/deploy.sh` and
`scripts/generate-bindings.sh` were stubs and no TypeScript client existed.

**Fix.** CI uses CLI 27.0.0, checks that the WASM declares `event_schema = proof-of-spend/1` and
that the committed bindings match the contract. Both scripts are implemented; `deploy.sh` refuses
mainnet without explicit confirmation, refuses a dirty working tree, and records every instance
in `deployments/<network>.json`.

### BT-07: One admin approver is enough for membership and code changes

Adding or removing members and replacing the code need one admin besides the proposer. Two
colluding admins can replace the code. `contract_upgraded` is emitted before the code changes,
so readers can see it.

**Plan.** Approval by a configurable number of admins and a timelock on upgrades, before
mainnet.

### BT-08: Proposals never expire

A pending proposal can be approved months later. Execution checks the current state again, so
invariants hold, but the approver may act on stale context.

**Plan.** An expiry ledger on each proposal.

### BT-09: No pause

There is no way to stop all spending at once if a member's key is compromised. Admins can remove
the member while quorum holds.

**Plan.** A pause that any admin can trigger and only two admins can lift.

### BT-10: Anyone can fund under any funding source label

`fund` is open, and `source` is a label chosen by the caller. Someone could deposit under a
funder's label. It only adds money, and `project_funded.from` shows who deposited, so readers
bind a source to a funder by address.

### BT-11: The asset code is given at construction

`token_symbol` is passed to the constructor instead of read from the token. The deployer must
pass the right code; `deployments/` records both the token and the code.

### BT-12: An expense paid outside the ledger is attested by two members, not proven

`RecordExpense` moves no funds. The ledger proves who approved it, against which budget and with
which document, not that the bank transfer or cash payment happened. `payment_ref` links the
external evidence for auditors. This is the scope of the standard.

### BT-13: The upgrade path is not exercised by unit tests

Tests cover who may propose and approve an upgrade, not the code swap itself, which needs a
second WASM. It will be exercised on testnet before mainnet.

### BT-14: One member can veto any proposal they could decide

An approver can reject any budget, expense or payment, and an admin any proposal. Spending needs
two people to agree and stopping it needs one. Documented in the contract README as intended.

### BT-15: Declared deployment did not match the code

`deployments/testnet.json` pointed at the v0.3 instance (SDK 23, with BT-01 to BT-04), while the
only instance built from the current code had been deployed by the platform and not recorded.
The v0.4 instance is now the recorded one, its WASM hash matches a build of `6a5acb6`, and v0.3 is
retired. The July 2026 instances of the legacy contracts, which predate this repository and accept
`initialize` and writes from any account, are listed as retired.

### BT-16: Builds were not verifiable by explorers

Reproducibility was checked by hand (v0.3 rebuilt byte for byte from `ca2e196`). The release
workflow now builds, attests (SEP-55) and publishes the WASM on each version tag; instances
deployed from a release asset verify on explorers. The current demo instance was deployed from a
local build and does not verify; it will be replaced by one deployed from the first release.

## Verification

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace          # 72 tests, 27 in budget-treasury
./scripts/build.sh              # budget_treasury.wasm: 27,285 bytes, protocol 29
```
