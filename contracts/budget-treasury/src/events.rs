//! Events of the Proof of Spend open standard (`proof-of-spend/1`).
//! See `docs/proof-of-spend.md`. Names and fields are part of the standard.

use soroban_sdk::{contractevent, Address, BytesN, Symbol};

use crate::types::Role;

/// Topics: `("project_funded", project)`. `funds` is what this source holds
/// for the project after the deposit.
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectFunded {
    #[topic]
    pub project: BytesN<32>,
    pub source: BytesN<32>,
    pub from: Address,
    pub amount: i128,
    pub funds: i128,
}

/// Topics: `("budget_set", project)`.
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BudgetSet {
    #[topic]
    pub project: BytesN<32>,
    pub line: BytesN<32>,
    pub source: BytesN<32>,
    pub chart: Symbol,
    pub account: Symbol,
    pub currency: Symbol,
    pub budget: i128,
    pub proposal_id: u64,
    pub proposer: Address,
    pub approver: Address,
}

/// Topics: `("expense_recorded", project)`. An expense paid outside the
/// ledger, recorded against its line with two signers and its proof of spend.
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExpenseRecorded {
    #[topic]
    pub project: BytesN<32>,
    pub line: BytesN<32>,
    pub source: BytesN<32>,
    pub proposal_id: u64,
    pub amount: i128,
    pub currency: Symbol,
    pub chart: Symbol,
    pub account: Symbol,
    pub receipt_hash: BytesN<32>,
    pub payment_ref: BytesN<32>,
    pub payee_ref: BytesN<32>,
    pub proposer: Address,
    pub approver: Address,
    pub line_paid: i128,
    pub line_budget: i128,
}

/// Topics: `("payment_executed", project)`. A payment that left the treasury
/// on the ledger, with its proof of spend.
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PaymentExecuted {
    #[topic]
    pub project: BytesN<32>,
    pub line: BytesN<32>,
    pub source: BytesN<32>,
    pub proposal_id: u64,
    pub to: Address,
    pub amount: i128,
    pub currency: Symbol,
    pub chart: Symbol,
    pub account: Symbol,
    pub receipt_hash: BytesN<32>,
    pub reference: BytesN<32>,
    pub proposer: Address,
    pub approver: Address,
    pub line_paid: i128,
    pub line_budget: i128,
    pub source_funds: i128,
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

/// Topics: `("proposal_executed", id)`. Emitted for every executed proposal,
/// so governance can be audited from events alone.
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProposalExecuted {
    #[topic]
    pub id: u64,
    pub kind: Symbol,
    pub proposer: Address,
    pub approver: Address,
}

/// Topics: `("proposal_rejected", id)`.
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProposalRejected {
    #[topic]
    pub id: u64,
    pub rejected_by: Address,
}

/// Topics: `("member_added", member)`. Also emitted when a role changes.
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

/// Topics: `("contract_upgraded", proposal_id)`. Emitted before the code is
/// replaced, so a reader knows the rules may have changed from this point.
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContractUpgraded {
    #[topic]
    pub proposal_id: u64,
    pub wasm_hash: BytesN<32>,
    pub proposer: Address,
    pub approver: Address,
}
