use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum VaultError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    InvalidAmount = 3,
    InvalidPeriod = 4,
    MemberNotFound = 5,
    VaultFrozen = 6,
    MemberPaused = 7,
    AllowanceExceeded = 8,
    InsufficientVaultBalance = 9,
}
