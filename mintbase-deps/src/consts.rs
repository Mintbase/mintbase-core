use crate::SafeFraction;
use near_sdk::{
    Balance,
    Gas,
};
use std::collections::HashMap;

/// Near denominated units are in 10^24
pub const fn ntoy(near_amount: Balance) -> Balance {
    near_amount * 10u128.pow(24)
}

pub const ONE_YOCTO: Balance = 1;

pub const GAS_PAYOUT_RESOLVE: Gas = ntot(Gas(30));

/// The amount of Storage in bytes consumed by a maximal sized Token with NO
/// metadata and NO Royalty field. Rounded to 360 for extra cushion.
pub const LIST_STORAGE: near_sdk::StorageUsage = 360;

pub const GAS_NFT_TRANSFER_PAYOUT: Gas = ntot(Gas(15));

pub const GAS_CREATE_STORE: Gas = ntot(Gas(65 + 5));
pub const GAS_ON_CREATE_CALLBACK: Gas = ntot(Gas(10));
pub const STORE_STORAGE: u64 = 550_000; // 499kB

pub type SplitBetween = HashMap<near_sdk::AccountId, SafeFraction>;

/// The argument for non-payable cross contract calls.
/// ref: https://github.com/near/core-contracts/blob/master/staking-pool/src/lib.rs#L26
pub const NO_DEPOSIT: Balance = 0;

/// Royalty upper limit is 50%.
pub const ROYALTY_UPPER_LIMIT: u32 = 5000;

/// Gas is in TerraUnits, default gas call is 100TGas.
pub const fn ntot(near_amount: Gas) -> Gas {
    Gas(near_amount.0 * 10u64.pow(12))
}

pub const MAX_LEN_PAYOUT: u32 = 50;

pub const MINIMUM_CUSHION: near_sdk::Balance = 5 * 10u128.pow(23);

//?
pub const GAS_NFT_BATCH_APPROVE: Gas = ntot(Gas(100));

/// The amount of Storage in bytes consumed by a maximal sized Token with NO
/// metadata and NO Royalty field. Rounded to 360 for extra cushion.
pub const TOKEN_STORAGE: near_sdk::StorageUsage = 360;
/// The storage in bytes (with a little padding) for:
/// - a single royalty
/// - a single approval
/// - adding a new entry to the `tokens_per_account` map
/// - adding a new entry to the `composeables` map
pub const COMMON_STORAGE: near_sdk::StorageUsage = 80;
// ref: https://github.com/near-apps/nft-market/blob/main/contracts/nft-simple/src/nft_core.rs
pub const GAS_RESOLVE_TRANSFER: u64 = 10_000_000_000_000;
pub const GAS_NFT_TRANSFER_CALL: u64 = 25_000_000_000_000 + GAS_RESOLVE_TRANSFER;

// https://explorer.testnet.near.org/transactions/B2u2b1zenw9zBnC97G8DEzVWmVnXA92bNGsPJCHkdfMw
// 10 locally.
// 5 on market + 16 on market if market has to discard a token + cushion
//pub(crate) const GAS_NFT_APPROVE: u64 = mintbase_utils::ntot(40);
#[cfg(feature = "store-wasm")]
pub(crate) const GAS_PASS_TO_APPROVED: Gas = ntot(Gas(25));
