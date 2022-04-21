use near_events::{
    near_event_data,
    near_event_data_log,
};
use near_sdk::json_types::U64;
#[cfg(feature = "de")]
use near_sdk::serde::Deserialize;
#[cfg(feature = "ser")]
use near_sdk::serde::Serialize;
use near_sdk::{
    env,
    AccountId,
};

#[cfg_attr(feature = "all", derive(Clone))]
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
pub struct NftTransferData(Vec<NftTransferLog>);

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

#[allow(clippy::too_many_arguments)]
pub fn log_nft_batch_mint(
    first_token_id: u64,
    last_token_id: u64,
    minter: &str,
    owner: &str,
    royalty: &Option<crate::common::Royalty>,
    split_owners: &Option<crate::common::SplitOwners>,
    meta_ref: &Option<String>,
    meta_extra: &Option<String>,
) {
    let memo = serde_json::to_string(&NftMintLogMemo {
        royalty: royalty.clone(),
        split_owners: split_owners.clone(),
        meta_id: meta_ref.clone(),
        meta_extra: meta_extra.clone(),
        minter: minter.to_string(),
    })
    .unwrap();
    let token_ids = (first_token_id..=last_token_id)
        .map(|x| x.to_string())
        .collect::<Vec<_>>();
    let log = NftMintLog {
        owner_id: owner.to_string(),
        token_ids,
        memo: Option::from(memo),
    };

    env::log_str(log.serialize_event().as_str());
}

pub fn log_nft_transfer(
    to: &AccountId,
    token_id: u64,
    memo: &Option<String>,
    old_owner: String,
) {
    let data = NftTransferData(vec![NftTransferLog {
        authorized_id: None,
        old_owner_id: old_owner,
        new_owner_id: to.to_string(),
        token_ids: vec![token_id.to_string()],
        memo: memo.clone(),
    }]);

    env::log_str(data.serialize_event().as_str());
}

pub fn log_nft_batch_transfer(
    tokens: &[U64],
    accounts: &[AccountId],
    old_owners: Vec<String>,
) {
    let data = NftTransferData(
        accounts
            .iter()
            .enumerate()
            .map(|(u, x)| NftTransferLog {
                authorized_id: None,
                old_owner_id: old_owners[u].clone(),
                new_owner_id: x.to_string(),
                token_ids: vec![tokens[u].0.to_string()],
                memo: None,
            })
            .collect::<Vec<_>>(),
    );

    env::log_str(data.serialize_event().as_str());
}

pub fn log_nft_batch_burn(
    token_ids: &[U64],
    owner_id: String,
) {
    let token_ids = token_ids
        .iter()
        .map(|x| x.0.to_string())
        .collect::<Vec<_>>();
    let log = NftBurnLog {
        owner_id,
        authorized_id: None,
        token_ids,
        memo: None,
    };

    env::log_str(log.serialize_event().as_str());
}
