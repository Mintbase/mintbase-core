use std::collections::HashMap;

use near_sdk::borsh::{
    self,
    BorshDeserialize,
    BorshSerialize,
};
use near_sdk::{
    env,
    AccountId,
};
use serde::{
    Deserialize,
    Serialize,
};

use crate::common::SafeFraction;

pub type SplitBetweenUnparsed = HashMap<AccountId, u32>;
pub type SplitBetween = HashMap<near_sdk::AccountId, SafeFraction>;

/// A representation of the splitting of ownership of the Token. Percentages
/// must add to 1. On purchase of the `Token`, the value of the transaction
/// (minus royalty percentage) will be paid out to each account in `SplitOwners`
/// mapping. The `SplitOwner` field on the `Token` will be set to `None` after
/// each transfer of the token.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "wasm", derive(BorshDeserialize, BorshSerialize))]
pub struct SplitOwners {
    pub split_between: HashMap<AccountId, SafeFraction>,
}

impl SplitOwners {
    pub fn new(split_between: HashMap<near_sdk::AccountId, u32>) -> Self {
        crate::near_assert!(
            split_between.len() >= 2,
            "Requires at least two accounts to split revenue"
        );
        // validate args
        let mut sum: u32 = 0;
        let split_between: HashMap<AccountId, SafeFraction> = split_between
            .into_iter()
            .map(|(addr, numerator)| {
                crate::near_assert!(
                    // TODO: different method than royalty?
                    env::is_valid_account_id(addr.as_bytes()),
                    "{} is not a valid account ID on NEAR",
                    addr
                );
                let sf = SafeFraction::new(numerator);
                sum += sf.numerator;
                (addr, sf)
            })
            .collect();
        crate::near_assert!(sum == 10_000, "Splits numerators must sum up to 10_000");

        Self { split_between }
    }
}
