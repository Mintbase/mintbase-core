use near_sdk::env;
use near_sdk::json_types::U64;
use near_sdk::serde::{
    Deserialize,
    Serialize,
};

use crate::logging::NearJsonEvent;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NftMovedLog {
    pub token_id: U64,
    pub contract_id: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NftOnMoveLog {
    pub token_id: U64,
    pub origin_key: String,
}

pub fn log_on_move(
    token_id: U64,
    origin_key: &str,
) {
    let log = NftOnMoveLog {
        token_id,
        origin_key: origin_key.to_string(),
    };
    let event = NearJsonEvent {
        standard: "nep171".to_string(),
        version: "1.0.0".to_string(),
        event: "nft_on_move".to_string(),
        data: serde_json::to_string(&log).unwrap(),
    };
    env::log_str(event.near_json_event().as_str());
}

pub fn log_nft_moved(
    token_id: U64,
    contract_id: String,
) {
    let log = NftMovedLog {
        token_id,
        contract_id,
    };
    let event = NearJsonEvent {
        standard: "nep171".to_string(),
        version: "1.0.0".to_string(),
        event: "nft_moved".to_string(),
        data: serde_json::to_string(&log).unwrap(),
    };
    env::log_str(event.near_json_event().as_str());
}
