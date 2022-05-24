use std::collections::HashMap;

use near_events::{
    near_event_data,
    near_event_data_log,
};
use near_sdk::json_types::{
    U128,
    U64,
};
#[cfg(feature = "de")]
use near_sdk::serde::Deserialize;
#[cfg(feature = "ser")]
use near_sdk::serde::Serialize;
use near_sdk::{
    env,
    AccountId,
};

use crate::common::TokenOffer;

// ----------------------------- create listing ----------------------------- //
#[cfg_attr(feature = "ser", derive(Serialize))]
#[cfg_attr(feature = "de", derive(Deserialize))]
#[cfg_attr(any(feature = "ser", feature = "de"), serde(crate = "near_sdk::serde"))]
pub struct NftListLog {
    pub list_id: String,
    pub price: String,
    pub token_key: String,
    pub owner_id: String,
    pub autotransfer: bool,
    pub approval_id: String,
    pub token_id: String,
    pub store_id: String,
}

#[near_event_data(standard = "mb_market", version = "0.1.0", event = "nft_list")]
pub struct NftListData(Vec<NftListLog>);

#[cfg(feature = "ser")]
pub fn log_listing_created(
    list_id: &str,
    price: &U128,
    token_key: &str,
    owner_id: &AccountId,
    autotransfer: bool,
) {
    let mut iter = token_key.split(':');
    let mut iter2 = list_id.split(':');
    let token_id = iter.next();
    let store_id = iter.next();
    iter2.next();
    let approval_id = iter2.next().unwrap();
    let data = NftListData(vec![NftListLog {
        list_id: list_id.to_string(),
        price: price.0.to_string(),
        token_key: token_key.to_string(),
        owner_id: owner_id.to_string(),
        autotransfer,
        approval_id: approval_id.to_string(),
        token_id: token_id.unwrap().to_string(),
        store_id: store_id.unwrap().to_string(),
    }]);
    env::log_str(&data.serialize_event());
}

#[cfg(feature = "ser")]
pub fn log_batch_listing_created(
    approval_ids: &[U64],
    price: &U128,
    token_ids: &[U64],
    owner_id: &AccountId,
    store_id: &AccountId,
    autotransfer: bool,
) {
    let data = NftListData(
        approval_ids
            .iter()
            .enumerate()
            .map(|(u, x)| {
                let list_id = format!("{}:{}:{}", token_ids[u].0, x.0, store_id);
                let token_key = format!("{}:{}", token_ids[u].0, store_id);
                NftListLog {
                    list_id,
                    price: price.0.to_string(),
                    token_key,
                    owner_id: owner_id.to_string(),
                    autotransfer,
                    approval_id: x.0.to_string(),
                    token_id: token_ids[u].0.to_string(),
                    store_id: store_id.to_string(),
                }
            })
            .collect::<Vec<_>>(),
    );
    env::log_str(&data.serialize_event());
}

// ---------------------------- update listings ----------------------------- //
// TODO: test!
#[cfg_attr(feature = "all", derive(Clone, Debug))]
#[near_event_data(standard = "mb_market", version = "0.1.0", event = "nft_update_list")]
pub struct NftUpdateListLog {
    pub list_id: String,
    pub auto_transfer: Option<bool>,
    pub price: Option<String>,
}

#[cfg(feature = "ser")]
pub fn log_set_token_autotransfer(
    auto_transfer: bool,
    list_id: &str,
) {
    let data = NftUpdateListLog {
        list_id: list_id.to_string(),
        auto_transfer: Option::from(auto_transfer),
        price: None,
    };
    env::log_str(&data.serialize_event());
}

#[cfg(feature = "ser")]
pub fn log_set_token_asking_price(
    price: &U128,
    list_id: &str,
) {
    let data = NftUpdateListLog {
        list_id: list_id.to_string(),
        auto_transfer: None,
        price: Option::from(price.0.to_string()),
    };
    env::log_str(&data.serialize_event());
}

// ----------------------------- creating offer ----------------------------- //
#[cfg_attr(feature = "all", derive(Clone, Debug))]
#[cfg_attr(feature = "ser", derive(Serialize))]
#[cfg_attr(feature = "de", derive(Deserialize))]
#[cfg_attr(any(feature = "ser", feature = "de"), serde(crate = "near_sdk::serde"))]
pub struct NftMakeOfferLog {
    pub offer: TokenOffer, // TODO: TokenOfferJson to stringify u128?
    pub list_id: String,
    pub token_key: String,
    pub offer_num: u64,
}

// FIXME: u128 is not supported -_____-
#[cfg_attr(feature = "all", derive(Clone, Debug))]
#[near_event_data(standard = "mb_market", version = "0.1.0", event = "nft_make_offer")]
pub struct NftMakeOfferData(Vec<NftMakeOfferLog>);

#[cfg(feature = "ser")]
pub fn log_make_offer(
    offer: Vec<&TokenOffer>,
    token_key: Vec<&String>,
    list_id: Vec<String>,
    offer_num: Vec<u64>,
) {
    let data = NftMakeOfferData(
        offer
            .iter()
            .enumerate()
            .map(|(u, &x)| NftMakeOfferLog {
                offer: x.clone(),
                list_id: list_id[u].clone(),
                token_key: token_key[u].clone(),
                offer_num: offer_num[u],
            })
            .collect::<Vec<_>>(),
    );
    env::log_str(&data.serialize_event());
}

// --------------------------- withdrawing offer ---------------------------- //
#[cfg_attr(feature = "all", derive(Clone, Debug))]
#[near_event_data(standard = "mb_market", version = "0.1.0", event = "nft_update_offer")]
pub struct NftUpdateOfferData {
    pub list_id: String,
    pub offer_num: u64,
}

#[cfg(feature = "ser")]
pub fn log_withdraw_token_offer(
    list_id: &str,
    offer_num: u64,
) {
    let data = NftUpdateOfferData {
        offer_num,
        list_id: list_id.to_string(),
    };
    env::log_str(&data.serialize_event());
}

// ------------------------------ sell listing ------------------------------ //
#[cfg_attr(feature = "all", derive(Clone, Debug))]
#[near_event_data(standard = "mb_market", version = "0.1.0", event = "nft_sold")]
pub struct NftSaleData {
    pub list_id: String,
    pub offer_num: u64,
    pub token_key: String,
    pub payout: HashMap<AccountId, U128>,
}

#[cfg(feature = "ser")]
pub fn log_sale(
    list_id: &str,
    offer_num: u64,
    token_key: &str,
    payout: &HashMap<AccountId, U128>,
) {
    let data = NftSaleData {
        list_id: list_id.to_string(),
        offer_num,
        token_key: token_key.to_string(),
        payout: payout.clone(),
    };
    env::log_str(&data.serialize_event());
}

// ------------------------------- unlisting -------------------------------- //
#[cfg_attr(feature = "all", derive(Clone, Debug))]
#[near_event_data_log(standard = "mb_market", version = "0.1.0", event = "nft_unlist")]
pub struct NftUnlistLog {
    list_id: String,
}

#[cfg(feature = "ser")]
pub fn log_token_removed(list_id: &str) {
    let log = NftUnlistLog {
        list_id: list_id.to_string(),
    };
    env::log_str(&log.serialize_event());
}

// ----------------------- updating banlist/allowlist ----------------------- //
#[cfg_attr(feature = "all", derive(Clone, Debug))]
#[near_event_data(standard = "mb_market", version = "0.1.0", event = "update_banlist")]
pub struct UpdateBanlistData {
    pub account_id: String,
    pub state: bool,
}

#[cfg(feature = "ser")]
pub fn log_banlist_update(
    account_id: &AccountId,
    state: bool,
) {
    let data = UpdateBanlistData {
        account_id: account_id.to_string(),
        state,
    };
    env::log_str(&data.serialize_event());
}

#[cfg_attr(feature = "all", derive(Clone, Debug))]
#[near_event_data(standard = "mb_market", version = "0.1.0", event = "update_allowlist")]
pub struct UpdateAllowlistData {
    pub account_id: String,
    pub state: bool,
}

#[cfg(feature = "ser")]
pub fn log_allowlist_update(
    account_id: &AccountId,
    state: bool,
) {
    let data = UpdateAllowlistData {
        account_id: account_id.to_string(),
        state,
    };
    env::log_str(&data.serialize_event());
}
