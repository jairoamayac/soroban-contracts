# 3. Members sign, and records follow Proof of Spend

- **Status:** Accepted
- **Date:** 2026-11-07
- **Supersedes:** [ADR 0002](0002-server-attested-records.md)
- **Related:** [Proof of Spend](../proof-of-spend.md), [budget-treasury internal review](../../audits/2026-11-07-budget-treasury-internal-review.md)

## Context

Under ADR 0002 a single platform key wrote every record. A record proved that Truway attested a
fact, not that the organisation's own people approved it, and its format was specific to
Truway's contracts.

Funders, auditors and public officials asked for two things the attested model cannot give:
proof that the people responsible approved each expense, and a common format that any system can
read. At a regional workshop for Colombia's National Development Plan (October 2026), officials
asked for an interoperable system linking budget allocation, contracting and execution of public
funds.

Stellar supports both. A contract can require the signature of each member's address, and an
embedded signer lets members sign without handling seed phrases. Events are public and permanent
once ingested, so a documented event format becomes an interface anyone can implement.

## Decision

1. **A treasury per organisation.** `budget-treasury` keeps the organisation's members and
   roles. Every budget line, expense, payment, membership change and upgrade is proposed by one
   member and approved by a different one. Truway deploys the contract and is not a member.
2. **An open standard as the public interface.** Records follow Proof of Spend
   (`proof-of-spend/1`), declared in the contract meta as `event_schema`. The standard, not the
   platform's API, is what funders and governments integrate with, and any contract can implement
   it.
3. **Two kinds of spending.** `RecordExpense` records an expense paid outside the ledger (bank
   transfer, cash) and moves no funds. `Pay` pays from funds held in the contract. Both need a
   supporting document, used once, and both book to an accounting code.
4. **Demo signers only on testnet.** Until per-person signing ships, the platform may hold
   testnet keys that stand in for demo members. They are never used on mainnet.
5. **Legacy contracts frozen.** `fund-tracker`, `expense-anchor` and `sbt-badge` get no new
   deployments and are removed once the platform no longer calls them.

## Consequences

- The ledger shows who proposed and who approved each record, by address.
- Anyone can rebuild a budget execution report per funder, and a general ledger, from events
  alone.
- Each member needs a key, which the platform provisions through an embedded signer.
- Every record needs two signatures, so it lands after the second member acts, not immediately.
- The event format is a public commitment: changes follow the versioning rules of the standard,
  and error codes are never renumbered.
- Until the platform migrates, the repository carries legacy code that readers must not confuse
  with the reference implementation.
