use near_sdk::borsh::{
    self,
    BorshDeserialize,
    BorshSerialize,
};
use near_sdk::serde::{
    Deserialize,
    Serialize,
};

/// To enable recursive composeability, need to track:
/// 1. How many levels deep a token is recursively composed
/// 2. Whether and how many cross-contract children a token has.
///
/// Tracking depth limits potential bugs around recursive ownership
/// consuming excessive amounts of gas.
///
/// Tracking the number of cross-contract children a token has prevents
/// breaking of the Only-One-Cross-Linkage Invariant.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "wasm", derive(BorshDeserialize, BorshSerialize))]
pub struct ComposeableStats {
    /// How deep this token is in a chain of composeability on THIS contract.
    /// If this token is cross-composed, it's depth will STILL be 0. `depth`
    /// equal to the parent's `depth`+1. If this is a top level token, this
    /// number is 0.
    pub local_depth: u8,
    /// How many cross contract children this token has, direct AND indirect.
    /// That is, any parent's `cross_contract_children` value equals the sum
    /// of of its children's values. If this number is non-zero, deny calls
    /// to `nft_cross_compose`.
    pub cross_contract_children: u8,
}

impl ComposeableStats {
    pub(crate) fn new() -> Self {
        Self {
            local_depth: 0,
            cross_contract_children: 0,
        }
    }
}
