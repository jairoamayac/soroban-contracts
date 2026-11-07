use soroban_sdk::{contract, contractimpl, Address, BytesN, ContractExecutable, Env, Symbol, Vec};
use truway_common::ttl;

use crate::events::{BadgeMinted, BadgeRevoked};
use crate::storage;
use crate::types::{is_valid_badge_type, Badge, BadgeStatus};

#[contract]
pub struct SbtBadge;

#[contractimpl]
impl SbtBadge {
    /// Initializes the contract. Can only be called once.
    pub fn initialize(env: Env, admin: Address) {
        if storage::has_admin(&env) {
            panic!("already_initialized");
        }
        admin.require_auth();
        storage::set_admin(&env, &admin);
        storage::set_next_token_id(&env, 1);
        ttl::extend_instance(&env);
    }

    /// Mints a reputation SBT to `organization`.
    /// Only the admin can call this function. An organization can hold at most
    /// one active badge of each type; revoke it before minting it again.
    pub fn mint_badge(env: Env, organization: Address, badge_type: Symbol) -> u64 {
        storage::require_admin(&env);
        if !is_valid_badge_type(&env, &badge_type) {
            panic!("invalid_badge_type");
        }

        let already_active = Self::get_active_badges(env.clone(), organization.clone())
            .iter()
            .any(|b| b.badge_type == badge_type);
        if already_active {
            panic!("badge_already_active");
        }

        let token_id = storage::next_token_id(&env);
        storage::set_next_token_id(&env, token_id + 1);

        let badge = Badge {
            token_id,
            badge_type: badge_type.clone(),
            organization: organization.clone(),
            status: BadgeStatus::Active,
            issued_at: env.ledger().timestamp(),
            revoked_at: 0,
        };
        storage::set_badge(&env, &badge);

        let mut ids = storage::org_badge_ids(&env, organization.clone());
        ids.push_back(token_id);
        storage::set_org_badge_ids(&env, organization.clone(), &ids);
        ttl::extend_instance(&env);

        BadgeMinted {
            badge_type,
            token_id,
            organization,
        }
        .publish(&env);

        token_id
    }

    /// Revokes a badge. The original record remains on ledger with status=Revoked.
    /// Only the admin can revoke.
    pub fn revoke_badge(env: Env, token_id: u64) {
        storage::require_admin(&env);

        let mut badge =
            storage::get_badge(&env, token_id).unwrap_or_else(|| panic!("token_not_found"));

        if badge.status == BadgeStatus::Revoked {
            panic!("already_revoked");
        }

        badge.status = BadgeStatus::Revoked;
        badge.revoked_at = env.ledger().timestamp();
        storage::set_badge(&env, &badge);
        ttl::extend_instance(&env);

        BadgeRevoked {
            token_id,
            organization: badge.organization,
        }
        .publish(&env);
    }

    /// Returns a badge by token_id. None if it does not exist.
    pub fn get_badge(env: Env, token_id: u64) -> Option<Badge> {
        storage::get_badge(&env, token_id)
    }

    /// All badges (active + revoked) for an organization.
    pub fn get_badges(env: Env, organization: Address) -> Vec<Badge> {
        let ids = storage::org_badge_ids(&env, organization);

        let mut badges = Vec::new(&env);
        for id in ids.iter() {
            if let Some(b) = storage::get_badge(&env, id) {
                badges.push_back(b);
            }
        }
        badges
    }

    /// Only active badges for an organization.
    pub fn get_active_badges(env: Env, organization: Address) -> Vec<Badge> {
        let all = Self::get_badges(env.clone(), organization);
        let mut active = Vec::new(&env);
        for b in all.iter() {
            if b.status == BadgeStatus::Active {
                active.push_back(b);
            }
        }
        active
    }

    /// Replaces the contract code in place, keeping its ID and storage. Admin only.
    pub fn upgrade(env: Env, new_wasm_hash: BytesN<32>) {
        storage::require_admin(&env);
        env.deployer()
            .update_current_contract(ContractExecutable::Wasm(new_wasm_hash));
    }
}
