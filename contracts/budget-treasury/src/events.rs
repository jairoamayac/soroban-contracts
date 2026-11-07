use soroban_sdk::{contractevent, Address, BytesN, Symbol};

use crate::types::Role;

/// Topics: `("project_funded", project)`.
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectFunded {
    #[topic]
    pub project: BytesN<32>,
    pub from: Address,
    pub amount: i128,
    pub funds: i128,
}

/// Topics: `("proposal_created", id)`.
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProposalCreated {
    #[topic]
    pub id: u64,
    pub proposer: Address,
    pub kind: Symbol,
}

/// Topics: `("proposal_rejected", id)`.
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProposalRejected {
    #[topic]
    pub id: u64,
    pub rejected_by: Address,
}

/// Topics: `("budget_set", project)`.
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BudgetSet {
    #[topic]
    pub project: BytesN<32>,
    pub line: BytesN<32>,
    pub chart: Symbol,
    pub account: Symbol,
    pub budget: i128,
    pub proposal_id: u64,
    pub proposer: Address,
    pub approver: Address,
}

/// Topics: `("payment_executed", project)`. Carries everything an accountant
/// needs to book the payment and a funder needs to trace it.
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PaymentExecuted {
    #[topic]
    pub project: BytesN<32>,
    pub line: BytesN<32>,
    pub proposal_id: u64,
    pub to: Address,
    pub amount: i128,
    pub chart: Symbol,
    pub account: Symbol,
    pub receipt_hash: BytesN<32>,
    pub reference: BytesN<32>,
    pub proposer: Address,
    pub approver: Address,
    pub line_paid: i128,
    pub line_budget: i128,
}

/// Topics: `("member_added", member)`.
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MemberAdded {
    #[topic]
    pub member: Address,
    pub role: Role,
}

/// Topics: `("member_removed", member)`.
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MemberRemoved {
    #[topic]
    pub member: Address,
}
