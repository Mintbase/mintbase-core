use near_sdk::json_types::U64;
use near_sdk::serde::{
    Deserialize,
    Serialize,
};
use near_sdk::{
    env,
    AccountId,
};

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "event", content = "data")]
#[serde(rename_all = "snake_case")]
pub enum Nep171EventLog {
    NftMint(Vec<NftMintLog>),
    NftBurn(Vec<NftBurnLog>),
    NftTransfer(Vec<NftTransferLog>),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Nep171Event {
    pub standard: String,
    pub version: String,
    #[serde(flatten)]
    pub event_kind: Nep171EventLog,
}

impl Nep171Event {
    pub fn near_json_event(&self) -> String {
        let json = serde_json::to_string(&self).unwrap();
        format!("EVENT_JSON: {}", &json)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NftMintLog {
    pub owner_id: String,
    pub token_ids: Vec<String>,
    pub memo: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NftBurnLog {
    pub owner_id: String,
    pub authorized_id: Option<String>,
    pub token_ids: Vec<String>,
    pub memo: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NftTransferLog {
    pub authorized_id: Option<String>,
    pub old_owner_id: String,
    pub new_owner_id: String,
    pub token_ids: Vec<String>,
    pub memo: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
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
    let log = vec![NftMintLog {
        owner_id: owner.to_string(),
        token_ids,
        memo: Option::from(memo),
    }];
    let event = Nep171Event {
        standard: "nep171".to_string(),
        version: "1.0.0".to_string(),
        event_kind: Nep171EventLog::NftMint(log),
    };

    env::log_str(event.near_json_event().as_str());
}

pub fn log_nft_transfer(
    to: &AccountId,
    token_id: u64,
    memo: &Option<String>,
    old_owner: String,
) {
    let log = vec![NftTransferLog {
        authorized_id: None,
        old_owner_id: old_owner,
        new_owner_id: to.to_string(),
        token_ids: vec![token_id.to_string()],
        memo: memo.clone(),
    }];
    let event = Nep171Event {
        standard: "nep171".to_string(),
        version: "1.0.0".to_string(),
        event_kind: Nep171EventLog::NftTransfer(log),
    };
    env::log_str(event.near_json_event().as_str());
}

pub fn log_nft_batch_transfer(
    tokens: &[U64],
    accounts: &[AccountId],
    old_owners: Vec<String>,
) {
    let log = accounts
        .iter()
        .enumerate()
        .map(|(u, x)| NftTransferLog {
            authorized_id: None,
            old_owner_id: old_owners[u].clone(),
            new_owner_id: x.to_string(),
            token_ids: vec![tokens[u].0.to_string()],
            memo: None,
        })
        .collect::<Vec<_>>();
    let event = Nep171Event {
        standard: "nep171".to_string(),
        version: "1.0.0".to_string(),
        event_kind: Nep171EventLog::NftTransfer(log),
    };
    env::log_str(event.near_json_event().as_str());
}

pub fn log_nft_batch_burn(
    token_ids: &[U64],
    owner_id: String,
) {
    let token_ids = token_ids
        .iter()
        .map(|x| x.0.to_string())
        .collect::<Vec<_>>();
    let log = vec![NftBurnLog {
        owner_id,
        authorized_id: None,
        token_ids,
        memo: None,
    }];
    let event = Nep171Event {
        standard: "nep171".to_string(),
        version: "1.0.0".to_string(),
        event_kind: Nep171EventLog::NftBurn(log),
    };
    env::log_str(event.near_json_event().as_str());
}
