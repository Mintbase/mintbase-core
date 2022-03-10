use std::convert::TryInto;

use near_sdk::borsh::{
    self,
    BorshDeserialize,
    BorshSerialize,
};
use near_sdk::json_types::U128;
use near_sdk::AccountId;
use serde::{
    Deserialize,
    Serialize,
};

use crate::common::{
    TokenKey,
    TokenOffer,
};

/// A Token for sale on the Marketplace.
#[derive(Deserialize, Serialize, Debug)]
#[cfg_attr(feature = "wasm", derive(BorshDeserialize, BorshSerialize))]
pub struct TokenListing {
    /// Id of this `Token`.
    pub id: u64,
    /// Owner of this `Token`.
    pub owner_id: AccountId,
    /// `Store` that originated this `Token`.
    pub store_id: AccountId,
    /// If `autotransfer` is enabled, the Token will automatically be
    /// transferred to an Offerer if their `Offer::price` is greater than the
    /// `asking_price`. Note that enabling `autotransfer` does not
    /// retroactively trigger on the presently held `current_offer`
    pub autotransfer: bool,
    /// The price set by the owner of this Token.
    pub asking_price: U128,
    /// The `approval_id` of the Token allows the Marketplace to transfer the
    /// Token, if purchased. The `approval_id` is also used to generate
    /// unique identifiers for Token-listings.
    pub approval_id: u64,
    /// The current `Offer` for this listing. This `Offer` may have timed
    /// out; if the `Marketplace::min_offer_hours` has transpired, the
    /// `Offer` may be withdrawn by the account in `Offer::from`.
    pub current_offer: Option<TokenOffer>,
    /// The number of `Offer`s that have been made on this listing. Used to
    /// generate Offer `id`s.
    pub num_offers: u64,
    /// When the transfer process is initiated, the token is locked, and no
    /// further changes may be made on the token.
    pub locked: bool,
}

impl TokenListing {
    /// Check that the given `account_id` is valid before instantiating a
    /// `Token`. Note that all input validation for `Token` functions should
    /// be performed at the `Marketplace` level.
    pub fn new(
        owner_id: AccountId,
        store_id: AccountId,
        id: u64,
        approval_id: u64,
        autotransfer: bool,
        asking_price: U128,
    ) -> Self {
        Self {
            id,
            owner_id,
            store_id,
            approval_id,
            autotransfer,
            asking_price,
            current_offer: None,
            num_offers: 0,
            locked: false,
        }
    }

    /// Unique identifier of the Token.
    pub fn get_token_key(&self) -> TokenKey {
        TokenKey::new(self.id, self.store_id.to_string().try_into().unwrap())
    }

    /// Unique identifier of the Token, which is also unique across
    /// relistings of the Token.
    pub fn get_list_id(&self) -> String {
        format!("{}:{}:{}", self.id, self.approval_id, self.store_id)
    }

    pub fn assert_not_locked(&self) {
        assert!(!self.locked);
    }
}
