# Documentation

| Document | Status | What it covers |
|---|---|---|
| [proof-of-spend.md](proof-of-spend.md) | Draft 2 | The open standard: identifiers, events, invariants, how to rebuild reports, chart registry |
| [architecture.md](architecture.md) | Current | Components, trust model, life of an expense and of a payment, upgrades |
| [events.md](events.md) | Current | Where each contract's events are specified |
| [storage-and-ttl.md](storage-and-ttl.md) | Current | Storage layout of `budget-treasury` and the TTL policy |
| [verification-spec.md](verification-spec.md) | Current | How anyone checks a record without an account |
| [adr/](adr/) | Living | Architecture Decision Records |

Guides and diagrams in English and Spanish are on the
[documentation site](https://truway.gitbook.io/docs).

Per-contract interface references live next to the code, in each `contracts/<name>/README.md`.
Security reviews are in [`../audits/`](../audits/).
