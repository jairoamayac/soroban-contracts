// Amounts are written as <units>_<7 decimals>, e.g. 1_000_0000000 is 1,000.
#![allow(clippy::inconsistent_digit_grouping)]

use crate::{
    Action, BudgetChange, BudgetTreasury, BudgetTreasuryClient, Error, MemberChange, Payment,
    ProposalStatus, Role,
};
use soroban_sdk::testutils::{Address as _, Events as _};
use soroban_sdk::{symbol_short, token, vec, Address, BytesN, Env, Symbol};

struct Ctx {
    env: Env,
    client: BudgetTreasuryClient<'static>,
    token: token::TokenClient<'static>,
    coordinator: Address,
    approver: Address,
    funder: Address,
    supplier: Address,
}

fn id(env: &Env, n: u8) -> BytesN<32> {
    BytesN::from_array(env, &[n; 32])
}

fn project(env: &Env) -> BytesN<32> {
    id(env, 1)
}

fn food(env: &Env) -> BytesN<32> {
    id(env, 2)
}

/// Funder A (e.g. an international cooperation agency).
fn funder_a(env: &Env) -> BytesN<32> {
    id(env, 3)
}

/// Funder B (e.g. a corporate donor).
fn funder_b(env: &Env) -> BytesN<32> {
    id(env, 4)
}

fn setup() -> Ctx {
    let env = Env::default();
    env.mock_all_auths();
    let token_admin = Address::generate(&env);
    let sac = env.register_stellar_asset_contract_v2(token_admin);
    let token_addr = sac.address();
    let coordinator = Address::generate(&env);
    let approver = Address::generate(&env);
    let funder = Address::generate(&env);
    let supplier = Address::generate(&env);
    token::StellarAssetClient::new(&env, &token_addr).mint(&funder, &10_000_0000000);
    let contract_id = env.register(
        BudgetTreasury,
        (
            vec![&env, coordinator.clone(), approver.clone()],
            token_addr.clone(),
        ),
    );
    let client = BudgetTreasuryClient::new(&env, &contract_id);
    let token = token::TokenClient::new(&env, &token_addr);
    Ctx {
        env,
        client,
        token,
        coordinator,
        approver,
        funder,
        supplier,
    }
}

fn budget(c: &Ctx, amount: i128) -> Action {
    Action::SetBudget(BudgetChange {
        project: project(&c.env),
        line: food(&c.env),
        chart: symbol_short!("CO_PUC"),
        account: Symbol::new(&c.env, "5135"),
        amount,
    })
}

fn pay(c: &Ctx, amount: i128) -> Action {
    pay_with(c, amount, funder_a(&c.env), id(&c.env, 9))
}

fn pay_with(c: &Ctx, amount: i128, source: BytesN<32>, receipt: BytesN<32>) -> Action {
    Action::Pay(Payment {
        project: project(&c.env),
        line: food(&c.env),
        source,
        to: c.supplier.clone(),
        amount,
        chart: symbol_short!("CO_PUC"),
        account: Symbol::new(&c.env, "513595"),
        receipt_hash: receipt,
        reference: id(&c.env, 10),
    })
}

/// Budget of 1,000 on the food line and 1,500 from funder A for the project.
fn ready(c: &Ctx) {
    let pid = c.client.propose(&c.coordinator, &budget(c, 1_000_0000000));
    c.client.approve(&c.approver, &pid);
    c.client.fund(
        &c.funder,
        &project(&c.env),
        &funder_a(&c.env),
        &1_500_0000000,
    );
}

#[test]
fn constructor_sets_admins_and_no_one_else() {
    let c = setup();
    assert_eq!(c.client.role(&c.coordinator), Some(Role::Admin));
    assert_eq!(c.client.role(&c.approver), Some(Role::Admin));
    assert_eq!(c.client.role(&c.funder), None);
    assert_eq!(c.client.members().len(), 2);
}

#[test]
fn budget_needs_a_second_person() {
    let c = setup();
    let pid = c.client.propose(&c.coordinator, &budget(&c, 1_000_0000000));
    assert!(c.client.line(&project(&c.env), &food(&c.env)).is_none());
    c.client.approve(&c.approver, &pid);
    let line = c.client.line(&project(&c.env), &food(&c.env)).unwrap();
    assert_eq!(line.budget, 1_000_0000000);
    assert_eq!(line.paid, 0);
    assert_eq!(line.chart, symbol_short!("CO_PUC"));
    assert_eq!(
        c.client.proposal(&pid).unwrap().status,
        ProposalStatus::Executed
    );
}

#[test]
fn self_approval_is_rejected() {
    let c = setup();
    let pid = c.client.propose(&c.coordinator, &budget(&c, 1_000_0000000));
    assert_eq!(
        c.client.try_approve(&c.coordinator, &pid),
        Err(Ok(Error::SelfApproval))
    );
}

#[test]
fn proposal_cannot_be_approved_twice() {
    let c = setup();
    let pid = c.client.propose(&c.coordinator, &budget(&c, 1_000_0000000));
    c.client.approve(&c.approver, &pid);
    assert_eq!(
        c.client.try_approve(&c.approver, &pid),
        Err(Ok(Error::InvalidState))
    );
}

#[test]
fn fund_moves_tokens_into_the_treasury() {
    let c = setup();
    let funds = c.client.fund(
        &c.funder,
        &project(&c.env),
        &funder_a(&c.env),
        &1_500_0000000,
    );
    assert_eq!(funds, 1_500_0000000);
    assert_eq!(
        c.client.funds(&project(&c.env), &funder_a(&c.env)),
        1_500_0000000
    );
    assert_eq!(c.client.funds(&project(&c.env), &funder_b(&c.env)), 0);
    assert_eq!(c.token.balance(&c.client.address), 1_500_0000000);
    assert_eq!(c.token.balance(&c.funder), 8_500_0000000);
}

#[test]
fn payment_within_budget_reaches_the_supplier() {
    let c = setup();
    ready(&c);
    let pid = c.client.propose(&c.coordinator, &pay(&c, 300_0000000));
    c.client.approve(&c.approver, &pid);
    // The approval emits the token transfer and Truway's payment event.
    assert!(c.env.events().all().len() >= 2);
    assert_eq!(c.token.balance(&c.supplier), 300_0000000);
    let line = c.client.line(&project(&c.env), &food(&c.env)).unwrap();
    assert_eq!(line.paid, 300_0000000);
    assert_eq!(
        c.client.funds(&project(&c.env), &funder_a(&c.env)),
        1_200_0000000
    );
}

#[test]
fn payment_over_the_budget_line_is_rejected() {
    let c = setup();
    ready(&c);
    // Over the line at proposal time.
    assert_eq!(
        c.client
            .try_propose(&c.coordinator, &pay(&c, 1_200_0000000)),
        Err(Ok(Error::BudgetExceeded))
    );
    // Two proposals that only exceed the line together: the second fails on approval.
    let p1 = c.client.propose(
        &c.coordinator,
        &pay_with(&c, 700_0000000, funder_a(&c.env), id(&c.env, 21)),
    );
    let p2 = c.client.propose(
        &c.coordinator,
        &pay_with(&c, 400_0000000, funder_a(&c.env), id(&c.env, 22)),
    );
    c.client.approve(&c.approver, &p1);
    assert_eq!(
        c.client.try_approve(&c.approver, &p2),
        Err(Ok(Error::BudgetExceeded))
    );
    assert_eq!(c.token.balance(&c.supplier), 700_0000000);
}

#[test]
fn payment_cannot_exceed_project_funds() {
    let c = setup();
    let pid = c.client.propose(&c.coordinator, &budget(&c, 1_000_0000000));
    c.client.approve(&c.approver, &pid);
    c.client
        .fund(&c.funder, &project(&c.env), &funder_a(&c.env), &100_0000000);
    assert_eq!(
        c.client.try_propose(&c.coordinator, &pay(&c, 200_0000000)),
        Err(Ok(Error::InsufficientFunds))
    );
}

#[test]
fn payment_needs_an_existing_line_and_its_chart() {
    let c = setup();
    assert_eq!(
        c.client.try_propose(&c.coordinator, &pay(&c, 1)),
        Err(Ok(Error::LineNotFound))
    );
    ready(&c);
    let mut wrong = match pay(&c, 1) {
        Action::Pay(p) => p,
        _ => unreachable!(),
    };
    wrong.chart = symbol_short!("IFRS");
    assert_eq!(
        c.client.try_propose(&c.coordinator, &Action::Pay(wrong)),
        Err(Ok(Error::ChartMismatch))
    );
}

#[test]
fn invalid_amounts_are_rejected() {
    let c = setup();
    assert_eq!(
        c.client.try_propose(&c.coordinator, &budget(&c, 0)),
        Err(Ok(Error::InvalidAmount))
    );
    assert_eq!(
        c.client
            .try_fund(&c.funder, &project(&c.env), &funder_a(&c.env), &-5),
        Err(Ok(Error::InvalidAmount))
    );
    ready(&c);
    assert_eq!(
        c.client.try_propose(&c.coordinator, &pay(&c, -1)),
        Err(Ok(Error::InvalidAmount))
    );
}

#[test]
fn budget_cannot_drop_below_what_was_paid() {
    let c = setup();
    ready(&c);
    let pid = c.client.propose(&c.coordinator, &pay(&c, 600_0000000));
    c.client.approve(&c.approver, &pid);
    assert_eq!(
        c.client
            .try_propose(&c.coordinator, &budget(&c, 500_0000000)),
        Err(Ok(Error::BudgetBelowPaid))
    );
}

#[test]
fn outsiders_and_roles_are_enforced() {
    let c = setup();
    ready(&c);
    // Not a member.
    assert_eq!(
        c.client.try_propose(&c.funder, &pay(&c, 1)),
        Err(Ok(Error::Unauthorized))
    );
    // Add an operator: may propose payments, may not approve or set budgets.
    let operator = Address::generate(&c.env);
    let pid = c.client.propose(
        &c.coordinator,
        &Action::AddMember(MemberChange {
            member: operator.clone(),
            role: Role::Operator,
        }),
    );
    c.client.approve(&c.approver, &pid);
    let payment = c.client.propose(&operator, &pay(&c, 10_0000000));
    assert_eq!(
        c.client.try_approve(&operator, &payment),
        Err(Ok(Error::Unauthorized))
    );
    assert_eq!(
        c.client.try_propose(&operator, &budget(&c, 2_000_0000000)),
        Err(Ok(Error::Unauthorized))
    );
    c.client.approve(&c.coordinator, &payment);
    assert_eq!(c.token.balance(&c.supplier), 10_0000000);
}

#[test]
fn rejected_proposal_cannot_be_executed() {
    let c = setup();
    ready(&c);
    let pid = c.client.propose(&c.coordinator, &pay(&c, 10_0000000));
    c.client.reject(&c.approver, &pid);
    assert_eq!(
        c.client.proposal(&pid).unwrap().status,
        ProposalStatus::Rejected
    );
    assert_eq!(
        c.client.try_approve(&c.approver, &pid),
        Err(Ok(Error::InvalidState))
    );
    assert_eq!(c.token.balance(&c.supplier), 0);
}

#[test]
fn quorum_cannot_be_lost() {
    let c = setup();
    // Two admins: removing one would leave nobody to approve.
    let pid = c
        .client
        .propose(&c.coordinator, &Action::RemoveMember(c.approver.clone()));
    assert_eq!(
        c.client.try_approve(&c.approver, &pid),
        Err(Ok(Error::QuorumLost))
    );
    // With a third member able to approve, the removal goes through.
    let third = Address::generate(&c.env);
    let add = c.client.propose(
        &c.coordinator,
        &Action::AddMember(MemberChange {
            member: third.clone(),
            role: Role::Approver,
        }),
    );
    c.client.approve(&c.approver, &add);
    let remove = c
        .client
        .propose(&c.coordinator, &Action::RemoveMember(c.approver.clone()));
    c.client.approve(&third, &remove);
    assert_eq!(c.client.role(&c.approver), None);
    assert_eq!(c.client.role(&third), Some(Role::Approver));
}

#[test]
fn funds_of_one_funder_cannot_pay_for_another() {
    let c = setup();
    ready(&c); // 1,500 from funder A
    c.client
        .fund(&c.funder, &project(&c.env), &funder_b(&c.env), &50_0000000);
    // Funder B only earmarked 50 for this project.
    assert_eq!(
        c.client.try_propose(
            &c.coordinator,
            &pay_with(&c, 80_0000000, funder_b(&c.env), id(&c.env, 30))
        ),
        Err(Ok(Error::InsufficientFunds))
    );
    let pid = c.client.propose(
        &c.coordinator,
        &pay_with(&c, 40_0000000, funder_b(&c.env), id(&c.env, 31)),
    );
    c.client.approve(&c.approver, &pid);
    assert_eq!(
        c.client.funds(&project(&c.env), &funder_b(&c.env)),
        10_0000000
    );
    assert_eq!(
        c.client.funds(&project(&c.env), &funder_a(&c.env)),
        1_500_0000000
    );
}

#[test]
fn a_receipt_backs_only_one_payment() {
    let c = setup();
    ready(&c);
    let receipt = id(&c.env, 40);
    let pid = c.client.propose(
        &c.coordinator,
        &pay_with(&c, 10_0000000, funder_a(&c.env), receipt.clone()),
    );
    c.client.approve(&c.approver, &pid);
    assert_eq!(c.client.receipt_use(&receipt), Some(pid));
    c.client
        .fund(&c.funder, &project(&c.env), &funder_b(&c.env), &50_0000000);
    // The same invoice cannot be charged again, to the same or another funder.
    assert_eq!(
        c.client.try_propose(
            &c.coordinator,
            &pay_with(&c, 10_0000000, funder_b(&c.env), receipt.clone())
        ),
        Err(Ok(Error::ReceiptAlreadyUsed))
    );
}

#[test]
fn pending_support_is_allowed_more_than_once() {
    let c = setup();
    ready(&c);
    let none = BytesN::from_array(&c.env, &[0u8; 32]);
    for _ in 0..2 {
        let pid = c.client.propose(
            &c.coordinator,
            &pay_with(&c, 5_0000000, funder_a(&c.env), none.clone()),
        );
        c.client.approve(&c.approver, &pid);
    }
    assert_eq!(c.token.balance(&c.supplier), 10_0000000);
    assert_eq!(c.client.receipt_use(&none), None);
}
