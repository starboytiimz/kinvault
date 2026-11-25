use soroban_sdk::{contracttype, Address, String};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Config,
    Member(Address),
    MemberList,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VaultConfig {
    pub admin: Address,
    pub token: Address,
    pub frozen: bool,
    pub total_deposited: i128,
    pub total_withdrawn: i128,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MemberAllowance {
    pub member: Address,
    pub label: String,
    pub cap_per_period: i128,
    pub period_seconds: u64,
    pub period_start: u64,
    pub withdrawn_in_period: i128,
    pub bonus_balance: i128,
    pub paused: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VaultSummary {
    pub admin: Address,
    pub token: Address,
    pub frozen: bool,
    pub current_balance: i128,
    pub total_deposited: i128,
    pub total_withdrawn: i128,
    pub member_count: u32,
}
