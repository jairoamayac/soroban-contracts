use soroban_sdk::{
    contract, contractimpl, Address, Bytes, BytesN, ContractExecutable, Env, Symbol,
};
use truway_common::ttl;

use crate::events::ExpenseAnchored;
use crate::storage;
use crate::types::{AnchoredExpense, RECEIPT_HASH_LEN};

#[contract]
pub struct ExpenseAnchor;

#[contractimpl]
impl ExpenseAnchor {
    /// Sets the admin. Can only be called once.
    pub fn initialize(env: Env, admin: Address) {
        if storage::has_admin(&env) {
            panic!("already_initialized");
        }
        admin.require_auth();
        storage::set_admin(&env, &admin);
        ttl::extend_instance(&env);
    }

    /// Anchors an approved expense. Records are immutable: anchoring an existing
    /// `expense_id` again with the same data is a no-op (so retries are safe),
    /// and with different data it fails.
    /// Only the admin can write; `submitted_by` stores the attesting signer
    /// (see ADR 0002).
    pub fn anchor(
        env: Env,
        caller: Address,
        expense_id: Symbol,
        project_id: Symbol,
        amount_xlm: i128,
        receipt_hash: Bytes,
    ) {
        storage::require_caller_is_admin(&env, &caller);

        if amount_xlm < 0 {
            panic!("invalid_amount");
        }
        if receipt_hash.len() != RECEIPT_HASH_LEN {
            panic!("invalid_receipt_hash");
        }

        if let Some(existing) = storage::get_expense(&env, expense_id.clone()) {
            let same = existing.project_id == project_id
                && existing.amount_xlm == amount_xlm
                && existing.receipt_hash == receipt_hash;
            if !same {
                panic!("expense_already_exists");
            }
            storage::extend_expense(&env, expense_id);
            return;
        }

        let expense = AnchoredExpense {
            expense_id: expense_id.clone(),
            project_id: project_id.clone(),
            submitted_by: caller,
            amount_xlm,
            receipt_hash,
            anchored_at: env.ledger().timestamp(),
        };

        storage::set_expense(&env, expense_id.clone(), &expense);
        ttl::extend_instance(&env);

        ExpenseAnchored {
            project_id,
            expense_id,
            submitted_by: expense.submitted_by,
            amount_xlm: expense.amount_xlm,
            receipt_hash: expense.receipt_hash,
            anchored_at: expense.anchored_at,
        }
        .publish(&env);
    }

    pub fn get_expense(env: Env, expense_id: Symbol) -> Option<AnchoredExpense> {
        storage::get_expense(&env, expense_id)
    }

    /// Replaces the contract code in place, keeping its ID and storage. Admin only.
    pub fn upgrade(env: Env, new_wasm_hash: BytesN<32>) {
        storage::admin(&env).require_auth();
        env.deployer()
            .update_current_contract(ContractExecutable::Wasm(new_wasm_hash));
    }
}
