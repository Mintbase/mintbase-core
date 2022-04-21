use std::collections::HashMap;

use near_sdk::json_types::{
    U128,
    U64,
};
use near_sdk::serde::{
    Deserialize,
    Serialize,
};
use near_sdk::{
    env,
    AccountId,
};

use crate::common::TokenOffer;
use crate::logging::{
    NearJsonEvent,
    NftStringLog,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
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

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NftUpdateListLog {
    pub auto_transfer: Option<bool>,
    pub price: Option<String>,
    pub list_id: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NftOfferLog2 {
    pub offer: TokenOffer,
    pub list_id: String,
    pub token_key: String,
    pub offer_num: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NftOfferLog {
    pub price: String,
    pub from: String,
    pub timeout: String,
    pub list_id: String,
    pub token_key: String,
    pub offer_num: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NftUpdateOfferLog {
    pub list_id: String,
    pub offer_num: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NftSaleLog {
    pub list_id: String,
    pub offer_num: u64,
    pub token_key: String,
    pub payout: HashMap<AccountId, U128>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NftMarketLog {
    pub account_id: String,
    pub state: bool,
}

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
    let log = vec![NftListLog {
        list_id: list_id.to_string(),
        price: price.0.to_string(),
        token_key: token_key.to_string(),
        owner_id: owner_id.to_string(),
        autotransfer,
        approval_id: approval_id.to_string(),
        token_id: token_id.unwrap().to_string(),
        store_id: store_id.unwrap().to_string(),
    }];
    let event = NearJsonEvent {
        standard: "nep171".to_string(),
        version: "1.0.0".to_string(),
        event: "nft_1_list".to_string(),
        data: serde_json::to_string(&log).unwrap(),
    };
    env::log_str(event.near_json_event().as_str());
}

pub fn log_batch_listing_created(
    approval_ids: &[U64],
    price: &U128,
    token_ids: &[U64],
    owner_id: &AccountId,
    store_id: &AccountId,
    autotransfer: bool,
) {
    let log = approval_ids
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
        .collect::<Vec<_>>();
    let event = NearJsonEvent {
        standard: "nep171".to_string(),
        version: "1.0.0".to_string(),
        event: "nft_batch_list".to_string(),
        data: serde_json::to_string(&log).unwrap(),
    };
    env::log_str(event.near_json_event().as_str());
}

pub fn log_set_token_autotransfer(
    auto_transfer: bool,
    list_id: &str,
) {
    let log = vec![NftUpdateListLog {
        auto_transfer: Option::from(auto_transfer),
        price: None,
        list_id: Option::from(list_id.to_string()),
    }];
    let event = NearJsonEvent {
        standard: "nep171".to_string(),
        version: "1.0.0".to_string(),
        event: "nft_set_autotransfer".to_string(),
        data: serde_json::to_string(&log).unwrap(),
    };
    env::log_str(event.near_json_event().as_str());
}

pub fn log_set_token_asking_price(
    price: &U128,
    list_id: &str,
) {
    let log = vec![NftUpdateListLog {
        auto_transfer: None,
        price: Option::from(price.0.to_string()),
        list_id: Option::from(list_id.to_string()),
    }];
    let event = NearJsonEvent {
        standard: "nep171".to_string(),
        version: "1.0.0".to_string(),
        event: "nft_set_price".to_string(),
        data: serde_json::to_string(&log).unwrap(),
    };
    env::log_str(event.near_json_event().as_str());
}

pub fn log_make_offer(
    offer: Vec<&TokenOffer>,
    token_key: Vec<&String>,
    list_id: Vec<String>,
    offer_num: Vec<u64>,
) {
    let log = offer
        .iter()
        .enumerate()
        .map(|(u, &x)| NftOfferLog2 {
            offer: x.clone(),
            list_id: list_id[u].clone(),
            token_key: token_key[u].clone(),
            offer_num: offer_num[u],
        })
        .collect::<Vec<_>>();
    let event = NearJsonEvent {
        standard: "nep171".to_string(),
        version: "1.0.0".to_string(),
        event: "nft_make_offer".to_string(),
        data: serde_json::to_string(&log).unwrap(),
    };
    env::log_str(event.near_json_event().as_str());
}

pub fn log_withdraw_token_offer(
    list_id: &str,
    offer_num: u64,
) {
    let log = NftUpdateOfferLog {
        offer_num,
        list_id: list_id.to_string(),
    };
    let event = NearJsonEvent {
        standard: "nep171".to_string(),
        version: "1.0.0".to_string(),
        event: "nft_withdraw_offer".to_string(),
        data: serde_json::to_string(&log).unwrap(),
    };
    env::log_str(event.near_json_event().as_str());
}

pub fn log_sale(
    list_id: &str,
    offer_num: u64,
    token_key: &str,
    payout: &HashMap<AccountId, U128>,
) {
    let log = NftSaleLog {
        list_id: list_id.to_string(),
        offer_num,
        token_key: token_key.to_string(),
        payout: payout.clone(),
    };
    let event = NearJsonEvent {
        standard: "nep171".to_string(),
        version: "1.0.0".to_string(),
        event: "nft_sold".to_string(),
        data: serde_json::to_string(&log).unwrap(),
    };
    env::log_str(event.near_json_event().as_str());
}

pub fn log_token_removed(list_id: &str) {
    let log = NftStringLog {
        data: list_id.to_string(),
    };
    let event = NearJsonEvent {
        standard: "nep171".to_string(),
        version: "1.0.0".to_string(),
        event: "nft_removed".to_string(),
        data: serde_json::to_string(&log).unwrap(),
    };
    env::log_str(event.near_json_event().as_str());
}

pub fn log_banlist_update(
    account_id: &AccountId,
    state: bool,
) {
    let log = vec![NftMarketLog {
        account_id: account_id.to_string(),
        state,
    }];
    let event = NearJsonEvent {
        standard: "nep171".to_string(),
        version: "1.0.0".to_string(),
        event: "nft_banlist".to_string(),
        data: serde_json::to_string(&log).unwrap(),
    };
    env::log_str(event.near_json_event().as_str());
}

pub fn log_allowlist_update(
    account_id: &AccountId,
    state: bool,
) {
    let log = vec![NftMarketLog {
        account_id: account_id.to_string(),
        state,
    }];
    let event = NearJsonEvent {
        standard: "nep171".to_string(),
        version: "1.0.0".to_string(),
        event: "nft_allowlist".to_string(),
        data: serde_json::to_string(&log).unwrap(),
    };
    env::log_str(event.near_json_event().as_str());
}
