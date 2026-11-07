use soroban_sdk::{contracttype, Address, BytesN, Env, Map};
use truway_common::ttl;

use crate::types::{BudgetLine, LineKey, Proposal, Role};

/// Storage keys. Variant names are part of the on-ledger encoding: never
/// rename or remove one used by a deployed instance without a migration.
#[contracttype]
pub enum DataKey {
    Token,
    Members,
    NextId,
    Proposal(u64),
    Line(LineKey),
    Funds(BytesN<32>),
}

pub(crate) fn token(env: &Env) -> Address {
    env.storage().instance().get(&DataKey::Token).unwrap()
}

pub(crate) fn set_token(env: &Env, token: &Address) {
    env.storage().instance().set(&DataKey::Token, token);
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

pub(crate) fn funds(env: &Env, project: &BytesN<32>) -> i128 {
    env.storage()
        .persistent()
        .get(&DataKey::Funds(project.clone()))
        .unwrap_or(0)
}

pub(crate) fn set_funds(env: &Env, project: &BytesN<32>, amount: i128) {
    let k = DataKey::Funds(project.clone());
    env.storage().persistent().set(&k, &amount);
    ttl::extend_persistent(env, &k);
}
