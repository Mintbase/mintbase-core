use near_sdk::json_types::U64;
use near_sdk::serde::{
    Deserialize,
    Serialize,
};
use near_sdk::{
    env,
    AccountId,
};

use crate::logging::{
    NearJsonEvent,
    NftStringLog,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NftApproveLog {
    pub token_id: u64,
    pub approval_id: u64,
    pub account_id: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NftRevokeLog {
    pub token_id: u64,
    pub account_id: String,
}

pub fn log_approve(
    token_id: u64,
    approval_id: u64,
    account_id: &AccountId,
) {
    let log = vec![NftApproveLog {
        token_id,
        approval_id,
        account_id: account_id.to_string(),
    }];
    let event = NearJsonEvent {
        standard: "nep171".to_string(),
        version: "1.0.0".to_string(),
        event: "nft_approve".to_string(),
        data: serde_json::to_string(&log).unwrap(),
    };
    env::log_str(event.near_json_event().as_str());
}

pub fn log_batch_approve(
    tokens: &[U64],
    approvals: &[U64],
    account_id: &AccountId,
) {
    let log = approvals
        .iter()
        .enumerate()
        .map(|(u, x)| NftApproveLog {
            token_id: tokens[u].0,
            approval_id: x.0,
            account_id: account_id.to_string(),
        })
        .collect::<Vec<_>>();
    let event = NearJsonEvent {
        standard: "nep171".to_string(),
        version: "1.0.0".to_string(),
        event: "nft_approve".to_string(),
        data: serde_json::to_string(&log).unwrap(),
    };
    env::log_str(event.near_json_event().as_str());
}

pub fn log_revoke(
    token_id: u64,
    account_id: &AccountId,
) {
    let log = NftRevokeLog {
        token_id,
        account_id: account_id.to_string(),
    };
    let event = NearJsonEvent {
        standard: "nep171".to_string(),
        version: "1.0.0".to_string(),
        event: "nft_revoke".to_string(),
        data: serde_json::to_string(&log).unwrap(),
    };
    env::log_str(event.near_json_event().as_str());
}

pub fn log_revoke_all(token_id: u64) {
    let log = NftStringLog {
        data: token_id.to_string(),
    };
    let event = NearJsonEvent {
        standard: "nep171".to_string(),
        version: "1.0.0".to_string(),
        event: "nft_revoke_all".to_string(),
        data: serde_json::to_string(&log).unwrap(),
    };
    env::log_str(event.near_json_event().as_str());
}
