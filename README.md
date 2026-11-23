# KinVault

**Programmable Family Allowance & Cross-Border Household Budget Vault on Stellar Soroban**

KinVault lets a household manager or diaspora earner fund a single self-custody USDC vault on Stellar and assign rolling weekly or monthly withdrawal allowances to family members—without giving anyone access to the entire balance.

## Why Stellar & Soroban

- **Sub-cent fees:** Weekly $10–$100 family withdrawals aren't eaten by transaction fees.
- **Native USDC / SEP-41:** Works directly with Stellar Asset Contracts (SAC) and SEP-41 tokens.
- **On-chain enforcement:** Rolling period caps, instant emergency freeze, and one-off chore/bonus releases are enforced deterministically by a Rust Soroban smart contract.

## Repository Structure

```text
kinvault/
├── contracts/kinvault/   # Rust Soroban smart contract (soroban-sdk 22)
└── web/                  # Consumer web app (Next.js / TypeScript)
```

## Quick Start

```bash
# Run Soroban contract test suite (real Env + SAC token)
cargo test

# Build optimized WASM target
cargo build --target wasm32-unknown-unknown --release
```

## License

MIT OR Apache-2.0
