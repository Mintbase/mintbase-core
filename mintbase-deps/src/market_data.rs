use near_sdk::borsh::{
    self,
    BorshDeserialize,
    BorshSerialize,
};
use near_sdk::json_types::U128;
use near_sdk::{
    env,
    AccountId,
};
use serde::{
    Deserialize,
    Serialize,
};

use crate::utils::TokenKey;

/// A Token for sale on the Marketplace.
#[derive(Deserialize, Serialize, Debug, BorshDeserialize, BorshSerialize)]
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
        TokenKey {
            token_id: self.id,
            account_id: self.store_id.to_string(),
        }
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

/// Type representing an offer for a `Token` the marketplace
#[derive(Serialize, Deserialize, Clone, Debug, BorshDeserialize, BorshSerialize)]
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
        Self {
            id,
            price,
            from: env::predecessor_account_id(),
            timestamp: NearTime::now(),
            timeout: NearTime::new(timeout),
        }
    }

    /// An offer is active if it has yet to timeout.
    pub fn is_active(&self) -> bool {
        self.timeout.is_before_timeout()
    }
}

/// Time duration.
/// This enum used to support other time denominations, which were dropped
/// for simplicity.
#[derive(Debug, Serialize, Deserialize, Clone, BorshSerialize, BorshDeserialize)]
pub enum TimeUnit {
    Hours(u64),
}

/// Time instant, the u64 is in nanoseconds since epoch.
#[derive(Debug, Serialize, Deserialize, Clone, BorshDeserialize, BorshSerialize)]
pub struct NearTime(pub u64);

impl NearTime {
    fn is_before_timeout(&self) -> bool {
        env::block_timestamp() < self.0
    }

    fn new(span: TimeUnit) -> Self {
        match span {
            TimeUnit::Hours(n) => Self::now_plus_n_hours(n),
        }
    }

    fn now() -> Self {
        Self(env::block_timestamp())
    }

    fn now_plus_n_hours(n: u64) -> Self {
        crate::near_assert!(n > 0, "Cannot set times into the past");
        crate::near_assert!(
            n < 70_000,
            "Cannot set times more than 70_000 hours into the future (~8 years)"
        );
        let now = env::block_timestamp();
        let hour_ns = 10u64.pow(9) * 3600;
        Self(now + n * hour_ns)
    }
}

/// ref: https://github.com/near-apps/nft-market/blob/main/contracts/market-simple/src/lib.rs#L54
#[derive(Serialize, Deserialize)]
pub struct SaleArgs {
    pub price: U128,
    pub autotransfer: bool,
}
