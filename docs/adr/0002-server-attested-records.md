# 2. Records are attested by the platform

- **Status:** Superseded by [ADR 0003](0003-members-sign-proof-of-spend.md)
- **Date:** 2026-09-22
- **Related:** audit finding A-01

## Context

Foundations using Truway do not hold or manage keys. Every write to `fund-tracker` and
`expense-anchor` is signed by a single platform key, which is also the contract admin. The
fields `organization` (in `fund-tracker`) and `submitted_by` (in `expense-anchor`) therefore
store that signer, not the foundation.

The alternative is for each foundation to sign its own records. That would prove authorship
on-chain, but it requires every foundation to manage a key, and it changes both the contract
interface and the platform.

## Decision

Keep the attestation model: the platform attests every record, and attribution to each
foundation lives off-chain in the platform.

## Consequences

- A record proves that Truway attested a fact at a given ledger time, and that the fact has
  not changed since. It does not prove that a specific foundation submitted it.
- Foundations need no key management, which keeps onboarding simple.
- Compromise of the platform key would allow false records. The planned role separation (a
  cold multisig admin and a hot writer key) limits the damage to writes, removing upgrade and
  admin powers from the online key.
- Revisit this decision if foundations start signing their own records.
