use near_sdk::borsh::{
    self,
    BorshDeserialize,
    BorshSerialize,
};
#[cfg(feature = "all")]
use near_sdk::serde::{
    Deserialize,
    Serialize,
};

// use near_sdk::AccountId;
use crate::consts::LIST_STORAGE;

#[derive(Clone, Debug)]
#[cfg_attr(feature = "all", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "wasm", derive(BorshDeserialize, BorshSerialize))]
pub struct StorageCosts {
    /// The Near-denominated price-per-byte of storage. As of April 2021, the
    /// price per bytes is set by default to 10^19, but this may change in the
    /// future, thus this future-proofing field.
    pub storage_price_per_byte: u128,
    /// 80 bytes as a Near price. Used for:
    /// - a single royalty
    /// - a single approval
    /// - adding a new entry to the `tokens_per_account` map
    /// - adding a new entry to the `composeables` map
    pub common: u128,
    pub token: u128,
}

impl StorageCosts {
    pub fn new(storage_price_per_byte: u128) -> Self {
        Self {
            storage_price_per_byte,
            common: storage_price_per_byte * 80_u64 as u128,
            token: storage_price_per_byte * 360_u64 as u128,
        }
    }
}

#[cfg_attr(feature = "market-wasm", derive(BorshDeserialize, BorshSerialize))]
pub struct StorageCostsMarket {
    /// The Near-denominated price-per-byte of storage. As of April 2021, the
    /// price per bytes is set by default to 10^19, but this may change in the
    /// future, thus this future-proofing field.
    pub storage_price_per_byte: u128,
    pub list: u128,
}

impl StorageCostsMarket {
    pub fn new(storage_price_per_byte: u128) -> Self {
        Self {
            storage_price_per_byte,
            list: storage_price_per_byte * LIST_STORAGE as u128,
        }
    }
}
