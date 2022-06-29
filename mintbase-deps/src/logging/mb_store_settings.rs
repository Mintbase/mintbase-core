use near_events::near_event_data;
#[cfg(feature = "de")]
use near_sdk::serde::Deserialize;
#[cfg(feature = "ser")]
use near_sdk::serde::Serialize;
use near_sdk::{
    env,
    AccountId,
};

use crate::common::NFTContractMetadata;

#[near_event_data(standard = "mb_store", version = "0.1.0", event = "deploy")]
pub struct MbStoreDeployData {
    pub contract_metadata: NFTContractMetadata,
    pub owner_id: String,
    pub store_id: String,
}

#[near_event_data(standard = "mb_store", version = "0.1.0", event = "change_setting")]
pub struct MbStoreChangeSettingData {
    pub granted_minter: Option<String>,
    pub revoked_minter: Option<String>,
    pub new_owner: Option<String>,
    pub new_icon_base64: Option<String>,
    pub new_base_uri: Option<String>,
}

impl MbStoreChangeSettingData {
    fn empty() -> Self {
        MbStoreChangeSettingData {
            granted_minter: None,
            revoked_minter: None,
            new_owner: None,
            new_icon_base64: None,
            new_base_uri: None,
        }
    }
}

pub fn log_grant_minter(account_id: &AccountId) {
    env::log_str(
        &MbStoreChangeSettingData {
            granted_minter: Some(account_id.to_string()),
            ..MbStoreChangeSettingData::empty()
        }
        .serialize_event(),
    );
}

pub fn log_revoke_minter(account_id: &AccountId) {
    env::log_str(
        &MbStoreChangeSettingData {
            revoked_minter: Some(account_id.to_string()),
            ..MbStoreChangeSettingData::empty()
        }
        .serialize_event(),
    );
}

pub fn log_transfer_store(account_id: &AccountId) {
    env::log_str(
        &MbStoreChangeSettingData {
            new_owner: Some(account_id.to_string()),
            ..MbStoreChangeSettingData::empty()
        }
        .serialize_event(),
    );
}

pub fn log_set_icon_base64(base64: &Option<String>) {
    // this will not take care of icon deletion -> no accessible via UI
    // TODO: document for coders that deletion will happen e.g. by inserting
    //  empty icon
    env::log_str(
        &MbStoreChangeSettingData {
            new_icon_base64: base64.clone(),
            ..MbStoreChangeSettingData::empty()
        }
        .serialize_event(),
    );
}

pub fn log_set_base_uri(base_uri: &str) {
    // TODO: disallow this setting anyhow -> configurable on deploy only
    env::log_str(
        &MbStoreChangeSettingData {
            new_base_uri: Some(base_uri.to_string()),
            ..MbStoreChangeSettingData::empty()
        }
        .serialize_event(),
    );
}
