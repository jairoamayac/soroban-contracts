// Amounts are written as <xlm>_<stroops>, e.g. 1_000_0000000 is 1,000 XLM,
// which clippy reads as inconsistent digit grouping.
#![allow(clippy::inconsistent_digit_grouping)]

use crate::{BadgeStatus, SbtBadge, SbtBadgeClient};
use soroban_sdk::testutils::{Address as _, Events as _, Ledger as _};
use soroban_sdk::{symbol_short, vec, xdr, Address, Env, IntoVal, Symbol, Val, Vec};

/// Topics of the last event emitted by the last invocation.
fn last_topics(env: &Env) -> xdr::VecM<xdr::ScVal> {
    let events = env.events().all();
    let xdr::ContractEventBody::V0(body) = &events.events().last().expect("no event").body;
    body.topics.clone()
}

/// Expected topics, in their XDR form.
fn topics(t: Vec<Val>) -> xdr::VecM<xdr::ScVal> {
    t.into()
}

fn setup() -> (Env, SbtBadgeClient<'static>, Address) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(SbtBadge, ());
    let client = SbtBadgeClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    client.initialize(&admin);
    (env, client, admin)
}

// ── mint_badge ────────────────────────────────────────────────────────────

#[test]
fn test_mint_returns_sequential_ids() {
    let (env, client, _admin) = setup();
    let org = Address::generate(&env);
    assert_eq!(
        client.mint_badge(&org, &Symbol::new(&env, "kyb_verified")),
        1
    );
    assert_eq!(
        client.mint_badge(&org, &Symbol::new(&env, "transparency_bronze")),
        2
    );
    assert_eq!(
        client.mint_badge(&org, &Symbol::new(&env, "transparency_silver")),
        3
    );
}

#[test]
fn test_mint_requires_admin_auth() {
    let (env, client, admin) = setup();
    let org = Address::generate(&env);
    client.mint_badge(&org, &Symbol::new(&env, "kyb_verified"));

    let auths = env.auths();
    assert_eq!(auths.len(), 1);
    assert_eq!(auths[0].0, admin);
}

#[test]
fn test_mint_emits_event() {
    let (env, client, _admin) = setup();
    let org = Address::generate(&env);
    let bt = Symbol::new(&env, "kyb_verified");
    client.mint_badge(&org, &bt);

    let expected_topic = Symbol::new(&env, "badge_minted");
    assert_eq!(
        last_topics(&env),
        topics(vec![&env, expected_topic.into_val(&env), bt.into_val(&env)])
    );
}

// ── get_badge ─────────────────────────────────────────────────────────────

#[test]
fn test_get_badge_correct_data() {
    let (env, client, _admin) = setup();
    let org = Address::generate(&env);
    env.ledger().set_timestamp(1_700_000_000);

    let bt = Symbol::new(&env, "transparency_bronze");
    let id = client.mint_badge(&org, &bt);
    let badge = client.get_badge(&id).unwrap();

    assert_eq!(badge.token_id, id);
    assert_eq!(badge.organization, org);
    assert_eq!(badge.badge_type, bt);
    assert_eq!(badge.status, BadgeStatus::Active);
    assert_eq!(badge.issued_at, 1_700_000_000);
    assert_eq!(badge.revoked_at, 0);
}

#[test]
fn test_get_badge_nonexistent_returns_none() {
    let (_env, client, _admin) = setup();
    assert!(client.get_badge(&999).is_none());
}

// ── get_badges / get_active_badges ────────────────────────────────────────

#[test]
fn test_get_badges_returns_all() {
    let (env, client, _admin) = setup();
    let org = Address::generate(&env);
    client.mint_badge(&org, &Symbol::new(&env, "kyb_verified"));
    client.mint_badge(&org, &Symbol::new(&env, "transparency_bronze"));
    assert_eq!(client.get_badges(&org).len(), 2);
}

#[test]
fn test_get_badges_empty_for_unknown_org() {
    let (env, client, _admin) = setup();
    let org = Address::generate(&env);
    assert_eq!(client.get_badges(&org).len(), 0);
}

#[test]
fn test_get_active_badges_excludes_revoked() {
    let (env, client, _admin) = setup();
    let org = Address::generate(&env);
    let id1 = client.mint_badge(&org, &Symbol::new(&env, "kyb_verified"));
    client.mint_badge(&org, &Symbol::new(&env, "transparency_bronze"));
    client.revoke_badge(&id1);

    let active = client.get_active_badges(&org);
    assert_eq!(active.len(), 1);
    assert_eq!(active.get(0).unwrap().status, BadgeStatus::Active);
}

// ── revoke_badge ──────────────────────────────────────────────────────────

#[test]
fn test_revoke_sets_status_and_timestamp() {
    let (env, client, _admin) = setup();
    let org = Address::generate(&env);
    env.ledger().set_timestamp(1_700_000_000);
    let id = client.mint_badge(&org, &Symbol::new(&env, "kyb_verified"));

    env.ledger().set_timestamp(1_800_000_000);
    client.revoke_badge(&id);

    let badge = client.get_badge(&id).unwrap();
    assert_eq!(badge.status, BadgeStatus::Revoked);
    assert_eq!(badge.revoked_at, 1_800_000_000);
    assert_eq!(badge.issued_at, 1_700_000_000); // original field unchanged
}

#[test]
fn test_revoke_emits_event() {
    let (env, client, _admin) = setup();
    let org = Address::generate(&env);
    let id = client.mint_badge(&org, &Symbol::new(&env, "kyb_verified"));
    // events().all() returns only events from the last invocation;
    // verify right after revoke that the event was emitted.
    client.revoke_badge(&id);
    assert!(!env.events().all().events().is_empty());
}

#[test]
#[should_panic(expected = "already_revoked")]
fn test_revoke_already_revoked_panics() {
    let (env, client, _admin) = setup();
    let org = Address::generate(&env);
    let id = client.mint_badge(&org, &Symbol::new(&env, "kyb_verified"));
    client.revoke_badge(&id);
    client.revoke_badge(&id);
}

#[test]
#[should_panic(expected = "token_not_found")]
fn test_revoke_nonexistent_panics() {
    let (_env, client, _admin) = setup();
    client.revoke_badge(&999);
}

// ── badge type validation ─────────────────────────────────────────────────

#[test]
fn test_all_valid_badge_types_accepted() {
    let (env, client, _admin) = setup();
    let org = Address::generate(&env);
    for bt in &[
        "kyb_verified",
        "transparency_bronze",
        "transparency_silver",
        "transparency_gold",
    ] {
        client.mint_badge(&org, &Symbol::new(&env, bt));
    }
    assert_eq!(client.get_badges(&org).len(), 4);
}

#[test]
#[should_panic(expected = "invalid_badge_type")]
fn test_invalid_badge_type_panics() {
    let (env, client, _admin) = setup();
    let org = Address::generate(&env);
    client.mint_badge(&org, &symbol_short!("unknown"));
}

// ── multi-org isolation ───────────────────────────────────────────────────

#[test]
fn test_badges_are_per_organization() {
    let (env, client, _admin) = setup();
    let org_a = Address::generate(&env);
    let org_b = Address::generate(&env);
    client.mint_badge(&org_a, &Symbol::new(&env, "kyb_verified"));
    client.mint_badge(&org_a, &Symbol::new(&env, "transparency_bronze"));
    client.mint_badge(&org_b, &Symbol::new(&env, "kyb_verified"));

    assert_eq!(client.get_badges(&org_a).len(), 2);
    assert_eq!(client.get_badges(&org_b).len(), 1);
}

// ── initialize idempotency ────────────────────────────────────────────────

#[test]
#[should_panic(expected = "already_initialized")]
fn test_double_initialize_panics() {
    let (_env, client, admin) = setup();
    client.initialize(&admin);
}

#[test]
#[should_panic(expected = "not_initialized")]
fn test_mint_before_initialize_panics() {
    let env = Env::default();
    env.mock_all_auths();
    let client = SbtBadgeClient::new(&env, &env.register(SbtBadge, ()));
    client.mint_badge(&Address::generate(&env), &Symbol::new(&env, "kyb_verified"));
}

// ── one active badge per type ─────────────────────────────────────────────

#[test]
#[should_panic(expected = "badge_already_active")]
fn test_duplicate_active_badge_panics() {
    let (env, client, _admin) = setup();
    let org = Address::generate(&env);
    let bt = Symbol::new(&env, "kyb_verified");

    client.mint_badge(&org, &bt);
    client.mint_badge(&org, &bt);
}

#[test]
fn test_remint_after_revoke_allowed() {
    let (env, client, _admin) = setup();
    let org = Address::generate(&env);
    let bt = Symbol::new(&env, "kyb_verified");

    let id1 = client.mint_badge(&org, &bt);
    client.revoke_badge(&id1);
    let id2 = client.mint_badge(&org, &bt);

    assert_ne!(id1, id2);
    assert_eq!(client.get_badges(&org).len(), 2);
    assert_eq!(client.get_active_badges(&org).len(), 1);
}
