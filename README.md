# Truway Soroban Contracts

[![CI](https://github.com/truway-xyz/soroban-contracts/actions/workflows/ci.yml/badge.svg)](https://github.com/truway-xyz/soroban-contracts/actions/workflows/ci.yml)
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

The public record layer of [Truway](https://truway.xyz), a platform that helps social
foundations prove how every donated dollar was spent, from the supplier's invoice to the
donor's report. These Soroban contracts keep that record on Stellar, where any donor,
auditor or developer can check it without asking Truway.

> **Status: pre-release.** Nothing is deployed from this repository yet. Earlier testnet
> instances are retired and must not be used. New deployments will be listed in
> [`deployments/`](deployments/). Interfaces may change before `v1.0.0`.

## Contracts

| Contract | What it records | Reference |
|---|---|---|
| `fund-tracker` | The budget allocated to each project | [README](contracts/fund-tracker/README.md) |
| `expense-anchor` | Approved expenses and donor reports, each with the SHA-256 hash of its receipt | [README](contracts/expense-anchor/README.md) |
| `sbt-badge` | Non-transferable reputation badges for foundations | [README](contracts/sbt-badge/README.md) |
| `budget-treasury` | An organisation's funds, paid only against approved budget lines with two signers and an accounting code | [README](contracts/budget-treasury/README.md) |

## Open schema

[Budget Execution Events](docs/budget-execution-events.md) (`truway-budget-execution/1`) is an open
event schema for executing earmarked money on Stellar: funding by source, approved budget lines, and
payments with their accounting code and proof of spend. `budget-treasury` is its reference
implementation; any contract can implement it and anyone can rebuild a budget execution report from
the ledger.

## How it fits together

```
Truway platform (closed source)
  │  signs every write with the contract admin key
  ├── fund-tracker     allocate · get_allocation
  ├── expense-anchor   anchor · get_expense
  └── sbt-badge        mint_badge · revoke_badge · get_badges

Donors, auditors, developers
  └── read records and events directly from the network
```

Records are **attested by Truway**: the signer of every write is the platform's admin key, and
attribution to each foundation lives off-chain. The ledger proves that Truway recorded a fact
at a given time and that it has not changed since. See
[ADR 0002](docs/adr/0002-server-attested-records.md) and the [audit history](audits/).

Checking an expense needs no account: download the receipt, compute its SHA-256 hash and
compare it with the `receipt_hash` returned by `get_expense`. The full verification format
is specified in [`docs/verification-spec.md`](docs/verification-spec.md).

## Repository layout

```
.
├── contracts/            One crate per contract (contract, storage, events, types, tests)
│   ├── fund-tracker/
│   ├── expense-anchor/
│   └── sbt-badge/
├── packages/
│   └── common/           truway-common: building blocks shared by every contract
├── deployments/          Contract IDs and WASM hashes per network
├── scripts/              Build, deploy and bindings scripts
├── bindings/             Generated TypeScript clients
├── examples/             Read-only integration examples
├── docs/                 Architecture, events, storage, verification spec, ADRs
└── audits/               Security reviews and their status
```

## Getting started

Requirements:

- Rust through [rustup](https://rustup.rs). The toolchain and the `wasm32v1-none` target are
  pinned in [`rust-toolchain.toml`](rust-toolchain.toml) and install on first use.
- [Stellar CLI](https://developers.stellar.org/docs/tools/cli) 23 or later.

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

- [x] Fixes from the internal review: one-time `initialize`, admin checks on writes, input
  validation, events on every write, TTL extension, immutable expense records, `upgrade`
- [ ] Testnet redeployment under Truway-controlled keys
- [ ] Role separation: a cold multisig admin for upgrades and roles, a hot writer for records
- [ ] Typed errors (`#[contracterror]`)
- [ ] Asset-aware amounts: the asset code stored with every amount
- [ ] Collision-free identifiers: full 16-byte UUIDs instead of truncated symbols
- [ ] Generated TypeScript bindings and read-only examples
- [ ] Verification spec
- [ ] Independent security review
- [ ] Reproducible builds with source verification
- [ ] Mainnet deployment

## Use of AI

We use AI coding assistants to draft code, tests and documentation. Every change is reviewed
and tested by the team before merge. Commits written with substantial AI assistance carry a
`Co-Authored-By` trailer.

## Maintainers

[@EmanuXBe](https://github.com/EmanuXBe) · [@angel1901](https://github.com/angel1901) ·
[@jairoamayac](https://github.com/jairoamayac)

## License

[Apache-2.0](LICENSE).
