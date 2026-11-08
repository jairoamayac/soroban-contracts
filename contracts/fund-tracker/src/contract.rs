use soroban_sdk::{contract, contractimpl, Address, BytesN, ContractExecutable, Env, Symbol};
use truway_common::ttl;

use crate::events::FundAllocated;
use crate::storage;
use crate::types::FundAllocation;

#[contract]
pub struct FundTracker;

#[contractimpl]
impl FundTracker {
    /// Sets the admin. Can only be called once.
    pub fn initialize(env: Env, admin: Address) {
        if storage::has_admin(&env) {
            panic!("already_initialized");
        }
        admin.require_auth();
        storage::set_admin(&env, &admin);
        ttl::extend_instance(&env);
    }

    /// Records (or re-records) the allocation for `project_id`.
    /// Only the admin can write: allocations are attested by the Truway server,
    /// and `organization` stores the attesting signer (see ADR 0002).
    pub fn allocate(env: Env, caller: Address, project_id: Symbol, amount_xlm: i128) {
        storage::require_caller_is_admin(&env, &caller);

        if amount_xlm <= 0 {
            panic!("invalid_amount");
        }

        let allocation = FundAllocation {
            project_id: project_id.clone(),
            organization: caller,
            amount_xlm,
            allocated_at: env.ledger().timestamp(),
        };

        storage::set_allocation(&env, project_id.clone(), &allocation);
        ttl::extend_instance(&env);

        FundAllocated {
            project_id,
            organization: allocation.organization,
            amount_xlm: allocation.amount_xlm,
            allocated_at: allocation.allocated_at,
        }
        .publish(&env);
    }

    pub fn get_allocation(env: Env, project_id: Symbol) -> Option<FundAllocation> {
        storage::get_allocation(&env, project_id)
    }

    /// Replaces the contract code in place, keeping its ID and storage. Admin only.
    pub fn upgrade(env: Env, new_wasm_hash: BytesN<32>) {
        storage::admin(&env).require_auth();
        env.deployer()
            .update_current_contract(ContractExecutable::Wasm(new_wasm_hash));
    }
}
