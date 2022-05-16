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
pub struct NftLoanSetLog {
    pub account_id: Option<String>,
    pub token_id: u64,
}

pub fn log_nft_loan_set(
    token_id: u64,
    account_id: &Option<AccountId>,
) {
    let log = NftLoanSetLog {
        account_id: account_id.as_ref().map(|x| x.to_string()),
        token_id,
    };
    let event = NearJsonEvent {
        standard: "nep171".to_string(),
        version: "1.0.0".to_string(),
        event: "nft_loan_set".to_string(),
        data: serde_json::to_string(&log).unwrap(),
    };
    env::log_str(event.near_json_event().as_str());
}
