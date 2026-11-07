# Truway Soroban Contracts

[![CI](https://github.com/truway-xyz/soroban-contracts/actions/workflows/ci.yml/badge.svg)](https://github.com/truway-xyz/soroban-contracts/actions/workflows/ci.yml)
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

Proof of Spend for [Truway](https://truway.xyz): an open standard for budget execution on
Stellar, and the contract that implements it. Truway traces how funds are budgeted, distributed
and spent down to the last mile; these contracts keep that record on Stellar, where any funder,
auditor or public official can check it without asking Truway.

> **Status: pre-release, testnet only.** `budget-treasury` runs on testnet (see
> [`deployments/testnet.json`](deployments/testnet.json)). Interfaces may change before `v1.0.0`.

## Proof of Spend

A payment on Stellar proves that money moved. [Proof of Spend](docs/proof-of-spend.md)
(`proof-of-spend/1`) proves what it bought: against which budget line, from which funder, booked
to which account, backed by which document and approved by whom. Any contract can implement it,
and anyone can rebuild a budget execution report or a general ledger from the events alone.

## Contracts

| Contract | Status | What it does | Reference |
|---|---|---|---|
| `budget-treasury` | Active, testnet | Reference implementation of Proof of Spend: budget lines per funder, expenses and payments with two signers and a supporting document | [README](contracts/budget-treasury/README.md) |
| `fund-tracker` | Legacy | Budget allocated to each project, attested by the platform | [README](contracts/fund-tracker/README.md) |
| `expense-anchor` | Legacy | Expenses anchored by the platform with the hash of their receipt | [README](contracts/expense-anchor/README.md) |
| `sbt-badge` | Legacy | Non-transferable reputation badges | [README](contracts/sbt-badge/README.md) |

Legacy contracts follow the server-attested model of ADR 0002, superseded by
[ADR 0003](docs/adr/0003-members-sign-proof-of-spend.md). They get no new deployments and will be
removed once the platform no longer calls them.

## How it fits together

```
Organisation members (admins, approvers, field operators)
  │  each signs with their own key
  ▼
Truway platform (closed source)
  │  builds transactions, stores documents; deploys but is never a member
  ▼
budget-treasury   propose · approve · reject · fund        (one instance per organisation)
  │
  └── Proof of Spend events ──► funders, auditors, oversight bodies, public explorer
```

Every budget line, expense, payment, membership change and upgrade is proposed by one member and
approved by a different one. Truway cannot move funds, approve or upgrade. Details in
[`docs/architecture.md`](docs/architecture.md).

Checking a record needs no account: hash the supporting document with SHA-256 and compare it with
`receipt_hash` in the event. See [`docs/verification-spec.md`](docs/verification-spec.md).

## Repository layout

```
.
├── contracts/
│   ├── budget-treasury/  Proof of Spend reference implementation
│   └── fund-tracker/, expense-anchor/, sbt-badge/   legacy
├── packages/
│   └── common/           truway-common: TTL policy shared by every contract
├── docs/                 Proof of Spend, architecture, events, storage, verification, ADRs
├── deployments/          Contract IDs and WASM hashes per network
├── scripts/              Build, deploy and bindings scripts
├── bindings/             Generated TypeScript client
├── examples/             Read-only integration examples
└── audits/               Security reviews and their status
```

## Getting started

Requirements:

- Rust through [rustup](https://rustup.rs). The toolchain and the `wasm32v1-none` target are
  pinned in [`rust-toolchain.toml`](rust-toolchain.toml) and install on first use.
- [Stellar CLI](https://developers.stellar.org/docs/tools/cli) 27 (pinned in CI).

```bash
make test    # unit tests for every contract
make lint    # rustfmt and clippy, warnings as errors
make build   # release WASM in target/wasm32v1-none/release/
```

## Contributing

Read [CONTRIBUTING.md](CONTRIBUTING.md). Design decisions are recorded as ADRs in
[`docs/adr/`](docs/adr/). Changes are listed in [CHANGELOG.md](CHANGELOG.md).

## Security

Please do not open public issues for vulnerabilities. See [SECURITY.md](SECURITY.md) for how
to report one, and [`audits/`](audits/) for past reviews.

## Roadmap to mainnet

- [x] `budget-treasury`: two signers on every change, budgets per funding source, single-use
  supporting documents, typed errors, events that follow Proof of Spend
- [x] Testnet deployment under Truway-controlled keys, with the deployer outside the treasury
- [x] Internal review of `budget-treasury` ([report](audits/2026-11-07-budget-treasury-internal-review.md))
- [x] Generated TypeScript client and deploy script
- [ ] Per-person signing in the platform (no demo signers)
- [ ] Approval by several admins and a timelock for membership and code changes; proposal
  expiry; pause
- [ ] Proof of Spend `v1.0` with public feedback, and a read-only reference indexer
- [ ] Reproducible builds with source verification (SEP-55)
- [ ] Independent security review
- [ ] Mainnet deployment
- [ ] Removal of the legacy contracts

## Use of AI

We use AI coding assistants to draft code, tests and documentation. Every change is reviewed
and tested by the team before merge. Commits written with substantial AI assistance carry a
`Co-Authored-By` trailer.

## Maintainers

[@emanuelbenavile](https://github.com/emanuelbenavile) · [@angel1901](https://github.com/angel1901) ·
[@jairoamayac](https://github.com/jairoamayac)

## License

[Apache-2.0](LICENSE).
