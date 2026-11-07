use soroban_sdk::{
    contract, contractimpl, panic_with_error, symbol_short, token, Address, BytesN, Env, Map,
    Symbol, Vec,
};
use truway_common::ttl;

use crate::events::{
    BudgetSet, MemberAdded, MemberRemoved, PaymentExecuted, ProjectFunded, ProposalCreated,
    ProposalRejected,
};
use crate::storage;
use crate::types::{Action, BudgetLine, Error, LineKey, Payment, Proposal, ProposalStatus, Role};

#[contract]
pub struct BudgetTreasury;

fn role_of(env: &Env, who: &Address) -> Option<Role> {
    storage::members(env).get(who.clone())
}

fn require_admin(role: Role) -> Result<(), Error> {
    if role == Role::Admin {
        Ok(())
    } else {
        Err(Error::Unauthorized)
    }
}

/// Checks a payment against its budget line and project funds. Runs when the
/// payment is proposed and again when it is approved, since other payments may
/// have been executed in between.
fn check_payment(env: &Env, p: &Payment) -> Result<BudgetLine, Error> {
    if p.amount <= 0 {
        return Err(Error::InvalidAmount);
    }
    let key = LineKey {
        project: p.project.clone(),
        line: p.line.clone(),
    };
    let line = storage::line(env, &key).ok_or(Error::LineNotFound)?;
    if line.chart != p.chart {
        return Err(Error::ChartMismatch);
    }
    let new_paid = line
        .paid
        .checked_add(p.amount)
        .ok_or(Error::InvalidAmount)?;
    if new_paid > line.budget {
        return Err(Error::BudgetExceeded);
    }
    if storage::funds(env, &p.project) < p.amount {
        return Err(Error::InsufficientFunds);
    }
    Ok(line)
}

/// After a removal there must be at least one admin and two members able to
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
    pub fn __constructor(env: Env, admins: Vec<Address>, token: Address) {
        if admins.is_empty() {
            panic_with_error!(&env, Error::NoAdmins);
        }
        let mut members = Map::new(&env);
        for a in admins.iter() {
            members.set(a, Role::Admin);
        }
        storage::set_members(&env, &members);
        storage::set_token(&env, &token);
        ttl::extend_instance(&env);
    }

    /// Moves `amount` of the treasury asset from `from` into the treasury and
    /// assigns it to `project`. Anyone may fund (e.g. the funder directly).
    pub fn fund(env: Env, from: Address, project: BytesN<32>, amount: i128) -> Result<i128, Error> {
        from.require_auth();
        if amount <= 0 {
            return Err(Error::InvalidAmount);
        }
        token::TokenClient::new(&env, &storage::token(&env)).transfer(
            &from,
            env.current_contract_address(),
            &amount,
        );
        let funds = storage::funds(&env, &project)
            .checked_add(amount)
            .ok_or(Error::InvalidAmount)?;
        storage::set_funds(&env, &project, funds);
        ttl::extend_instance(&env);
        ProjectFunded {
            project,
            from,
            amount,
            funds,
        }
        .publish(&env);
        Ok(funds)
    }

    /// Opens a proposal. Admins may propose anything; operators may propose
    /// payments only; approvers may not propose.
    pub fn propose(env: Env, proposer: Address, action: Action) -> Result<u64, Error> {
        proposer.require_auth();
        let role = role_of(&env, &proposer).ok_or(Error::Unauthorized)?;
        let kind: Symbol = match &action {
            Action::Pay(p) => {
                if role == Role::Approver {
                    return Err(Error::Unauthorized);
                }
                check_payment(&env, p)?;
                symbol_short!("pay")
            }
            Action::SetBudget(b) => {
                require_admin(role)?;
                if b.amount <= 0 {
                    return Err(Error::InvalidAmount);
                }
                let key = LineKey {
                    project: b.project.clone(),
                    line: b.line.clone(),
                };
                if let Some(l) = storage::line(&env, &key) {
                    if b.amount < l.paid {
                        return Err(Error::BudgetBelowPaid);
                    }
                }
                symbol_short!("budget")
            }
            Action::AddMember(_) => {
                require_admin(role)?;
                symbol_short!("add_mbr")
            }
            Action::RemoveMember(_) => {
                require_admin(role)?;
                symbol_short!("rm_mbr")
            }
            Action::Upgrade(_) => {
                require_admin(role)?;
                symbol_short!("upgrade")
            }
        };
        let id = storage::next_id(&env);
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

    /// Approves and executes a proposal. The approver must be an admin or an
    /// approver, and never the member who proposed it.
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
        Self::execute(&env, &proposal, &approver)?;
        proposal.status = ProposalStatus::Executed;
        proposal.decided_by = Some(approver);
        proposal.decided_at = env.ledger().timestamp();
        storage::set_proposal(&env, &proposal);
        ttl::extend_instance(&env);
        Ok(())
    }

    /// Rejects a pending proposal. Admins and approvers may reject, including
    /// the proposer withdrawing their own proposal.
    pub fn reject(env: Env, member: Address, id: u64) -> Result<(), Error> {
        member.require_auth();
        let role = role_of(&env, &member).ok_or(Error::Unauthorized)?;
        let mut proposal = storage::proposal(&env, id).ok_or(Error::NotFound)?;
        if role == Role::Operator && proposal.proposer != member {
            return Err(Error::Unauthorized);
        }
        if proposal.status != ProposalStatus::Pending {
            return Err(Error::InvalidState);
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
                let key = LineKey {
                    project: b.project.clone(),
                    line: b.line.clone(),
                };
                let paid = storage::line(env, &key).map(|l| l.paid).unwrap_or(0);
                if b.amount < paid {
                    return Err(Error::BudgetBelowPaid);
                }
                storage::set_line(
                    env,
                    &key,
                    &BudgetLine {
                        chart: b.chart.clone(),
                        account: b.account.clone(),
                        budget: b.amount,
                        paid,
                    },
                );
                BudgetSet {
                    project: b.project.clone(),
                    line: b.line.clone(),
                    chart: b.chart.clone(),
                    account: b.account.clone(),
                    budget: b.amount,
                    proposal_id: proposal.id,
                    proposer: proposal.proposer.clone(),
                    approver: approver.clone(),
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
                let key = LineKey {
                    project: p.project.clone(),
                    line: p.line.clone(),
                };
                storage::set_line(env, &key, &line);
                let funds = storage::funds(env, &p.project) - p.amount;
                storage::set_funds(env, &p.project, funds);
                PaymentExecuted {
                    project: p.project.clone(),
                    line: p.line.clone(),
                    proposal_id: proposal.id,
                    to: p.to.clone(),
                    amount: p.amount,
                    chart: p.chart.clone(),
                    account: p.account.clone(),
                    receipt_hash: p.receipt_hash.clone(),
                    reference: p.reference.clone(),
                    proposer: proposal.proposer.clone(),
                    approver: approver.clone(),
                    line_paid: line.paid,
                    line_budget: line.budget,
                }
                .publish(env);
            }
            Action::AddMember(m) => {
                let mut members = storage::members(env);
                members.set(m.member.clone(), m.role);
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
                env.deployer().update_current_contract_wasm(hash.clone());
            }
        }
        Ok(())
    }

    // ---- Reads ----

    pub fn token(env: Env) -> Address {
        storage::token(&env)
    }

    pub fn role(env: Env, member: Address) -> Option<Role> {
        role_of(&env, &member)
    }

    pub fn members(env: Env) -> Map<Address, Role> {
        storage::members(&env)
    }

    pub fn line(env: Env, project: BytesN<32>, line: BytesN<32>) -> Option<BudgetLine> {
        storage::line(&env, &LineKey { project, line })
    }

    pub fn funds(env: Env, project: BytesN<32>) -> i128 {
        storage::funds(&env, &project)
    }

    pub fn proposal(env: Env, id: u64) -> Option<Proposal> {
        storage::proposal(&env, id)
    }
}
