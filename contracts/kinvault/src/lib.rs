#![no_std]

pub mod errors;
pub mod types;

#[cfg(test)]
mod test;

use errors::VaultError;
use soroban_sdk::{contract, contractimpl, symbol_short, token, Address, Env, String, Vec};
use types::{DataKey, MemberAllowance, VaultConfig, VaultSummary};

const MIN_PERIOD_SECONDS: u64 = 3600; // 1 hour minimum rolling window
const TTL_THRESHOLD: u32 = 50_000;
const TTL_EXTEND_TO: u32 = 200_000;

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
        env.storage()
            .persistent()
            .extend_ttl(&DataKey::Member(member.clone()), TTL_THRESHOLD, TTL_EXTEND_TO);

        let mut list: Vec<Address> = env
            .storage()
            .instance()
            .get(&DataKey::MemberList)
            .unwrap_or_else(|| Vec::new(&env));
        if !list.contains(&member) {
            list.push_back(member.clone());
            env.storage().instance().set(&DataKey::MemberList, &list);
        }
        env.storage()
            .instance()
            .extend_ttl(TTL_THRESHOLD, TTL_EXTEND_TO);

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

    /// Freeze or unfreeze all vault withdrawals in an emergency.
    pub fn set_frozen(env: Env, frozen: bool) -> Result<(), VaultError> {
        let mut cfg: VaultConfig = env
            .storage()
            .instance()
            .get(&DataKey::Config)
            .ok_or(VaultError::NotInitialized)?;
        cfg.admin.require_auth();
        cfg.frozen = frozen;
        env.storage().instance().set(&DataKey::Config, &cfg);
        env.events().publish((symbol_short!("freeze"),), frozen);
        Ok(())
    }

    /// Pause or resume withdrawals for an individual member.
    pub fn set_member_paused(env: Env, member: Address, paused: bool) -> Result<(), VaultError> {
        let cfg: VaultConfig = env
            .storage()
            .instance()
            .get(&DataKey::Config)
            .ok_or(VaultError::NotInitialized)?;
        cfg.admin.require_auth();

        let key = DataKey::Member(member.clone());
        let mut allowance: MemberAllowance = env
            .storage()
            .persistent()
            .get(&key)
            .ok_or(VaultError::MemberNotFound)?;
        allowance.paused = paused;
        env.storage().persistent().set(&key, &allowance);
        env.events().publish((symbol_short!("pause"), member), paused);
        Ok(())
    }

    /// Credit a one-off chore or emergency bonus to a member's withdrawable balance.
    pub fn release_bonus(env: Env, member: Address, bonus: i128) -> Result<i128, VaultError> {
        if bonus <= 0 {
            return Err(VaultError::InvalidAmount);
        }
        let cfg: VaultConfig = env
            .storage()
            .instance()
            .get(&DataKey::Config)
            .ok_or(VaultError::NotInitialized)?;
        cfg.admin.require_auth();

        let key = DataKey::Member(member.clone());
        let mut allowance: MemberAllowance = env
            .storage()
            .persistent()
            .get(&key)
            .ok_or(VaultError::MemberNotFound)?;
        allowance.bonus_balance += bonus;
        env.storage().persistent().set(&key, &allowance);
        env.events().publish((symbol_short!("bonus"), member), bonus);
        Ok(allowance.bonus_balance)
    }

    /// Calculate how much a member can currently withdraw (rolling period remainder + bonus).
    pub fn remaining_allowance(env: Env, member: Address) -> Result<i128, VaultError> {
        let allowance: MemberAllowance = env
            .storage()
            .persistent()
            .get(&DataKey::Member(member))
            .ok_or(VaultError::MemberNotFound)?;
        let now = env.ledger().timestamp();
        let used = if now >= allowance.period_start.saturating_add(allowance.period_seconds) {
            0
        } else {
            allowance.withdrawn_in_period
        };
        let period_rem = (allowance.cap_per_period - used).max(0);
        Ok(period_rem + allowance.bonus_balance)
    }

    /// Withdraw up to the member's remaining rolling-period cap plus any approved bonus.
    pub fn withdraw(env: Env, member: Address, amount: i128) -> Result<i128, VaultError> {
        if amount <= 0 {
            return Err(VaultError::InvalidAmount);
        }
        member.require_auth();

        let mut cfg: VaultConfig = env
            .storage()
            .instance()
            .get(&DataKey::Config)
            .ok_or(VaultError::NotInitialized)?;
        if cfg.frozen {
            return Err(VaultError::VaultFrozen);
        }

        let key = DataKey::Member(member.clone());
        let mut allowance: MemberAllowance = env
            .storage()
            .persistent()
            .get(&key)
            .ok_or(VaultError::MemberNotFound)?;
        if allowance.paused {
            return Err(VaultError::MemberPaused);
        }

        let now = env.ledger().timestamp();
        if now >= allowance.period_start.saturating_add(allowance.period_seconds) {
            allowance.period_start = now;
            allowance.withdrawn_in_period = 0;
        }

        let period_rem = (allowance.cap_per_period - allowance.withdrawn_in_period).max(0);
        let total_avail = period_rem + allowance.bonus_balance;
        if amount > total_avail {
            return Err(VaultError::AllowanceExceeded);
        }

        let client = token::Client::new(&env, &cfg.token);
        let contract_bal = client.balance(&env.current_contract_address());
        if amount > contract_bal {
            return Err(VaultError::InsufficientVaultBalance);
        }

        // Consume rolling period allowance first, then bonus balance
        if amount <= period_rem {
            allowance.withdrawn_in_period += amount;
        } else {
            let from_bonus = amount - period_rem;
            allowance.withdrawn_in_period += period_rem;
            allowance.bonus_balance -= from_bonus;
        }

        client.transfer(&env.current_contract_address(), &member, &amount);

        cfg.total_withdrawn += amount;
        env.storage().instance().set(&DataKey::Config, &cfg);
        env.storage().persistent().set(&key, &allowance);
        env.events().publish((symbol_short!("withdraw"), member), amount);

        Ok(total_avail - amount)
    }

    /// Return all registered member addresses in this vault.
    pub fn list_members(env: Env) -> Vec<Address> {
        env.storage()
            .instance()
            .get(&DataKey::MemberList)
            .unwrap_or_else(|| Vec::new(&env))
    }

    /// Allow the vault admin to reclaim unspent funds back to their own address.
    pub fn admin_withdraw(env: Env, amount: i128) -> Result<i128, VaultError> {
        if amount <= 0 {
            return Err(VaultError::InvalidAmount);
        }
        let cfg: VaultConfig = env
            .storage()
            .instance()
            .get(&DataKey::Config)
            .ok_or(VaultError::NotInitialized)?;
        cfg.admin.require_auth();

        let client = token::Client::new(&env, &cfg.token);
        let bal = client.balance(&env.current_contract_address());
        if amount > bal {
            return Err(VaultError::InsufficientVaultBalance);
        }
        client.transfer(&env.current_contract_address(), &cfg.admin, &amount);
        env.storage()
            .instance()
            .extend_ttl(TTL_THRESHOLD, TTL_EXTEND_TO);
        env.events()
            .publish((symbol_short!("a_wdraw"), cfg.admin), amount);
        Ok(bal - amount)
    }

    /// Transfer vault administration to a new household manager address.
    pub fn transfer_admin(env: Env, new_admin: Address) -> Result<(), VaultError> {
        let mut cfg: VaultConfig = env
            .storage()
            .instance()
            .get(&DataKey::Config)
            .ok_or(VaultError::NotInitialized)?;
        cfg.admin.require_auth();
        cfg.admin = new_admin.clone();
        env.storage().instance().set(&DataKey::Config, &cfg);
        env.storage()
            .instance()
            .extend_ttl(TTL_THRESHOLD, TTL_EXTEND_TO);
        env.events().publish((symbol_short!("t_admin"),), new_admin);
        Ok(())
    }

    /// Return an aggregated vault summary combining config, live balance, and member count.
    pub fn get_summary(env: Env) -> Result<VaultSummary, VaultError> {
        let cfg: VaultConfig = env
            .storage()
            .instance()
            .get(&DataKey::Config)
            .ok_or(VaultError::NotInitialized)?;
        let client = token::Client::new(&env, &cfg.token);
        let current_balance = client.balance(&env.current_contract_address());
        let list: Vec<Address> = env
            .storage()
            .instance()
            .get(&DataKey::MemberList)
            .unwrap_or_else(|| Vec::new(&env));
        Ok(VaultSummary {
            admin: cfg.admin,
            token: cfg.token,
            frozen: cfg.frozen,
            current_balance,
            total_deposited: cfg.total_deposited,
            total_withdrawn: cfg.total_withdrawn,
            member_count: list.len(),
        })
    }
}

