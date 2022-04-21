use near_sdk::serde::{
    Deserialize,
    Serialize,
};
use near_sdk::{
    env,
    AccountId,
};

use crate::common::NFTContractMetadata;
use crate::logging::{
    NearJsonEvent,
    NftOptionStringLog,
    NftStringLog,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NftStoreCreateLog {
    pub contract_metadata: NFTContractMetadata,
    pub owner_id: String,
    pub id: String,
}

impl Default for NftStoreCreateLog {
    fn default() -> Self {
        Self {
            contract_metadata: Default::default(),
            owner_id: "".to_string(),
            id: "".to_string(),
        }
    }
}

pub fn log_grant_minter(account_id: &AccountId) {
    let log = NftStringLog {
        data: account_id.to_string(),
    };
    let event = NearJsonEvent {
        standard: "nep171".to_string(),
        version: "1.0.0".to_string(),
        event: "nft_grant_minter".to_string(),
        data: serde_json::to_string(&log).unwrap(),
    };
    env::log_str(event.near_json_event().as_str());
}

pub fn log_revoke_minter(account_id: &AccountId) {
    let log = NftStringLog {
        data: account_id.to_string(),
    };
    let event = NearJsonEvent {
        standard: "nep171".to_string(),
        version: "1.0.0".to_string(),
        event: "nft_revoke_minter".to_string(),
        data: serde_json::to_string(&log).unwrap(),
    };
    env::log_str(event.near_json_event().as_str());
}

pub fn log_transfer_store(to: &AccountId) {
    let log = NftStringLog {
        data: to.to_string(),
    };
    let event = NearJsonEvent {
        standard: "nep171".to_string(),
        version: "1.0.0".to_string(),
        event: "nft_transfer_store".to_string(),
        data: serde_json::to_string(&log).unwrap(),
    };
    env::log_str(event.near_json_event().as_str());
}

pub fn log_set_icon_base64(base64: &Option<String>) {
    let log = NftOptionStringLog {
        data: base64.clone(),
    };
    let event = NearJsonEvent {
        standard: "nep171".to_string(),
        version: "1.0.0".to_string(),
        event: "nft_set_icon_base64".to_string(),
        data: serde_json::to_string(&log).unwrap(),
    };
    env::log_str(event.near_json_event().as_str());
}

pub fn log_set_base_uri(base_uri: &str) {
    let log = NftStringLog {
        data: base_uri.to_string(),
    };
    let event = NearJsonEvent {
        standard: "nep171".to_string(),
        version: "1.0.0".to_string(),
        event: "nft_set_base_uri".to_string(),
        data: serde_json::to_string(&log).unwrap(),
    };
    env::log_str(event.near_json_event().as_str());
}
