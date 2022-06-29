use std::collections::HashMap;

use near_events::near_event_data;
use near_sdk::json_types::U64;
#[cfg(feature = "de")]
use near_sdk::serde::Deserialize;
#[cfg(feature = "ser")]
use near_sdk::serde::Serialize;
use near_sdk::{
    env,
    AccountId,
};

#[cfg_attr(feature = "all", derive(Debug, Clone))]
#[near_event_data(
    standard = "mb_store",
    version = "0.1.0",
    event = "nft_set_split_owners"
)]
pub struct NftSetSplitOwnerData {
    pub token_ids: Vec<U64>,
    pub split_owners: HashMap<AccountId, u16>,
}

pub fn log_set_split_owners(
    token_ids: Vec<U64>,
    mut split_owners: crate::common::SplitOwners,
) {
    env::log_str(
        &NftSetSplitOwnerData {
            token_ids,
            split_owners: split_owners
                .split_between
                .drain()
                .map(|(acc, fraction)| (acc, fraction.numerator as u16))
                .collect(),
        }
        .serialize_event(),
    );
}
