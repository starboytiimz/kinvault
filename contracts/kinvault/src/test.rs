#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token::{Client as TokenClient, StellarAssetClient},
    Address, Env, String,
};

fn setup_vault(
    env: &Env,
) -> (
    KinVaultContractClient<'_>,
    Address,
    Address,
    TokenClient<'_>,
    StellarAssetClient<'_>,
) {
    let admin = Address::generate(env);
    let issuer = Address::generate(env);
    let sac = env.register_stellar_asset_contract_v2(issuer);
    let token_id = sac.address();
    let token_client = TokenClient::new(env, &token_id);
    let token_admin = StellarAssetClient::new(env, &token_id);

    let contract_id = env.register(KinVaultContract, ());
    let client = KinVaultContractClient::new(env, &contract_id);
    client.init(&admin, &token_id);

    (client, admin, token_id, token_client, token_admin)
}

#[test]
fn test_rolling_allowance_and_period_reset() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().with_mut(|l| l.timestamp = 1_000_000);

    let (client, admin, _token_id, token, token_admin) = setup_vault(&env);
    let dependent = Address::generate(&env);

    token_admin.mint(&admin, &1_000_0000000); // 1,000 USDC
    client.fund(&admin, &500_0000000); // 500 USDC

    // 100 USDC weekly allowance
    let week = 7 * 24 * 3600;
    client.set_member(&dependent, &String::from_str(&env, "Tuition"), &100_0000000, &week);
    assert_eq!(client.remaining_allowance(&dependent), 100_0000000);

    // Withdraw 60 USDC
    let rem = client.withdraw(&dependent, &60_0000000);
    assert_eq!(rem, 40_0000000);
    assert_eq!(token.balance(&dependent), 60_0000000);

    // Attempting 50 USDC more in same week must fail
    let res = client.try_withdraw(&dependent, &50_0000000);
    assert_eq!(res, Err(Ok(VaultError::AllowanceExceeded)));

    // Advance ledger by 7 days -> full 100 USDC allowance resets
    env.ledger().with_mut(|l| l.timestamp += week);
    assert_eq!(client.remaining_allowance(&dependent), 100_0000000);

    client.withdraw(&dependent, &100_0000000);
    assert_eq!(token.balance(&dependent), 160_0000000);
}

#[test]
fn test_emergency_freeze_and_member_pause() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().with_mut(|l| l.timestamp = 1_000_000);

    let (client, admin, _token_id, _token, token_admin) = setup_vault(&env);
    let member = Address::generate(&env);

    token_admin.mint(&admin, &500_0000000);
    client.fund(&admin, &500_0000000);
    client.set_member(&member, &String::from_str(&env, "Groceries"), &100_0000000, &86400);

    // Freeze entire vault
    client.set_frozen(&true);
    assert_eq!(
        client.try_withdraw(&member, &10_0000000),
        Err(Ok(VaultError::VaultFrozen))
    );

    // Unfreeze vault, pause individual member
    client.set_frozen(&false);
    client.set_member_paused(&member, &true);
    assert_eq!(
        client.try_withdraw(&member, &10_0000000),
        Err(Ok(VaultError::MemberPaused))
    );

    // Resume member -> withdrawal succeeds
    client.set_member_paused(&member, &false);
    client.withdraw(&member, &25_0000000);
    assert_eq!(client.remaining_allowance(&member), 75_0000000);
}

#[test]
fn test_bonus_release_stacks_with_period_cap() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().with_mut(|l| l.timestamp = 1_000_000);

    let (client, admin, _token_id, token, token_admin) = setup_vault(&env);
    let kid = Address::generate(&env);

    token_admin.mint(&admin, &500_0000000);
    client.fund(&admin, &300_0000000);
    client.set_member(&kid, &String::from_str(&env, "Weekly"), &50_0000000, &604800);

    // Award 25 USDC chore bonus
    client.release_bonus(&kid, &25_0000000);
    assert_eq!(client.remaining_allowance(&kid), 75_0000000);

    // Withdraw 70 USDC (consumes 50 cap + 20 bonus, leaves 5 bonus)
    client.withdraw(&kid, &70_0000000);
    assert_eq!(token.balance(&kid), 70_0000000);
    assert_eq!(client.remaining_allowance(&kid), 5_0000000);
}

#[test]
fn test_member_listing_and_admin_sweep_recovery() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().with_mut(|l| l.timestamp = 1_000_000);

    let (client, admin, _token_id, token, token_admin) = setup_vault(&env);
    let m1 = Address::generate(&env);
    let m2 = Address::generate(&env);

    token_admin.mint(&admin, &400_0000000);
    client.fund(&admin, &400_0000000);
    client.set_member(&m1, &String::from_str(&env, "Books"), &50_0000000, &86400);
    client.set_member(&m2, &String::from_str(&env, "Food"), &80_0000000, &86400);

    let members = client.list_members();
    assert_eq!(members.len(), 2);

    // Admin reclaims 150 USDC
    let remaining = client.admin_withdraw(&150_0000000);
    assert_eq!(remaining, 250_0000000);
    assert_eq!(token.balance(&admin), 150_0000000);

    // Sweeping more than balance fails
    assert_eq!(
        client.try_admin_withdraw(&300_0000000),
        Err(Ok(VaultError::InsufficientVaultBalance))
    );
}

#[test]
fn test_admin_transfer_and_vault_summary() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().with_mut(|l| l.timestamp = 1_000_000);

    let (client, admin, _token_id, _token, token_admin) = setup_vault(&env);
    let new_admin = Address::generate(&env);
    let member = Address::generate(&env);

    token_admin.mint(&admin, &250_0000000);
    client.fund(&admin, &250_0000000);
    client.set_member(&member, &String::from_str(&env, "Transit"), &40_0000000, &86400);

    client.transfer_admin(&new_admin);
    let summary = client.get_summary();
    assert_eq!(summary.admin, new_admin);
    assert_eq!(summary.current_balance, 250_0000000);
    assert_eq!(summary.member_count, 1);
    assert!(!summary.frozen);
}
