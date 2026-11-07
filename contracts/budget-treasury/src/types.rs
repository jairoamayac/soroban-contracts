use soroban_sdk::{contracterror, contracttype, Address, BytesN, Symbol};

/// What a member may do.
///
/// * `Admin`: propose any action and approve or reject proposals.
/// * `Approver`: approve or reject proposals.
/// * `Operator`: propose payments only.
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Role {
    Admin,
    Approver,
    Operator,
}

/// A budget line of a project. `budget` and `paid` are in the treasury asset's
/// smallest unit (7 decimals for Stellar assets).
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BudgetLine {
    /// Chart of accounts the line books to, e.g. `CO_PUC` or `IFRS`.
    pub chart: Symbol,
    /// Default account code in that chart, e.g. `5135`.
    pub account: Symbol,
    pub budget: i128,
    pub paid: i128,
}

/// Identifies a budget line: SHA-256 of the project and line identifiers.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LineKey {
    pub project: BytesN<32>,
    pub line: BytesN<32>,
}

/// Identifies the funds one funder earmarked for one project.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FundKey {
    pub project: BytesN<32>,
    /// SHA-256 of the funder (funding source) identifier.
    pub source: BytesN<32>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BudgetChange {
    pub project: BytesN<32>,
    pub line: BytesN<32>,
    pub chart: Symbol,
    pub account: Symbol,
    pub amount: i128,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Payment {
    pub project: BytesN<32>,
    pub line: BytesN<32>,
    /// Funding source the payment is charged to.
    pub source: BytesN<32>,
    pub to: Address,
    pub amount: i128,
    /// Must match the chart of the budget line.
    pub chart: Symbol,
    /// Account the payment books to; may be an auxiliary of the line's account.
    pub account: Symbol,
    /// SHA-256 of the supporting document (invoice, receipt). All zeros means
    /// "support pending": allowed, but never counted as used.
    pub receipt_hash: BytesN<32>,
    /// Off-chain reference (e.g. SHA-256 of the payment UUID).
    pub reference: BytesN<32>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MemberChange {
    pub member: Address,
    pub role: Role,
}

/// Everything that changes state goes through a proposal.
// Contract types cannot hold a Box, so the size gap between variants is accepted.
#[allow(clippy::large_enum_variant)]
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Action {
    SetBudget(BudgetChange),
    Pay(Payment),
    AddMember(MemberChange),
    RemoveMember(Address),
    Upgrade(BytesN<32>),
}

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProposalStatus {
    Pending,
    Executed,
    Rejected,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Proposal {
    pub id: u64,
    pub proposer: Address,
    pub action: Action,
    pub status: ProposalStatus,
    pub decided_by: Option<Address>,
    pub created_at: u64,
    pub decided_at: u64,
}

/// Error codes are part of the public interface: never renumber them.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    Unauthorized = 1,
    SelfApproval = 2,
    NotFound = 3,
    InvalidAmount = 4,
    BudgetExceeded = 5,
    InsufficientFunds = 6,
    InvalidState = 7,
    LineNotFound = 8,
    BudgetBelowPaid = 9,
    ChartMismatch = 10,
    /// Removing the member would leave no admin, or fewer than two members
    /// able to approve, which would lock the treasury.
    QuorumLost = 11,
    NoAdmins = 12,
    /// The supporting document already backs another executed payment.
    ReceiptAlreadyUsed = 13,
}
