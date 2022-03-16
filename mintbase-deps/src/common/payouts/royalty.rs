use std::collections::HashMap;
use std::convert::TryFrom;

use near_sdk::borsh::{
    self,
    BorshDeserialize,
    BorshSerialize,
};
use near_sdk::serde::{
    Deserialize,
    Serialize,
};
use near_sdk::AccountId;

use crate::common::{
    SafeFraction,
    SplitBetween,
    SplitBetweenUnparsed,
};
use crate::constants::ROYALTY_UPPER_LIMIT;

/// A representation of permanent partial ownership of a Token's revenues.
/// Percentages must add to 10,000. On purchase of the `Token`, a percentage of
/// the value of the transaction will be paid out to each account in the
/// `Royalty` mapping. `Royalty` field once set can NEVER change for this
/// `Token`, even if removed and re-added.
#[derive(PartialEq)]
#[cfg_attr(feature = "wasm", derive(BorshDeserialize, BorshSerialize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Royalty {
    /// Mapping of addresses to relative percentages of the overall royalty percentage
    pub split_between: HashMap<near_sdk::AccountId, SafeFraction>,
    /// The overall royalty percentage taken
    pub percentage: SafeFraction,
}

/// Stable
impl Royalty {
    /// Validates all arguments. Addresses must be valid and percentages must be
    /// within accepted values. Hashmap percentages must add to 10000.
    pub fn new(royalty_args: RoyaltyArgs) -> Self {
        assert!(!royalty_args.split_between.is_empty());
        let percentage = royalty_args.percentage;
        let split_between = royalty_args.split_between;

        assert!(
            percentage <= ROYALTY_UPPER_LIMIT,
            "percentage: {} must be <= 5000",
            percentage
        );
        assert!(percentage > 0, "percentage cannot be zero");
        assert!(!split_between.is_empty(), "royalty mapping is empty");

        let mut sum: u32 = 0;
        let split_between: SplitBetween = split_between
            .into_iter()
            .map(|(addr, numerator)| {
                assert!(AccountId::try_from(addr.to_string()).is_ok());
                // assert!(env::is_valid_account_id(addr.as_bytes()));
                assert!(numerator > 0, "percentage cannot be zero");
                let sf = SafeFraction::new(numerator);
                sum += sf.numerator;
                (addr, sf)
            })
            .collect();
        assert_eq!(sum, 10_000, "fractions don't add to 10,000");

        Self {
            percentage: SafeFraction::new(percentage),
            split_between,
        }
    }
}

/// Unparsed pre-image of a Royalty struct. Used in `Store::mint_tokens`.
#[derive(Clone, Deserialize, Serialize)]
pub struct RoyaltyArgs {
    pub split_between: SplitBetweenUnparsed,
    pub percentage: u32,
}
