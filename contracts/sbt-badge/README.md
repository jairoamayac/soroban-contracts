# sbt-badge

> **Legacy.** Superseded by [`budget-treasury`](../budget-treasury) and
> [ADR 0003](../../docs/adr/0003-members-sign-proof-of-spend.md). No new deployments. Kept until
> the Truway platform no longer calls it.

Non-transferable (soulbound) reputation badges for organizations. There is no `transfer`
function by design. Revoked badges stay on the ledger with `status = Revoked`.

## Interface

| Function | Auth | Description |
|---|---|---|
| `initialize(admin)` | `admin` | Sets the admin and starts token ids at 1. Fails on a second call. |
| `mint_badge(organization, badge_type) -> u64` | admin | Mints a badge and returns its `token_id`. |
| `revoke_badge(token_id)` | admin | Revokes an active badge. |
| `get_badge(token_id) -> Option<Badge>` | none | Reads a badge. |
| `get_badges(organization) -> Vec<Badge>` | none | All badges of an organization, active and revoked. |
| `get_active_badges(organization) -> Vec<Badge>` | none | Active badges of an organization. |
| `upgrade(new_wasm_hash)` | admin | Replaces the code in place, keeping the contract ID and storage. |

## Badge types

| Type | Meaning |
|---|---|
| `kyb_verified` | The organization completed know-your-business verification |
| `transparency_bronze` | First transparency milestone |
| `transparency_silver` | Intermediate transparency milestone |
| `transparency_gold` | Highest verified transparency level |

Types are compiled into the contract. Adding one requires an `upgrade`.

## Storage

| Key | Storage | Value |
|---|---|---|
| `Admin` | instance | `Address` |
| `NextTokenId` | instance | `u64`, global counter starting at 1 |
| `BadgeById(token_id)` | persistent | `Badge { token_id, badge_type, organization, status, issued_at, revoked_at }` |
| `OrgBadges(organization)` | persistent | `Vec<u64>`, index of the organization's token ids |

`revoked_at` is 0 while the badge is active.

## Events

| Event | Topics | Data |
|---|---|---|
| `BadgeMinted` | `("badge_minted", badge_type)` | `token_id`, `organization` |
| `BadgeRevoked` | `("badge_revoked",)` | `token_id`, `organization` |

## Errors

| Panic message | When |
|---|---|
| `already_initialized` | `initialize` called twice |
| `not_initialized` | Any write before `initialize` |
| `invalid_badge_type` | `badge_type` is not one of the types above |
| `badge_already_active` | The organization already holds an active badge of that type |
| `token_not_found` | `revoke_badge` on an unknown `token_id` |
| `already_revoked` | `revoke_badge` on a revoked badge |
