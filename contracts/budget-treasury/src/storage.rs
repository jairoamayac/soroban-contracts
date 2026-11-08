use soroban_sdk::{contracttype, Address, BytesN, Env, Map, Symbol};
use truway_common::ttl;

use crate::types::{BudgetLine, FundKey, LineKey, Proposal, Role};

/// Storage keys. Variant names are part of the on-ledger encoding: never
/// rename or remove one used by a deployed instance without a migration.
#[contracttype]
pub enum DataKey {
    Token,
    TokenSymbol,
    Members,
    NextId,
    Proposal(u64),
    Line(LineKey),
    Funds(FundKey),
    Receipt(BytesN<32>),
}

pub(crate) fn token(env: &Env) -> Address {
    env.storage().instance().get(&DataKey::Token).unwrap()
}

pub(crate) fn token_symbol(env: &Env) -> Symbol {
    env.storage().instance().get(&DataKey::TokenSymbol).unwrap()
}

pub(crate) fn set_token(env: &Env, token: &Address, symbol: &Symbol) {
    env.storage().instance().set(&DataKey::Token, token);
    env.storage().instance().set(&DataKey::TokenSymbol, symbol);
}

pub(crate) fn members(env: &Env) -> Map<Address, Role> {
    env.storage()
        .instance()
        .get(&DataKey::Members)
        .unwrap_or_else(|| Map::new(env))
}

pub(crate) fn set_members(env: &Env, members: &Map<Address, Role>) {
    env.storage().instance().set(&DataKey::Members, members);
}

pub(crate) fn next_id(env: &Env) -> u64 {
    let id: u64 = env.storage().instance().get(&DataKey::NextId).unwrap_or(1);
    env.storage().instance().set(&DataKey::NextId, &(id + 1));
    id
}

pub(crate) fn proposal(env: &Env, id: u64) -> Option<Proposal> {
    env.storage().persistent().get(&DataKey::Proposal(id))
}

pub(crate) fn set_proposal(env: &Env, proposal: &Proposal) {
    let key = DataKey::Proposal(proposal.id);
    env.storage().persistent().set(&key, proposal);
    ttl::extend_persistent(env, &key);
}

pub(crate) fn line(env: &Env, key: &LineKey) -> Option<BudgetLine> {
    env.storage().persistent().get(&DataKey::Line(key.clone()))
}

pub(crate) fn set_line(env: &Env, key: &LineKey, line: &BudgetLine) {
    let k = DataKey::Line(key.clone());
    env.storage().persistent().set(&k, line);
    ttl::extend_persistent(env, &k);
}

pub(crate) fn funds(env: &Env, key: &FundKey) -> i128 {
    env.storage()
        .persistent()
        .get(&DataKey::Funds(key.clone()))
        .unwrap_or(0)
}

pub(crate) fn set_funds(env: &Env, key: &FundKey, amount: i128) {
    let k = DataKey::Funds(key.clone());
    env.storage().persistent().set(&k, &amount);
    ttl::extend_persistent(env, &k);
}

/// Proposal id of the executed expense or payment a supporting document backs.
pub(crate) fn receipt_use(env: &Env, hash: &BytesN<32>) -> Option<u64> {
    env.storage()
        .persistent()
        .get(&DataKey::Receipt(hash.clone()))
}

pub(crate) fn set_receipt_use(env: &Env, hash: &BytesN<32>, proposal_id: u64) {
    let k = DataKey::Receipt(hash.clone());
    env.storage().persistent().set(&k, &proposal_id);
    ttl::extend_persistent(env, &k);
}
