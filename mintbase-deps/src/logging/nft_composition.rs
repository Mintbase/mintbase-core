use near_sdk::json_types::U64;
use near_sdk::serde::{
    Deserialize,
    Serialize,
};
use near_sdk::{
    env,
    AccountId,
};

use crate::logging::NearJsonEvent;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NftComposeLog {
    pub token_ids: Vec<U64>,
    /// direct parent of token_ids
    pub parent: String,
    /// - "t": owned directly by a token on this contract
    /// - "k": owned directly by a token on another contract
    pub ttype: String,
    /// local root of chain of token_ids
    pub lroot: Option<u64>,
    /// holder of local root
    pub holder: String,
    pub depth: u8,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NftUncomposeLog {
    pub token_ids: Vec<U64>,
    pub holder: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NftOnComposeLog {
    pub predecessor: String,
    pub token_id: U64,
    /// direct parent of token_ids
    pub cross_child_id: U64,
    /// local root of chain of token_ids
    pub lroot: Option<u64>,
    /// holder of local root
    pub holder: String,
    pub depth: u8,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NftOnUncomposeLog {
    pub token_id: U64,
    pub holder: String,
    pub child_key: String,
}

pub fn log_nfts_compose(
    token_ids: &[U64],
    // direct parent of token_ids
    parent: &str,
    // - "t": owned directly by a token on this contract
    // - "k": owned directly by a token on another contract
    ttype: String,
    // local root of chain of token_ids
    lroot: Option<u64>,
    // holder of local root
    holder: String,
    depth: u8,
) {
    let log = NftComposeLog {
        token_ids: token_ids.to_vec(),
        parent: parent.to_string(),
        ttype,
        lroot,
        holder,
        depth,
    };
    let event = NearJsonEvent {
        standard: "nep171".to_string(),
        version: "1.0.0".to_string(),
        event: "nft_compose".to_string(),
        data: serde_json::to_string(&log).unwrap(),
    };
    env::log_str(event.near_json_event().as_str());
}

pub fn log_nfts_uncompose(
    token_ids: &[U64],
    holder: AccountId,
) {
    let log = NftUncomposeLog {
        token_ids: token_ids.to_vec(),
        holder: holder.to_string(),
    };
    let event = NearJsonEvent {
        standard: "nep171".to_string(),
        version: "1.0.0".to_string(),
        event: "nft_uncompose".to_string(),
        data: serde_json::to_string(&log).unwrap(),
    };
    env::log_str(event.near_json_event().as_str());
}

pub fn log_on_compose(
    predecessor: AccountId,
    token_id: U64,
    // direct parent of token_ids
    cross_child_id: U64,
    // local root of chain of token_ids
    lroot: Option<u64>,
    // holder of local root
    holder: String,
    depth: u8,
) {
    let log = NftOnComposeLog {
        predecessor: predecessor.to_string(),
        token_id,
        cross_child_id,
        lroot,
        holder,
        depth,
    };
    let event = NearJsonEvent {
        standard: "nep171".to_string(),
        version: "1.0.0".to_string(),
        event: "nft_on_compose".to_string(),
        data: serde_json::to_string(&log).unwrap(),
    };
    env::log_str(event.near_json_event().as_str());
}

pub fn log_on_uncompose(
    token_id: U64,
    holder: &str,
    child_key: String,
) {
    let log = NftOnUncomposeLog {
        token_id,
        holder: holder.to_string(),
        child_key,
    };
    let event = NearJsonEvent {
        standard: "nep171".to_string(),
        version: "1.0.0".to_string(),
        event: "nft_on_uncompose".to_string(),
        data: serde_json::to_string(&log).unwrap(),
    };
    env::log_str(event.near_json_event().as_str());
}
