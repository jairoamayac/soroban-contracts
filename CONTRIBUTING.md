# Contributing

Thanks for your interest. These contracts hold public records that cannot be edited once
written, so we favor small, well-tested changes and clear reasoning over speed.

## Setup

1. Install Rust through [rustup](https://rustup.rs). The pinned toolchain, `wasm32v1-none`
   target, `rustfmt` and `clippy` install on first use from `rust-toolchain.toml`.
2. Install the [Stellar CLI](https://developers.stellar.org/docs/tools/cli), version 27 (the one CI pins).
3. Check that everything works:

   ```bash
   make all   # fmt-check, lint, test, build
   ```

## Workflow

1. Pick or open an issue, so work is visible before it starts.
2. Branch from `main`: `feat/<topic>`, `fix/<topic>`, `docs/<topic>`, `chore/<topic>`, `ci/<topic>`.
3. Keep pull requests small: one change, reviewable in about fifteen minutes. Push your work
   at the end of every session instead of keeping it local.
4. Write commits in [Conventional Commits](https://www.conventionalcommits.org/) style, scoped
   by crate when it applies: `feat(expense-anchor): ...`, `docs(adr): ...`.
5. A pull request needs green CI and one approval from a maintainer.
6. Merge with a merge commit or a rebase. Squash merging is disabled so every commit keeps its
   author and date.

## Changing a contract

- Any change to a public function, event, type or storage key affects deployed instances and
  generated bindings. Record the reasoning in an ADR (`docs/adr/`) and the change in
  `CHANGELOG.md`.
- Storage key variants are part of the on-ledger encoding. Never rename or remove one used by
  a deployed contract without a migration plan.
- Logic shared by more than one contract belongs in `packages/common`.
- Every new code path needs a test, including the failure paths. Panic tests assert the exact
  message with `#[should_panic(expected = "...")]`.

## Commit identity

Commit with an email address verified on your GitHub account, or with your GitHub `noreply`
address, so your work is attributed to you.

## AI assistance

AI coding assistants are welcome. You remain responsible for every line you submit: read it,
test it and be ready to explain it in review. When an assistant wrote a substantial part of a
commit, add a `Co-Authored-By` trailer.

## Code of conduct

This project follows the [Contributor Covenant](CODE_OF_CONDUCT.md).
