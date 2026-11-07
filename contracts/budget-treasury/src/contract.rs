use soroban_sdk::{
    contract, contractimpl, contractmeta, panic_with_error, symbol_short, token, Address, BytesN,
    Env, Map, Symbol, Vec,
};
use truway_common::ttl;

use crate::events::{
    BudgetSet, ContractUpgraded, ExpenseRecorded, MemberAdded, MemberRemoved, PaymentExecuted,
    ProjectFunded, ProposalCreated, ProposalExecuted, ProposalRejected,
};
use crate::storage;
use crate::types::{
    Action, BudgetChange, BudgetLine, Error, Expense, FundKey, LineKey, Payment, Proposal,
    ProposalStatus, Role,
};

// Declares the open standard this contract implements, so indexers, auditors
// and government systems can recognise it without asking Truway.
contractmeta!(key = "event_schema", val = "proof-of-spend/1");

#[contract]
pub struct BudgetTreasury;

fn role_of(env: &Env, who: &Address) -> Option<Role> {
    storage::members(env).get(who.clone())
}

fn kind_of(action: &Action) -> Symbol {
    match action {
        Action::SetBudget(_) => symbol_short!("budget"),
        Action::RecordExpense(_) => symbol_short!("expense"),
        Action::Pay(_) => symbol_short!("pay"),
        Action::AddMember(_) => symbol_short!("add_mbr"),
        Action::RemoveMember(_) => symbol_short!("rm_mbr"),
        Action::Upgrade(_) => symbol_short!("upgrade"),
    }
}

/// Membership and code changes can lock or capture the treasury, so only an
/// admin may propose or approve them.
fn is_sensitive(action: &Action) -> bool {
    matches!(
        action,
        Action::AddMember(_) | Action::RemoveMember(_) | Action::Upgrade(_)
    )
}

fn line_key(project: &BytesN<32>, line: &BytesN<32>, source: &BytesN<32>) -> LineKey {
    LineKey {
        project: project.clone(),
        line: line.clone(),
        source: source.clone(),
    }
}

/// Checks shared by expenses and payments: a supporting document that was
/// never used, an existing line in the same chart, and room in its budget.
fn check_spend(
    env: &Env,
    key: &LineKey,
    amount: i128,
    chart: &Symbol,
    receipt_hash: &BytesN<32>,
) -> Result<BudgetLine, Error> {
    if amount <= 0 {
        return Err(Error::InvalidAmount);
    }
    if receipt_hash.to_array() == [0u8; 32] {
        return Err(Error::ReceiptRequired);
    }
    if storage::receipt_use(env, receipt_hash).is_some() {
        return Err(Error::ReceiptAlreadyUsed);
    }
    let line = storage::line(env, key).ok_or(Error::LineNotFound)?;
    if &line.chart != chart {
        return Err(Error::ChartMismatch);
    }
    let new_paid = line.paid.checked_add(amount).ok_or(Error::InvalidAmount)?;
    if new_paid > line.budget {
        return Err(Error::BudgetExceeded);
    }
    Ok(line)
}

fn check_expense(env: &Env, x: &Expense) -> Result<BudgetLine, Error> {
    let key = line_key(&x.project, &x.line, &x.source);
    check_spend(env, &key, x.amount, &x.chart, &x.receipt_hash)
}

/// A payment also needs its line to be in the treasury asset and enough funds
/// from its source for this project.
fn check_payment(env: &Env, p: &Payment) -> Result<BudgetLine, Error> {
    let key = line_key(&p.project, &p.line, &p.source);
    let line = check_spend(env, &key, p.amount, &p.chart, &p.receipt_hash)?;
    if line.currency != storage::token_symbol(env) {
        return Err(Error::CurrencyMismatch);
    }
    let fund = FundKey {
        project: p.project.clone(),
        source: p.source.clone(),
    };
    if storage::funds(env, &fund) < p.amount {
        return Err(Error::InsufficientFunds);
    }
    Ok(line)
}

fn check_budget(env: &Env, b: &BudgetChange) -> Result<(), Error> {
    if b.amount <= 0 {
        return Err(Error::InvalidAmount);
    }
    if let Some(l) = storage::line(env, &line_key(&b.project, &b.line, &b.source)) {
        if b.amount < l.paid {
            return Err(Error::BudgetBelowPaid);
        }
        if l.paid > 0 && l.currency != b.currency {
            return Err(Error::CurrencyMismatch);
        }
    }
    Ok(())
}

/// The treasury must always keep at least one admin and two members able to
/// approve; otherwise no proposal could ever pass again.
fn keeps_quorum(members: &Map<Address, Role>) -> bool {
    let admins = members
        .values()
        .iter()
        .filter(|r| *r == Role::Admin)
        .count();
    let deciders = members
        .values()
        .iter()
        .filter(|r| *r == Role::Admin || *r == Role::Approver)
        .count();
    admins >= 1 && deciders >= 2
}

#[contractimpl]
impl BudgetTreasury {
    /// Creates the treasury for one organisation. `admins` are the
    /// organisation's own people; Truway, as deployer, is not a member.
    /// `token` is the asset held on the ledger and `token_symbol` its code
    /// (e.g. `USDC`), which payment budget lines must use.
    pub fn __constructor(env: Env, admins: Vec<Address>, token: Address, token_symbol: Symbol) {
        if admins.is_empty() {
            panic_with_error!(&env, Error::NoAdmins);
        }
        let mut members = Map::new(&env);
        for a in admins.iter() {
            members.set(a, Role::Admin);
        }
        storage::set_members(&env, &members);
        storage::set_token(&env, &token, &token_symbol);
        ttl::extend_instance(&env);
    }

    /// Moves `amount` of the treasury asset from `from` into the treasury,
    /// earmarked for `project` by funding `source`. Anyone may fund (e.g. the
    /// funder directly). Returns what this source now holds for the project.
    pub fn fund(
        env: Env,
        from: Address,
        project: BytesN<32>,
        source: BytesN<32>,
        amount: i128,
    ) -> Result<i128, Error> {
        from.require_auth();
        if amount <= 0 {
            return Err(Error::InvalidAmount);
        }
        token::TokenClient::new(&env, &storage::token(&env)).transfer(
            &from,
            env.current_contract_address(),
            &amount,
        );
        let key = FundKey {
            project: project.clone(),
            source: source.clone(),
        };
        let funds = storage::funds(&env, &key)
            .checked_add(amount)
            .ok_or(Error::InvalidAmount)?;
        storage::set_funds(&env, &key, funds);
        ttl::extend_instance(&env);
        ProjectFunded {
            project,
            source,
            from,
            amount,
            funds,
        }
        .publish(&env);
        Ok(funds)
    }

    /// Opens a proposal. Admins may propose anything; operators may propose
    /// expenses and payments; approvers may not propose.
    pub fn propose(env: Env, proposer: Address, action: Action) -> Result<u64, Error> {
        proposer.require_auth();
        let role = role_of(&env, &proposer).ok_or(Error::Unauthorized)?;
        match &action {
            Action::RecordExpense(x) => {
                if role == Role::Approver {
                    return Err(Error::Unauthorized);
                }
                check_expense(&env, x)?;
            }
            Action::Pay(p) => {
                if role == Role::Approver {
                    return Err(Error::Unauthorized);
                }
                check_payment(&env, p)?;
            }
            Action::SetBudget(b) => {
                if role != Role::Admin {
                    return Err(Error::Unauthorized);
                }
                check_budget(&env, b)?;
            }
            Action::AddMember(_) | Action::RemoveMember(_) | Action::Upgrade(_) => {
                if role != Role::Admin {
                    return Err(Error::Unauthorized);
                }
            }
        }
        let id = storage::next_id(&env);
        let kind = kind_of(&action);
        let proposal = Proposal {
            id,
            proposer: proposer.clone(),
            action,
            status: ProposalStatus::Pending,
            decided_by: None,
            created_at: env.ledger().timestamp(),
            decided_at: 0,
        };
        storage::set_proposal(&env, &proposal);
        ttl::extend_instance(&env);
        ProposalCreated { id, proposer, kind }.publish(&env);
        Ok(id)
    }

    /// Approves and executes a proposal. The approver is never the proposer;
    /// membership and code changes need an admin approver.
    pub fn approve(env: Env, approver: Address, id: u64) -> Result<(), Error> {
        approver.require_auth();
        let role = role_of(&env, &approver).ok_or(Error::Unauthorized)?;
        if role == Role::Operator {
            return Err(Error::Unauthorized);
        }
        let mut proposal = storage::proposal(&env, id).ok_or(Error::NotFound)?;
        if proposal.status != ProposalStatus::Pending {
            return Err(Error::InvalidState);
        }
        if proposal.proposer == approver {
            return Err(Error::SelfApproval);
        }
        if is_sensitive(&proposal.action) && role != Role::Admin {
            return Err(Error::Unauthorized);
        }
        Self::execute(&env, &proposal, &approver)?;
        proposal.status = ProposalStatus::Executed;
        proposal.decided_by = Some(approver.clone());
        proposal.decided_at = env.ledger().timestamp();
        storage::set_proposal(&env, &proposal);
        ttl::extend_instance(&env);
        ProposalExecuted {
            id,
            kind: kind_of(&proposal.action),
            proposer: proposal.proposer.clone(),
            approver,
        }
        .publish(&env);
        Ok(())
    }

    /// Rejects a pending proposal. Admins and approvers may reject what they
    /// could approve; anyone may withdraw their own proposal.
    pub fn reject(env: Env, member: Address, id: u64) -> Result<(), Error> {
        member.require_auth();
        let role = role_of(&env, &member).ok_or(Error::Unauthorized)?;
        let mut proposal = storage::proposal(&env, id).ok_or(Error::NotFound)?;
        if proposal.status != ProposalStatus::Pending {
            return Err(Error::InvalidState);
        }
        let own = proposal.proposer == member;
        let may_decide = match role {
            Role::Admin => true,
            Role::Approver => !is_sensitive(&proposal.action),
            Role::Operator => false,
        };
        if !own && !may_decide {
            return Err(Error::Unauthorized);
        }
        proposal.status = ProposalStatus::Rejected;
        proposal.decided_by = Some(member.clone());
        proposal.decided_at = env.ledger().timestamp();
        storage::set_proposal(&env, &proposal);
        ttl::extend_instance(&env);
        ProposalRejected {
            id,
            rejected_by: member,
        }
        .publish(&env);
        Ok(())
    }

    fn execute(env: &Env, proposal: &Proposal, approver: &Address) -> Result<(), Error> {
        match &proposal.action {
            Action::SetBudget(b) => {
                check_budget(env, b)?;
                let key = line_key(&b.project, &b.line, &b.source);
                let paid = storage::line(env, &key).map(|l| l.paid).unwrap_or(0);
                storage::set_line(
                    env,
                    &key,
                    &BudgetLine {
                        chart: b.chart.clone(),
                        account: b.account.clone(),
                        currency: b.currency.clone(),
                        budget: b.amount,
                        paid,
                    },
                );
                BudgetSet {
                    project: b.project.clone(),
                    line: b.line.clone(),
                    source: b.source.clone(),
                    chart: b.chart.clone(),
                    account: b.account.clone(),
                    currency: b.currency.clone(),
                    budget: b.amount,
                    proposal_id: proposal.id,
                    proposer: proposal.proposer.clone(),
                    approver: approver.clone(),
                }
                .publish(env);
            }
            Action::RecordExpense(x) => {
                let mut line = check_expense(env, x)?;
                line.paid += x.amount;
                storage::set_line(env, &line_key(&x.project, &x.line, &x.source), &line);
                storage::set_receipt_use(env, &x.receipt_hash, proposal.id);
                ExpenseRecorded {
                    project: x.project.clone(),
                    line: x.line.clone(),
                    source: x.source.clone(),
                    proposal_id: proposal.id,
                    amount: x.amount,
                    currency: line.currency.clone(),
                    chart: x.chart.clone(),
                    account: x.account.clone(),
                    receipt_hash: x.receipt_hash.clone(),
                    payment_ref: x.payment_ref.clone(),
                    payee_ref: x.payee_ref.clone(),
                    proposer: proposal.proposer.clone(),
                    approver: approver.clone(),
                    line_paid: line.paid,
                    line_budget: line.budget,
                }
                .publish(env);
            }
            Action::Pay(p) => {
                let mut line = check_payment(env, p)?;
                token::TokenClient::new(env, &storage::token(env)).transfer(
                    &env.current_contract_address(),
                    &p.to,
                    &p.amount,
                );
                line.paid += p.amount;
                storage::set_line(env, &line_key(&p.project, &p.line, &p.source), &line);
                let fund = FundKey {
                    project: p.project.clone(),
                    source: p.source.clone(),
                };
                let source_funds = storage::funds(env, &fund) - p.amount;
                storage::set_funds(env, &fund, source_funds);
                storage::set_receipt_use(env, &p.receipt_hash, proposal.id);
                PaymentExecuted {
                    project: p.project.clone(),
                    line: p.line.clone(),
                    source: p.source.clone(),
                    proposal_id: proposal.id,
                    to: p.to.clone(),
                    amount: p.amount,
                    currency: line.currency.clone(),
                    chart: p.chart.clone(),
                    account: p.account.clone(),
                    receipt_hash: p.receipt_hash.clone(),
                    reference: p.reference.clone(),
                    proposer: proposal.proposer.clone(),
                    approver: approver.clone(),
                    line_paid: line.paid,
                    line_budget: line.budget,
                    source_funds,
                }
                .publish(env);
            }
            Action::AddMember(m) => {
                let mut members = storage::members(env);
                members.set(m.member.clone(), m.role);
                // Adding never breaks quorum, but changing an existing role can.
                if !keeps_quorum(&members) {
                    return Err(Error::QuorumLost);
                }
                storage::set_members(env, &members);
                MemberAdded {
                    member: m.member.clone(),
                    role: m.role,
                }
                .publish(env);
            }
            Action::RemoveMember(member) => {
                let mut members = storage::members(env);
                if !members.contains_key(member.clone()) {
                    return Err(Error::NotFound);
                }
                members.remove(member.clone());
                if !keeps_quorum(&members) {
                    return Err(Error::QuorumLost);
                }
                storage::set_members(env, &members);
                MemberRemoved {
                    member: member.clone(),
                }
                .publish(env);
            }
            Action::Upgrade(hash) => {
                ContractUpgraded {
                    proposal_id: proposal.id,
                    wasm_hash: hash.clone(),
                    proposer: proposal.proposer.clone(),
                    approver: approver.clone(),
                }
                .publish(env);
                env.deployer().update_current_contract_wasm(hash.clone());
            }
        }
        Ok(())
    }

    // ---- Reads ----

    pub fn token(env: Env) -> Address {
        storage::token(&env)
    }

    pub fn token_symbol(env: Env) -> Symbol {
        storage::token_symbol(&env)
    }

    pub fn role(env: Env, member: Address) -> Option<Role> {
        role_of(&env, &member)
    }

    pub fn members(env: Env) -> Map<Address, Role> {
        storage::members(&env)
    }

    pub fn line(
        env: Env,
        project: BytesN<32>,
        line: BytesN<32>,
        source: BytesN<32>,
    ) -> Option<BudgetLine> {
        storage::line(&env, &line_key(&project, &line, &source))
    }

    pub fn funds(env: Env, project: BytesN<32>, source: BytesN<32>) -> i128 {
        storage::funds(&env, &FundKey { project, source })
    }

    /// Proposal id of the executed expense or payment a supporting document backs.
    pub fn receipt_use(env: Env, receipt_hash: BytesN<32>) -> Option<u64> {
        storage::receipt_use(&env, &receipt_hash)
    }

    pub fn proposal(env: Env, id: u64) -> Option<Proposal> {
        storage::proposal(&env, id)
    }
}
