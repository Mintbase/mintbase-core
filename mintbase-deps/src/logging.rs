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
    pub royalty: Option<crate::common::Royalty>,
    pub split_owners: Option<crate::common::SplitOwners>,
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
pub struct NftListData(pub Vec<NftListLog>);

#[cfg_attr(feature = "all", derive(Clone, Debug))]
#[near_event_data(standard = "mb_market", version = "0.1.0", event = "nft_update_list")]
pub struct NftUpdateListData {
    pub list_id: String,
    pub auto_transfer: Option<bool>,
    pub price: Option<String>,
}

#[cfg_attr(feature = "all", derive(Clone, Debug))]
#[near_event_data_log(standard = "mb_market", version = "0.1.0", event = "nft_unlist")]
pub struct NftUnlistLog {
    pub list_id: String,
}

#[cfg_attr(feature = "all", derive(Clone, Debug))]
#[near_event_data(standard = "mb_market", version = "0.1.0", event = "nft_sold")]
pub struct NftSaleData {
    pub list_id: String,
    pub offer_num: u64,
    pub token_key: String,
    pub payout: HashMap<AccountId, U128>,
    // Not originally in 0.1.0, but option makes it backwards compatible with
    // serde_json
    pub mintbase_amount: Option<U128>,
}

#[cfg_attr(feature = "all", derive(Clone, Debug))]
#[cfg_attr(feature = "ser", derive(Serialize))]
#[cfg_attr(feature = "de", derive(Deserialize))]
#[cfg_attr(any(feature = "ser", feature = "de"), serde(crate = "near_sdk::serde"))]
pub struct NftMakeOfferLog {
    pub offer: crate::common::TokenOffer, // TODO: TokenOfferJson to stringify u128?
    pub list_id: String,
    pub token_key: String,
    pub offer_num: u64,
}

// FIXME: u128 is not supported -_____-
#[cfg_attr(feature = "all", derive(Clone, Debug))]
#[near_event_data(standard = "mb_market", version = "0.1.0", event = "nft_make_offer")]
pub struct NftMakeOfferData(pub Vec<NftMakeOfferLog>);

#[cfg_attr(feature = "all", derive(Clone, Debug))]
#[near_event_data(
    standard = "mb_market",
    version = "0.1.0",
    event = "nft_withdraw_offer"
)]
pub struct NftWithdrawOfferData {
    pub list_id: String,
    pub offer_num: u64,
}

#[cfg_attr(feature = "all", derive(Clone, Debug))]
#[near_event_data(standard = "mb_market", version = "0.1.0", event = "update_banlist")]
pub struct UpdateBanlistData {
    pub account_id: String,
    pub state: bool,
}

#[cfg_attr(feature = "all", derive(Clone, Debug))]
#[near_event_data(standard = "mb_market", version = "0.1.0", event = "update_allowlist")]
pub struct UpdateAllowlistData {
    pub account_id: String,
    pub state: bool,
}
