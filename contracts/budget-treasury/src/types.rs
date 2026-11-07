use soroban_sdk::{contracterror, contracttype, Address, BytesN, Symbol};

/// What a member may do.
///
/// * `Admin`: propose any action; approve or reject any proposal.
/// * `Approver`: approve or reject budgets, expenses and payments.
/// * `Operator`: propose expenses and payments only.
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Role {
    Admin,
    Approver,
    Operator,
}

/// A budget line of a project, funded by one source. Amounts are fixed point
/// with 7 decimals in `currency` (45,000 COP is `450_000_000_000`).
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BudgetLine {
    /// Chart of accounts the line books to, e.g. `CO_PUC` or `IFRS`.
    pub chart: Symbol,
    /// Default account code in that chart, e.g. `5135`.
    pub account: Symbol,
    /// Currency of the budget, e.g. `COP` or `USDC`.
    pub currency: Symbol,
    pub budget: i128,
    pub paid: i128,
}

/// Identifies a budget line: project, line and funding source, each the
/// SHA-256 of an opaque identifier.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LineKey {
    pub project: BytesN<32>,
    pub line: BytesN<32>,
    pub source: BytesN<32>,
}

/// Identifies the funds one source holds in the treasury for one project.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FundKey {
    pub project: BytesN<32>,
    pub source: BytesN<32>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BudgetChange {
    pub project: BytesN<32>,
    pub line: BytesN<32>,
    pub source: BytesN<32>,
    pub chart: Symbol,
    pub account: Symbol,
    pub currency: Symbol,
    pub amount: i128,
}

/// An expense paid outside the ledger (bank transfer, cash) and recorded
/// against its budget line. No funds move.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Expense {
    pub project: BytesN<32>,
    pub line: BytesN<32>,
    pub source: BytesN<32>,
    pub amount: i128,
    /// Must match the chart of the budget line.
    pub chart: Symbol,
    /// Account the expense books to; may be an auxiliary of the line's account.
    pub account: Symbol,
    /// SHA-256 of the supporting document (invoice, receipt). Required.
    pub receipt_hash: BytesN<32>,
    /// SHA-256 of the external payment evidence (transfer receipt, cash voucher id).
    pub payment_ref: BytesN<32>,
    /// SHA-256 of the payee identifier (e.g. tax id). Never the identifier itself.
    pub payee_ref: BytesN<32>,
}

/// A payment that leaves the treasury on the ledger.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Payment {
    pub project: BytesN<32>,
    pub line: BytesN<32>,
    pub source: BytesN<32>,
    pub to: Address,
    pub amount: i128,
    /// Must match the chart of the budget line.
    pub chart: Symbol,
    pub account: Symbol,
    /// SHA-256 of the supporting document. Required.
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
    RecordExpense(Expense),
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
    /// The change would leave no admin, or fewer than two members able to
    /// approve, which would lock the treasury.
    QuorumLost = 11,
    NoAdmins = 12,
    /// The supporting document already backs another executed expense or payment.
    ReceiptAlreadyUsed = 13,
    /// Every expense and payment needs a supporting document.
    ReceiptRequired = 14,
    /// The operation's currency does not match its budget line (or the treasury asset).
    CurrencyMismatch = 15,
}
