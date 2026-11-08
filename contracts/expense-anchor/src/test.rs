// Amounts are written as <xlm>_<stroops>, e.g. 1_000_0000000 is 1,000 XLM,
// which clippy reads as inconsistent digit grouping.
#![allow(clippy::inconsistent_digit_grouping)]

use crate::{ExpenseAnchor, ExpenseAnchorClient};
use soroban_sdk::testutils::{Address as _, Events as _, Ledger as _};
use soroban_sdk::{symbol_short, vec, xdr, Address, Bytes, Env, IntoVal, Symbol, Val, Vec};

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

fn make_hash(env: &Env, seed: u8) -> Bytes {
    Bytes::from_array(env, &[seed; 32])
}

fn setup() -> (Env, ExpenseAnchorClient<'static>, Address) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(ExpenseAnchor, ());
    let client = ExpenseAnchorClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    client.initialize(&admin);
    (env, client, admin)
}

#[test]
fn test_anchor_and_get() {
    let (env, client, admin) = setup();

    let expense_id = symbol_short!("exp1");
    let project_id = symbol_short!("proj1");
    let hash = make_hash(&env, 0xAB);

    client.anchor(&admin, &expense_id, &project_id, &250_0000000, &hash);

    let exp = client.get_expense(&expense_id).unwrap();
    assert_eq!(exp.expense_id, expense_id);
    assert_eq!(exp.project_id, project_id);
    assert_eq!(exp.submitted_by, admin);
    assert_eq!(exp.amount_xlm, 250_0000000);
    assert_eq!(exp.receipt_hash, hash);
}

#[test]
fn test_anchor_requires_admin_auth() {
    let (env, client, admin) = setup();
    client.anchor(
        &admin,
        &symbol_short!("exp1"),
        &symbol_short!("proj1"),
        &1,
        &make_hash(&env, 1),
    );

    let auths = env.auths();
    assert_eq!(auths.len(), 1);
    assert_eq!(auths[0].0, admin);
}

#[test]
fn test_get_nonexistent_returns_none() {
    let (_env, client, _admin) = setup();
    assert!(client.get_expense(&symbol_short!("ghost")).is_none());
}

#[test]
fn test_anchored_at_matches_ledger_time() {
    let (env, client, admin) = setup();
    env.ledger().set_timestamp(1_700_000_000);

    let expense_id = symbol_short!("exp2");
    client.anchor(
        &admin,
        &expense_id,
        &symbol_short!("proj1"),
        &100_0000000,
        &make_hash(&env, 0x01),
    );

    let exp = client.get_expense(&expense_id).unwrap();
    assert_eq!(exp.anchored_at, 1_700_000_000);
}

#[test]
fn test_different_expenses_independent() {
    let (env, client, admin) = setup();
    let proj = symbol_short!("proj1");

    client.anchor(
        &admin,
        &symbol_short!("expA"),
        &proj,
        &100_0000000,
        &make_hash(&env, 0x01),
    );
    client.anchor(
        &admin,
        &symbol_short!("expB"),
        &proj,
        &200_0000000,
        &make_hash(&env, 0x02),
    );

    assert_eq!(
        client
            .get_expense(&symbol_short!("expA"))
            .unwrap()
            .amount_xlm,
        100_0000000
    );
    assert_eq!(
        client
            .get_expense(&symbol_short!("expB"))
            .unwrap()
            .amount_xlm,
        200_0000000
    );
}

#[test]
fn test_zero_amount_accepted() {
    // Reports with no funds used are anchored with amount 0.
    let (env, client, admin) = setup();
    let expense_id = symbol_short!("rep0");
    client.anchor(
        &admin,
        &expense_id,
        &symbol_short!("proj1"),
        &0,
        &make_hash(&env, 0x01),
    );
    assert_eq!(client.get_expense(&expense_id).unwrap().amount_xlm, 0);
}

#[test]
fn test_re_anchor_same_data_is_noop() {
    let (env, client, admin) = setup();
    let expense_id = symbol_short!("exp3");
    let proj = symbol_short!("proj1");
    let hash = make_hash(&env, 0x01);

    env.ledger().set_timestamp(1_700_000_000);
    client.anchor(&admin, &expense_id, &proj, &50_0000000, &hash);
    env.ledger().set_timestamp(1_800_000_000);
    client.anchor(&admin, &expense_id, &proj, &50_0000000, &hash);

    let exp = client.get_expense(&expense_id).unwrap();
    assert_eq!(exp.anchored_at, 1_700_000_000);
    assert!(
        env.events().all().events().is_empty(),
        "a no-op must not emit an event"
    );
}

#[test]
#[should_panic(expected = "expense_already_exists")]
fn test_re_anchor_different_data_panics() {
    let (env, client, admin) = setup();
    let expense_id = symbol_short!("exp3");
    let proj = symbol_short!("proj1");

    client.anchor(
        &admin,
        &expense_id,
        &proj,
        &50_0000000,
        &make_hash(&env, 0x01),
    );
    client.anchor(
        &admin,
        &expense_id,
        &proj,
        &99_0000000,
        &make_hash(&env, 0x02),
    );
}

#[test]
fn test_anchor_emits_event() {
    let (env, client, admin) = setup();

    let expense_id = symbol_short!("exp4");
    let project_id = symbol_short!("proj1");
    let hash = make_hash(&env, 0xFF);

    client.anchor(&admin, &expense_id, &project_id, &300_0000000, &hash);

    let expected_topic = Symbol::new(&env, "expense_anchored");
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
#[should_panic(expected = "not_admin")]
fn test_non_admin_caller_panics() {
    let (env, client, _admin) = setup();
    let intruder = Address::generate(&env);
    client.anchor(
        &intruder,
        &symbol_short!("eA"),
        &symbol_short!("proj1"),
        &10_0000000,
        &make_hash(&env, 0x0A),
    );
}

#[test]
#[should_panic(expected = "already_initialized")]
fn test_double_initialize_panics() {
    let (_env, client, admin) = setup();
    client.initialize(&admin);
}

#[test]
#[should_panic(expected = "not_initialized")]
fn test_anchor_before_initialize_panics() {
    let env = Env::default();
    env.mock_all_auths();
    let client = ExpenseAnchorClient::new(&env, &env.register(ExpenseAnchor, ()));
    client.anchor(
        &Address::generate(&env),
        &symbol_short!("e1"),
        &symbol_short!("p1"),
        &1,
        &make_hash(&env, 1),
    );
}

#[test]
#[should_panic(expected = "invalid_amount")]
fn test_negative_amount_panics() {
    let (env, client, admin) = setup();
    client.anchor(
        &admin,
        &symbol_short!("neg"),
        &symbol_short!("proj1"),
        &-1,
        &make_hash(&env, 0x01),
    );
}

#[test]
#[should_panic(expected = "invalid_receipt_hash")]
fn test_short_receipt_hash_panics() {
    let (env, client, admin) = setup();
    let short_hash = Bytes::from_array(&env, &[0x01]);
    client.anchor(
        &admin,
        &symbol_short!("short"),
        &symbol_short!("proj1"),
        &100_0000000,
        &short_hash,
    );
}
