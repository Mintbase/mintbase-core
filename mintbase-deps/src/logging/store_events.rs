use near_events::{
    near_event_data,
    near_event_data_log,
};
use near_sdk::json_types::U64;
#[cfg(feature = "de")]
use near_sdk::serde::Deserialize;
#[cfg(feature = "ser")]
use near_sdk::serde::Serialize;
use near_sdk::AccountId;

// ----------------------------- Core (NEP171) ------------------------------ //
#[cfg_attr(feature = "all", derive(Clone, Debug))]
#[near_event_data_log(standard = "nep171", version = "1.0.0", event = "nft_mint")]
pub struct NftMintLog {
    pub owner_id: String,
    pub token_ids: Vec<String>,
    pub memo: Option<String>,
}

// #[near_event_data(standard = "nep171", version = "1.0.0", event = "nft_mint")]
// pub struct NftMintData(Vec<NftMintLog>);

#[near_event_data_log(standard = "nep171", version = "1.0.0", event = "nft_burn")]
pub struct NftBurnLog {
    pub owner_id: String,
    pub authorized_id: Option<String>,
    pub token_ids: Vec<String>,
    pub memo: Option<String>,
}

// #[near_event_data(standard = "nep171", version = "1.0.0", event = "nft_burn")]
// pub struct NftBurnData(Vec<NftBurnLog>);

#[cfg_attr(feature = "ser", derive(Serialize))]
#[cfg_attr(feature = "de", derive(Deserialize))]
#[cfg_attr(any(feature = "ser", feature = "de"), serde(crate = "near_sdk::serde"))]
pub struct NftTransferLog {
    pub authorized_id: Option<String>,
    pub old_owner_id: String,
    pub new_owner_id: String,
    pub token_ids: Vec<String>,
    pub memo: Option<String>,
}

#[near_event_data(standard = "nep171", version = "1.0.0", event = "nft_transfer")]
pub struct NftTransferData(pub Vec<NftTransferLog>);

#[cfg_attr(feature = "ser", derive(Serialize))]
#[cfg_attr(feature = "de", derive(Deserialize))]
#[cfg_attr(any(feature = "ser", feature = "de"), serde(crate = "near_sdk::serde"))]
pub struct NftMintLogMemo {
    pub royalty: Option<crate::store_data::Royalty>,
    pub split_owners: Option<crate::store_data::SplitOwners>,
    pub meta_id: Option<String>,
    pub meta_extra: Option<String>,
    pub minter: String,
}

// ------------------------------- Approvals -------------------------------- //
#[cfg_attr(feature = "ser", derive(Serialize))]
#[cfg_attr(feature = "de", derive(Deserialize))]
#[cfg_attr(any(feature = "ser", feature = "de"), serde(crate = "near_sdk::serde"))]
pub struct NftApproveLog {
    pub token_id: U64,
    pub approval_id: u64,
    pub account_id: String,
}

#[near_event_data(standard = "mb_store", version = "0.1.0", event = "nft_approve")]
pub struct NftApproveData(pub Vec<NftApproveLog>);

#[near_event_data(standard = "mb_store", version = "0.1.0", event = "nft_revoke")]
pub struct NftRevokeData {
    pub token_id: U64,
    pub account_id: String,
}

#[near_event_data(standard = "mb_store", version = "0.1.0", event = "nft_revoke_all")]
pub struct NftRevokeAllData {
    pub token_id: U64,
}

// -------------------------------- Payouts --------------------------------- //
use std::collections::HashMap;

// pub use market::*;
// pub use mb_store_settings::*;
#[cfg(feature = "de")]
use near_sdk::serde::Deserialize;
// #[cfg(feature = "ser")]
// use near_sdk::serde::Serialize;
// pub use nft_approvals::*;
// pub use nft_core::*;

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

// ----------------------------- Store settings ----------------------------- //
#[near_event_data(standard = "mb_store", version = "0.1.0", event = "change_setting")]
pub struct MbStoreChangeSettingData {
    pub granted_minter: Option<String>,
    pub revoked_minter: Option<String>,
    pub new_owner: Option<String>,
    pub new_icon_base64: Option<String>,
    pub new_base_uri: Option<String>,
}

impl MbStoreChangeSettingData {
    pub fn empty() -> Self {
        MbStoreChangeSettingData {
            granted_minter: None,
            revoked_minter: None,
            new_owner: None,
            new_icon_base64: None,
            new_base_uri: None,
        }
    }
}

// --------------------------------- Market --------------------------------- //
