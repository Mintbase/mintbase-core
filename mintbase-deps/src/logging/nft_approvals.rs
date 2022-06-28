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

#[cfg_attr(feature = "ser", derive(Serialize))]
#[cfg_attr(feature = "de", derive(Deserialize))]
#[cfg_attr(any(feature = "ser", feature = "de"), serde(crate = "near_sdk::serde"))]
pub struct NftApproveLog {
    pub token_id: U64,
    pub approval_id: u64,
    pub account_id: String,
}

#[near_event_data(standard = "mb_store", version = "0.1.0", event = "nft_approve")]
pub struct NftApproveData(Vec<NftApproveLog>);

pub fn log_approve(
    token_id: u64,
    approval_id: u64,
    account_id: &AccountId,
) {
    let data = NftApproveData(vec![NftApproveLog {
        token_id: token_id.into(),
        approval_id,
        account_id: account_id.to_string(),
    }]);
    env::log_str(&data.serialize_event());
}

pub fn log_batch_approve(
    tokens: &[U64],
    approvals: &[U64],
    account_id: &AccountId,
) {
    let data = NftApproveData(
        approvals
            .iter()
            .zip(tokens.iter())
            .map(|(approval_id, token_id)| NftApproveLog {
                token_id: token_id.clone(),
                approval_id: approval_id.0,
                account_id: account_id.to_string(),
            })
            .collect::<Vec<_>>(),
    );
    env::log_str(&data.serialize_event());
}

// #[cfg_attr(feature = "ser", derive(Serialize))]
// #[cfg_attr(feature = "de", derive(Deserialize))]
// #[cfg_attr(any(feature = "ser", feature = "de"), serde(crate = "near_sdk::serde"))]
// pub struct NftRevokeLog {
//     pub token_id: U64,
//     pub account_id: String,
// }

#[near_event_data(standard = "mb_store", version = "0.1.0", event = "nft_revoke")]
pub struct NftRevokeData {
    pub token_id: U64,
    pub account_id: String,
}

pub fn log_revoke(
    token_id: u64,
    account_id: &AccountId,
) {
    env::log_str(
        &NftRevokeData {
            token_id: token_id.into(),
            account_id: account_id.to_string(),
        }
        .serialize_event(),
    );
}

#[near_event_data(standard = "mb_store", version = "0.1.0", event = "nft_revoke_all")]
pub struct NftRevokeAllData {
    token_id: U64,
}

pub fn log_revoke_all(token_id: u64) {
    env::log_str(
        &NftRevokeAllData {
            token_id: token_id.into(),
        }
        .serialize_event(),
    );
}
