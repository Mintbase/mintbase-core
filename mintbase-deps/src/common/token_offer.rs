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

use crate::common::time::now;
use crate::common::{
    NearTime,
    TimeUnit,
};

/// Type representing an offer for a `Token` the marketplace
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "wasm", derive(BorshDeserialize, BorshSerialize))]
pub struct TokenOffer {
    /// The id of this `Offer` is the num of the previous `Offer` + 1. Generated
    /// from the field `Token::num_offers`.
    pub id: u64,
    /// The price the Offerer has posted.
    pub price: u128,
    /// The account who originated the `Offer`.
    pub from: AccountId,
    /// When the `Offer` was made.
    pub timestamp: NearTime,
    /// When the `Offer` will expire.
    pub timeout: NearTime,
}

impl TokenOffer {
    /// Timeout is in days.
    pub fn new(
        price: u128,
        timeout: TimeUnit,
        id: u64,
    ) -> Self {
        let timeout = NearTime::new(timeout);
        Self {
            id,
            price,
            from: env::predecessor_account_id(),
            timestamp: now(),
            timeout,
        }
    }

    /// An offer is active if it has yet to timeout.
    pub fn is_active(&self) -> bool {
        self.timeout.is_before_timeout()
    }
}
