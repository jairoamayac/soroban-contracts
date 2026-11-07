//! # budget-treasury
//!
//! Reference implementation of Proof of Spend (`proof-of-spend/1`), an open
//! standard for budget execution on Stellar (`docs/proof-of-spend.md`).
//!
//! Budget lines are set per project, line and funding source, each with an
//! accounting code (`chart` + `account`) and a currency. Expenses paid outside
//! the ledger are recorded against them, and money held in the contract is
//! paid against them:
//!
//! * every change (budget lines, expenses, payments, members, upgrades) is a
//!   proposal that a second, distinct member must approve;
//! * an expense or payment never exceeds what is left in its line for its
//!   funding source, and a payment never exceeds the funds that source holds;
//! * every expense and payment carries the SHA-256 hash of its supporting
//!   document, used at most once, so the funder report and the general ledger
//!   can be rebuilt from events alone.
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

pub use contract::{BudgetTreasury, BudgetTreasuryClient, MAX_MEMBERS};
pub use events::{
    BudgetSet, ContractUpgraded, ExpenseRecorded, MemberAdded, MemberRemoved, PaymentExecuted,
    ProjectFunded, ProposalCreated, ProposalExecuted, ProposalRejected,
};
pub use storage::DataKey;
pub use types::{
    Action, BudgetChange, BudgetLine, Error, Expense, FundKey, LineKey, MemberChange, Payment,
    Proposal, ProposalStatus, Role,
};
