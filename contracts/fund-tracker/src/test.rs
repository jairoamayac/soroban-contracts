// Amounts are written as <xlm>_<stroops>, e.g. 1_000_0000000 is 1,000 XLM,
// which clippy reads as inconsistent digit grouping.
#![allow(clippy::inconsistent_digit_grouping)]

use crate::{FundTracker, FundTrackerClient};
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

fn setup() -> (Env, FundTrackerClient<'static>, Address) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(FundTracker, ());
    let client = FundTrackerClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    client.initialize(&admin);
    (env, client, admin)
}

#[test]
fn test_allocate_and_get() {
    let (_env, client, admin) = setup();

    let project_id = symbol_short!("proj1");
    client.allocate(&admin, &project_id, &1_000_0000000);

    let alloc = client.get_allocation(&project_id).unwrap();
    assert_eq!(alloc.amount_xlm, 1_000_0000000);
    assert_eq!(alloc.project_id, project_id);
    assert_eq!(alloc.organization, admin);
}

#[test]
fn test_allocate_requires_admin_auth() {
    let (env, client, admin) = setup();
    client.allocate(&admin, &symbol_short!("proj1"), &1_0000000);

    let auths = env.auths();
    assert_eq!(auths.len(), 1);
    assert_eq!(auths[0].0, admin);
}

#[test]
fn test_get_nonexistent_returns_none() {
    let (_env, client, _admin) = setup();
    let missing = symbol_short!("ghost");
    assert!(client.get_allocation(&missing).is_none());
}

#[test]
fn test_reallocate_overwrites() {
    let (_env, client, admin) = setup();
    let project_id = symbol_short!("proj2");

    client.allocate(&admin, &project_id, &500_0000000);
    client.allocate(&admin, &project_id, &1_500_0000000);

    let alloc = client.get_allocation(&project_id).unwrap();
    assert_eq!(alloc.amount_xlm, 1_500_0000000);
}

#[test]
fn test_different_projects_independent() {
    let (_env, client, admin) = setup();
    let p1 = symbol_short!("projA");
    let p2 = symbol_short!("projB");

    client.allocate(&admin, &p1, &100_0000000);
    client.allocate(&admin, &p2, &200_0000000);

    assert_eq!(client.get_allocation(&p1).unwrap().amount_xlm, 100_0000000);
    assert_eq!(client.get_allocation(&p2).unwrap().amount_xlm, 200_0000000);
}

#[test]
fn test_allocated_at_matches_ledger_time() {
    let (env, client, admin) = setup();
    env.ledger().set_timestamp(1_700_000_000);

    let project_id = symbol_short!("proj3");
    client.allocate(&admin, &project_id, &50_0000000);

    let alloc = client.get_allocation(&project_id).unwrap();
    assert_eq!(alloc.allocated_at, 1_700_000_000);
}

#[test]
fn test_allocate_emits_event() {
    let (env, client, admin) = setup();
    let project_id = symbol_short!("proj4");

    client.allocate(&admin, &project_id, &75_0000000);

    let expected_topic = Symbol::new(&env, "fund_allocated");
    assert_eq!(
        last_topics(&env),
        topics(vec![
            &env,
            expected_topic.into_val(&env),
            project_id.into_val(&env)
        ])
    );
}

#[test]
#[should_panic(expected = "already_initialized")]
fn test_double_initialize_panics() {
    let (_env, client, admin) = setup();
    client.initialize(&admin);
}

#[test]
#[should_panic(expected = "invalid_amount")]
fn test_negative_amount_panics() {
    let (_env, client, admin) = setup();
    client.allocate(&admin, &symbol_short!("neg1"), &-1);
}

#[test]
#[should_panic(expected = "invalid_amount")]
fn test_zero_amount_panics() {
    let (_env, client, admin) = setup();
    client.allocate(&admin, &symbol_short!("zero1"), &0);
}

#[test]
#[should_panic(expected = "not_admin")]
fn test_non_admin_caller_panics() {
    let (env, client, _admin) = setup();
    let intruder = Address::generate(&env);
    client.allocate(&intruder, &symbol_short!("proj1"), &100_0000000);
}

#[test]
#[should_panic(expected = "not_initialized")]
fn test_allocate_before_initialize_panics() {
    let env = Env::default();
    env.mock_all_auths();
    let client = FundTrackerClient::new(&env, &env.register(FundTracker, ()));
    client.allocate(&Address::generate(&env), &symbol_short!("proj1"), &1);
}
