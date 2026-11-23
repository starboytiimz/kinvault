#![no_std]

pub mod errors;
pub mod types;

use errors::VaultError;
use soroban_sdk::{contract, contractimpl, symbol_short, token, Address, Env, String};
use types::{DataKey, MemberAllowance, VaultConfig};

const MIN_PERIOD_SECONDS: u64 = 3600; // 1 hour minimum rolling window

#[contract]
pub struct KinVaultContract;

#[contractimpl]
impl KinVaultContract {
    /// Initialize a household allowance vault controlled by `admin` using SEP-41 `token` (e.g. USDC).
    pub fn init(env: Env, admin: Address, token: Address) -> Result<(), VaultError> {
        if env.storage().instance().has(&DataKey::Config) {
            return Err(VaultError::AlreadyInitialized);
        }
        admin.require_auth();

        let cfg = VaultConfig {
            admin: admin.clone(),
            token: token.clone(),
            frozen: false,
            total_deposited: 0,
            total_withdrawn: 0,
        };
        env.storage().instance().set(&DataKey::Config, &cfg);
        env.events().publish((symbol_short!("init"), admin), token);
        Ok(())
    }

    /// Deposit funds into the household vault.
    pub fn fund(env: Env, from: Address, amount: i128) -> Result<i128, VaultError> {
        if amount <= 0 {
            return Err(VaultError::InvalidAmount);
        }
        from.require_auth();

        let mut cfg: VaultConfig = env
            .storage()
            .instance()
            .get(&DataKey::Config)
            .ok_or(VaultError::NotInitialized)?;

        let client = token::Client::new(&env, &cfg.token);
        client.transfer(&from, &env.current_contract_address(), &amount);

        cfg.total_deposited += amount;
        env.storage().instance().set(&DataKey::Config, &cfg);
        env.events().publish((symbol_short!("fund"), from), amount);

        Ok(client.balance(&env.current_contract_address()))
    }

    /// Register or update a family member's rolling allowance cap and period.
    pub fn set_member(
        env: Env,
        member: Address,
        label: String,
        cap_per_period: i128,
        period_seconds: u64,
    ) -> Result<(), VaultError> {
        if cap_per_period <= 0 {
            return Err(VaultError::InvalidAmount);
        }
        if period_seconds < MIN_PERIOD_SECONDS {
            return Err(VaultError::InvalidPeriod);
        }

        let cfg: VaultConfig = env
            .storage()
            .instance()
            .get(&DataKey::Config)
            .ok_or(VaultError::NotInitialized)?;
        cfg.admin.require_auth();

        let now = env.ledger().timestamp();
        let existing: Option<MemberAllowance> = env
            .storage()
            .persistent()
            .get(&DataKey::Member(member.clone()));

        let allowance = match existing {
            Some(mut m) => {
                m.label = label;
                m.cap_per_period = cap_per_period;
                m.period_seconds = period_seconds;
                m
            }
            None => MemberAllowance {
                member: member.clone(),
                label,
                cap_per_period,
                period_seconds,
                period_start: now,
                withdrawn_in_period: 0,
                bonus_balance: 0,
                paused: false,
            },
        };

        env.storage()
            .persistent()
            .set(&DataKey::Member(member.clone()), &allowance);
        env.events()
            .publish((symbol_short!("member"), member), cap_per_period);
        Ok(())
    }

    /// Remove a family member from the vault.
    pub fn remove_member(env: Env, member: Address) -> Result<(), VaultError> {
        let cfg: VaultConfig = env
            .storage()
            .instance()
            .get(&DataKey::Config)
            .ok_or(VaultError::NotInitialized)?;
        cfg.admin.require_auth();

        let key = DataKey::Member(member.clone());
        if !env.storage().persistent().has(&key) {
            return Err(VaultError::MemberNotFound);
        }
        env.storage().persistent().remove(&key);
        env.events().publish((symbol_short!("rm_mbr"), member), ());
        Ok(())
    }

    /// Return current vault configuration and totals.
    pub fn get_vault(env: Env) -> Result<VaultConfig, VaultError> {
        env.storage()
            .instance()
            .get(&DataKey::Config)
            .ok_or(VaultError::NotInitialized)
    }

    /// Return a member's allowance record.
    pub fn get_member(env: Env, member: Address) -> Result<MemberAllowance, VaultError> {
        env.storage()
            .persistent()
            .get(&DataKey::Member(member))
            .ok_or(VaultError::MemberNotFound)
    }

    /// Return current token balance held by the vault contract.
    pub fn vault_balance(env: Env) -> Result<i128, VaultError> {
        let cfg: VaultConfig = env
            .storage()
            .instance()
            .get(&DataKey::Config)
            .ok_or(VaultError::NotInitialized)?;
        let client = token::Client::new(&env, &cfg.token);
        Ok(client.balance(&env.current_contract_address()))
    }
}
