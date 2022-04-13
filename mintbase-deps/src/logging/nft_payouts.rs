use near_sdk::env;
use near_sdk::json_types::U64;
use near_sdk::serde::{
    Deserialize,
    Serialize,
};

use crate::logging::NearJsonEvent;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NftSetSplitOwnerLog {
    pub split_owners: crate::common::SplitOwners,
    pub token_ids: Vec<String>,
}

pub fn log_set_split_owners(
    token_ids: &[U64],
    split_owners: &crate::common::SplitOwners,
) {
    let token_ids = token_ids
        .iter()
        .map(|x| x.0.to_string())
        .collect::<Vec<_>>();

    let log = NftSetSplitOwnerLog {
        split_owners: split_owners.clone(),
        token_ids,
    };
    let event = NearJsonEvent {
        standard: "nep171".to_string(),
        version: "1.0.0".to_string(),
        event: "nft_set_split_owners".to_string(),
        data: serde_json::to_string(&log).unwrap(),
    };
    env::log_str(event.near_json_event().as_str());
    //         .to_string()
    //         .as_str(),
    // );
}
