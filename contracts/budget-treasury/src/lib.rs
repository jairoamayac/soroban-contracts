//! # budget-treasury
//!
//! Budget execution as a contract. An organisation's funds sit in this
//! contract and leave it only as payments against an approved budget line:
//!
//! * every change (budget lines, members, payments, upgrades) is a proposal
//!   that a second, distinct member must approve;
//! * a payment can never exceed what is left in its budget line, nor the
//!   funds assigned to its project;
//! * every payment carries the accounting code it books to (`chart` +
//!   `account`) and the SHA-256 hash of its supporting document, so the
//!   funder report and the general ledger can be rebuilt from events alone.
//!
//! Charts of accounts are plain symbols (`CO_PUC`, `IFRS`, `PE_PCGE`, ...):
//! nothing in the contract is specific to one country. Truway deploys the
//! contract but is not a member, so it cannot move funds or upgrade it.
#![no_std]

mod contract;
mod events;
mod storage;
mod types;

#[cfg(test)]
mod test;

pub use contract::{BudgetTreasury, BudgetTreasuryClient};
pub use events::{
    BudgetSet, MemberAdded, MemberRemoved, PaymentExecuted, ProjectFunded, ProposalCreated,
    ProposalRejected,
};
pub use storage::DataKey;
pub use types::{
    Action, BudgetChange, BudgetLine, Error, FundKey, LineKey, MemberChange, Payment, Proposal,
    ProposalStatus, Role,
};
