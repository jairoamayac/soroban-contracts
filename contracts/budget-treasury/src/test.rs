// Amounts are fixed point with 7 decimals: 1_000_0000000 is 1,000 units.
#![allow(clippy::inconsistent_digit_grouping)]

use crate::{
    Action, BudgetChange, BudgetTreasury, BudgetTreasuryClient, Error, Expense, MemberChange,
    Payment, ProposalStatus, Role,
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
/// The food line of the Soacha team.
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
    let sac = env.register_stellar_asset_contract_v2(Address::generate(&env));
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
            symbol_short!("USDC"),
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

fn budget(c: &Ctx, source: BytesN<32>, currency: Symbol, amount: i128) -> Action {
    Action::SetBudget(BudgetChange {
        project: project(&c.env),
        line: food(&c.env),
        source,
        chart: symbol_short!("CO_PUC"),
        account: Symbol::new(&c.env, "5135"),
        currency,
        amount,
    })
}

fn expense(c: &Ctx, source: BytesN<32>, amount: i128, receipt: BytesN<32>) -> Action {
    Action::RecordExpense(Expense {
        project: project(&c.env),
        line: food(&c.env),
        source,
        amount,
        chart: symbol_short!("CO_PUC"),
        account: Symbol::new(&c.env, "513595"),
        receipt_hash: receipt,
        payment_ref: id(&c.env, 50),
        payee_ref: id(&c.env, 51),
    })
}

fn pay(c: &Ctx, source: BytesN<32>, amount: i128, receipt: BytesN<32>) -> Action {
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

/// Approves an action proposed by the coordinator.
fn pass(c: &Ctx, action: &Action) -> u64 {
    let pid = c.client.propose(&c.coordinator, action);
    c.client.approve(&c.approver, &pid);
    pid
}

/// Food line: 700,000 COP from funder A and 200,000 COP from funder B.
fn cop_lines(c: &Ctx) {
    pass(
        c,
        &budget(c, funder_a(&c.env), symbol_short!("COP"), 700_000_0000000),
    );
    pass(
        c,
        &budget(c, funder_b(&c.env), symbol_short!("COP"), 200_000_0000000),
    );
}

/// Food line in USDC from funder A: budget 1,000 and 1,500 of funds.
fn usdc_line(c: &Ctx) {
    pass(
        c,
        &budget(c, funder_a(&c.env), symbol_short!("USDC"), 1_000_0000000),
    );
    c.client.fund(
        &c.funder,
        &project(&c.env),
        &funder_a(&c.env),
        &1_500_0000000,
    );
}

fn add(c: &Ctx, member: &Address, role: Role) {
    pass(
        c,
        &Action::AddMember(MemberChange {
            member: member.clone(),
            role,
        }),
    );
}

// ---- Construction and authorization ----

#[test]
fn constructor_sets_admins_and_no_one_else() {
    let c = setup();
    assert_eq!(c.client.role(&c.coordinator), Some(Role::Admin));
    assert_eq!(c.client.role(&c.approver), Some(Role::Admin));
    assert_eq!(c.client.role(&c.funder), None);
    assert_eq!(c.client.token_symbol(), symbol_short!("USDC"));
}

#[test]
fn each_call_is_authorized_by_the_right_address() {
    let c = setup();
    let pid = c.client.propose(
        &c.coordinator,
        &budget(&c, funder_a(&c.env), symbol_short!("COP"), 1),
    );
    assert_eq!(c.env.auths()[0].0, c.coordinator);
    c.client.approve(&c.approver, &pid);
    assert_eq!(c.env.auths()[0].0, c.approver);
    c.client
        .fund(&c.funder, &project(&c.env), &funder_a(&c.env), &1);
    assert_eq!(c.env.auths()[0].0, c.funder);
    let other = c.client.propose(
        &c.coordinator,
        &budget(&c, funder_b(&c.env), symbol_short!("COP"), 1),
    );
    c.client.reject(&c.approver, &other);
    assert_eq!(c.env.auths()[0].0, c.approver);
}

#[test]
fn self_approval_is_rejected() {
    let c = setup();
    let pid = c.client.propose(
        &c.coordinator,
        &budget(&c, funder_a(&c.env), symbol_short!("COP"), 1),
    );
    assert_eq!(
        c.client.try_approve(&c.coordinator, &pid),
        Err(Ok(Error::SelfApproval))
    );
}

#[test]
fn proposal_cannot_be_approved_twice() {
    let c = setup();
    let pid = pass(&c, &budget(&c, funder_a(&c.env), symbol_short!("COP"), 1));
    assert_eq!(
        c.client.try_approve(&c.approver, &pid),
        Err(Ok(Error::InvalidState))
    );
    assert_eq!(
        c.client.proposal(&pid).unwrap().status,
        ProposalStatus::Executed
    );
}

// ---- Budgets by line and source ----

#[test]
fn budget_is_kept_per_line_and_source() {
    let c = setup();
    cop_lines(&c);
    let a = c
        .client
        .line(&project(&c.env), &food(&c.env), &funder_a(&c.env))
        .unwrap();
    let b = c
        .client
        .line(&project(&c.env), &food(&c.env), &funder_b(&c.env))
        .unwrap();
    assert_eq!(a.budget, 700_000_0000000);
    assert_eq!(b.budget, 200_000_0000000);
    assert_eq!(a.currency, symbol_short!("COP"));
}

#[test]
fn one_funder_never_pays_for_another() {
    let c = setup();
    cop_lines(&c);
    // Funder B only has 200,000 in the food line.
    assert_eq!(
        c.client.try_propose(
            &c.coordinator,
            &expense(&c, funder_b(&c.env), 300_000_0000000, id(&c.env, 20))
        ),
        Err(Ok(Error::BudgetExceeded))
    );
    pass(
        &c,
        &expense(&c, funder_a(&c.env), 300_000_0000000, id(&c.env, 20)),
    );
    let a = c
        .client
        .line(&project(&c.env), &food(&c.env), &funder_a(&c.env))
        .unwrap();
    let b = c
        .client
        .line(&project(&c.env), &food(&c.env), &funder_b(&c.env))
        .unwrap();
    assert_eq!(a.paid, 300_000_0000000);
    assert_eq!(b.paid, 0);
}

#[test]
fn budget_cannot_drop_below_what_was_spent() {
    let c = setup();
    cop_lines(&c);
    pass(
        &c,
        &expense(&c, funder_a(&c.env), 600_000_0000000, id(&c.env, 21)),
    );
    assert_eq!(
        c.client.try_propose(
            &c.coordinator,
            &budget(&c, funder_a(&c.env), symbol_short!("COP"), 500_000_0000000)
        ),
        Err(Ok(Error::BudgetBelowPaid))
    );
}

// ---- Expenses paid outside the ledger ----

#[test]
fn recording_an_expense_moves_no_funds() {
    let c = setup();
    cop_lines(&c);
    let operator = Address::generate(&c.env);
    add(&c, &operator, Role::Operator);
    let pid = c.client.propose(
        &operator,
        &expense(&c, funder_a(&c.env), 45_000_0000000, id(&c.env, 22)),
    );
    c.client.approve(&c.approver, &pid);
    assert!(c.env.events().all().len() >= 2);
    assert_eq!(c.token.balance(&c.client.address), 0);
    assert_eq!(c.client.receipt_use(&id(&c.env, 22)), Some(pid));
}

#[test]
fn concurrent_expenses_cannot_exceed_the_line_together() {
    let c = setup();
    cop_lines(&c);
    let p1 = c.client.propose(
        &c.coordinator,
        &expense(&c, funder_b(&c.env), 150_000_0000000, id(&c.env, 23)),
    );
    let p2 = c.client.propose(
        &c.coordinator,
        &expense(&c, funder_b(&c.env), 100_000_0000000, id(&c.env, 24)),
    );
    c.client.approve(&c.approver, &p1);
    assert_eq!(
        c.client.try_approve(&c.approver, &p2),
        Err(Ok(Error::BudgetExceeded))
    );
}

#[test]
fn a_receipt_is_required_and_backs_only_one_operation() {
    let c = setup();
    cop_lines(&c);
    let none = BytesN::from_array(&c.env, &[0u8; 32]);
    assert_eq!(
        c.client
            .try_propose(&c.coordinator, &expense(&c, funder_a(&c.env), 1, none)),
        Err(Ok(Error::ReceiptRequired))
    );
    let receipt = id(&c.env, 25);
    pass(&c, &expense(&c, funder_a(&c.env), 1, receipt.clone()));
    // The same invoice cannot be charged again, to the same or another funder.
    assert_eq!(
        c.client
            .try_propose(&c.coordinator, &expense(&c, funder_b(&c.env), 1, receipt)),
        Err(Ok(Error::ReceiptAlreadyUsed))
    );
}

#[test]
fn an_expense_books_to_the_chart_of_its_line() {
    let c = setup();
    cop_lines(&c);
    let mut wrong = match expense(&c, funder_a(&c.env), 1, id(&c.env, 26)) {
        Action::RecordExpense(x) => x,
        _ => unreachable!(),
    };
    wrong.chart = symbol_short!("IFRS");
    assert_eq!(
        c.client
            .try_propose(&c.coordinator, &Action::RecordExpense(wrong)),
        Err(Ok(Error::ChartMismatch))
    );
    assert_eq!(
        c.client.try_propose(
            &c.coordinator,
            &Action::RecordExpense(Expense {
                project: id(&c.env, 99),
                line: food(&c.env),
                source: funder_a(&c.env),
                amount: 1,
                chart: symbol_short!("CO_PUC"),
                account: Symbol::new(&c.env, "5135"),
                receipt_hash: id(&c.env, 27),
                payment_ref: id(&c.env, 50),
                payee_ref: id(&c.env, 51),
            })
        ),
        Err(Ok(Error::LineNotFound))
    );
}

// ---- Payments on the ledger ----

#[test]
fn fund_moves_tokens_into_the_treasury_by_source() {
    let c = setup();
    let funds = c.client.fund(
        &c.funder,
        &project(&c.env),
        &funder_a(&c.env),
        &1_500_0000000,
    );
    assert_eq!(funds, 1_500_0000000);
    assert_eq!(c.client.funds(&project(&c.env), &funder_b(&c.env)), 0);
    assert_eq!(c.token.balance(&c.client.address), 1_500_0000000);
}

#[test]
fn payment_within_budget_reaches_the_supplier() {
    let c = setup();
    usdc_line(&c);
    pass(&c, &pay(&c, funder_a(&c.env), 300_0000000, id(&c.env, 30)));
    assert_eq!(c.token.balance(&c.supplier), 300_0000000);
    assert_eq!(
        c.client.funds(&project(&c.env), &funder_a(&c.env)),
        1_200_0000000
    );
}

#[test]
fn payment_needs_a_line_in_the_treasury_asset() {
    let c = setup();
    cop_lines(&c);
    c.client.fund(
        &c.funder,
        &project(&c.env),
        &funder_a(&c.env),
        &1_000_0000000,
    );
    assert_eq!(
        c.client.try_propose(
            &c.coordinator,
            &pay(&c, funder_a(&c.env), 1, id(&c.env, 31))
        ),
        Err(Ok(Error::CurrencyMismatch))
    );
}

#[test]
fn payment_cannot_exceed_the_funds_of_its_source() {
    let c = setup();
    pass(
        &c,
        &budget(&c, funder_a(&c.env), symbol_short!("USDC"), 1_000_0000000),
    );
    c.client
        .fund(&c.funder, &project(&c.env), &funder_a(&c.env), &100_0000000);
    assert_eq!(
        c.client.try_propose(
            &c.coordinator,
            &pay(&c, funder_a(&c.env), 200_0000000, id(&c.env, 32))
        ),
        Err(Ok(Error::InsufficientFunds))
    );
}

#[test]
fn invalid_amounts_are_rejected() {
    let c = setup();
    assert_eq!(
        c.client.try_propose(
            &c.coordinator,
            &budget(&c, funder_a(&c.env), symbol_short!("COP"), 0)
        ),
        Err(Ok(Error::InvalidAmount))
    );
    assert_eq!(
        c.client
            .try_fund(&c.funder, &project(&c.env), &funder_a(&c.env), &-5),
        Err(Ok(Error::InvalidAmount))
    );
}

// ---- Roles and governance ----

#[test]
fn roles_are_enforced() {
    let c = setup();
    cop_lines(&c);
    let operator = Address::generate(&c.env);
    let checker = Address::generate(&c.env);
    add(&c, &operator, Role::Operator);
    add(&c, &checker, Role::Approver);
    // Outsiders cannot propose.
    assert_eq!(
        c.client
            .try_propose(&c.funder, &expense(&c, funder_a(&c.env), 1, id(&c.env, 40))),
        Err(Ok(Error::Unauthorized))
    );
    // Operators propose expenses but cannot approve or set budgets.
    let pid = c
        .client
        .propose(&operator, &expense(&c, funder_a(&c.env), 1, id(&c.env, 41)));
    assert_eq!(
        c.client.try_approve(&operator, &pid),
        Err(Ok(Error::Unauthorized))
    );
    assert_eq!(
        c.client.try_propose(
            &operator,
            &budget(&c, funder_a(&c.env), symbol_short!("COP"), 1)
        ),
        Err(Ok(Error::Unauthorized))
    );
    // Approvers approve expenses but do not propose.
    c.client.approve(&checker, &pid);
    assert_eq!(
        c.client
            .try_propose(&checker, &expense(&c, funder_a(&c.env), 1, id(&c.env, 42))),
        Err(Ok(Error::Unauthorized))
    );
}

#[test]
fn only_an_admin_approves_membership_and_code_changes() {
    let c = setup();
    let checker = Address::generate(&c.env);
    add(&c, &checker, Role::Approver);
    let puppet = Address::generate(&c.env);
    let pid = c.client.propose(
        &c.coordinator,
        &Action::AddMember(MemberChange {
            member: puppet.clone(),
            role: Role::Admin,
        }),
    );
    assert_eq!(
        c.client.try_approve(&checker, &pid),
        Err(Ok(Error::Unauthorized))
    );
    assert_eq!(
        c.client.try_reject(&checker, &pid),
        Err(Ok(Error::Unauthorized))
    );
    let upgrade = c
        .client
        .propose(&c.coordinator, &Action::Upgrade(id(&c.env, 77)));
    assert_eq!(
        c.client.try_approve(&checker, &upgrade),
        Err(Ok(Error::Unauthorized))
    );
}

#[test]
fn quorum_cannot_be_lost_by_removal_or_role_change() {
    let c = setup();
    // Two admins: removing or demoting one would leave nobody to approve.
    let remove = c
        .client
        .propose(&c.coordinator, &Action::RemoveMember(c.approver.clone()));
    assert_eq!(
        c.client.try_approve(&c.approver, &remove),
        Err(Ok(Error::QuorumLost))
    );
    let demote = c.client.propose(
        &c.coordinator,
        &Action::AddMember(MemberChange {
            member: c.approver.clone(),
            role: Role::Operator,
        }),
    );
    assert_eq!(
        c.client.try_approve(&c.approver, &demote),
        Err(Ok(Error::QuorumLost))
    );
    // With a third member able to approve, the change goes through.
    let third = Address::generate(&c.env);
    add(&c, &third, Role::Approver);
    let demote_again = c.client.propose(
        &c.coordinator,
        &Action::AddMember(MemberChange {
            member: c.approver.clone(),
            role: Role::Operator,
        }),
    );
    c.client.approve(&c.approver, &demote_again);
    assert_eq!(c.client.role(&c.approver), Some(Role::Operator));
}

#[test]
fn rejected_proposal_cannot_be_executed() {
    let c = setup();
    cop_lines(&c);
    let pid = c.client.propose(
        &c.coordinator,
        &expense(&c, funder_a(&c.env), 1, id(&c.env, 43)),
    );
    c.client.reject(&c.approver, &pid);
    assert_eq!(
        c.client.try_approve(&c.approver, &pid),
        Err(Ok(Error::InvalidState))
    );
    assert_eq!(c.client.receipt_use(&id(&c.env, 43)), None);
}
