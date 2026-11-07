# Changelog

All notable changes to this project are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project uses
[Semantic Versioning](https://semver.org/).

## [Unreleased]

### Added

- `budget-treasury` 0.4.0, reference implementation of Proof of Spend (`proof-of-spend/1`):
  budget lines per project, line and funding source with an accounting code and a currency;
  `RecordExpense` for expenses paid outside the ledger and `Pay` for payments from funds held in
  the contract; two distinct signers on every change; single-use supporting documents; quorum and
  a cap of 32 members; typed errors; deployed on testnet.
- Proof of Spend Draft 2 (`docs/proof-of-spend.md`), and tests that check emitted events against
  it.
- ADR 0003: members sign, and records follow Proof of Spend.
- Internal review of `budget-treasury` (`audits/2026-11-07-budget-treasury-internal-review.md`).
- `scripts/deploy.sh` and `scripts/generate-bindings.sh`, and the generated TypeScript client in
  `bindings/typescript/budget-treasury`.
- CI checks that the WASM declares `event_schema` and that the bindings are up to date.

### Changed

- `soroban-sdk` 23.0.0 to 29.0.1, matching the protocol mainnet runs; Stellar CLI 27 in CI.
- Architecture, events, storage and verification documents describe `budget-treasury`.

### Deprecated

- `fund-tracker`, `expense-anchor` and `sbt-badge`: superseded by `budget-treasury` (ADR 0003,
  which supersedes ADR 0002). No new deployments.

### Earlier in this release

- `fund-tracker`, `expense-anchor` and `sbt-badge`, imported from the Truway platform with the
  fixes from the 2026-06-30 internal review: one-time `initialize`, admin checks on writes,
  input validation, events on every write, TTL extension on every write, immutable expense
  records and in-place `upgrade`.
- `truway-common` crate with the storage TTL policy shared by every contract.
- Cargo workspace with shared package metadata and dependency versions, and a pinned toolchain.
- CI: formatting, lints, tests and release WASM builds.
- Documentation: per-contract references, architecture decision records, audit history and
  deployment manifests.
