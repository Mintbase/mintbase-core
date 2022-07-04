use near_sdk::serde::{
    Deserialize,
    Serialize,
};

mod market;
mod mb_store_settings;
mod nft_approvals;
mod nft_core;
mod nft_payouts;
pub use market::*;
pub use mb_store_settings::*;
pub use nft_approvals::*;
pub use nft_core::*;
pub use nft_payouts::*;

// TODO: probably unused -> deprecate?
mod nft_composition;
mod nft_loan;
mod nft_move;
pub use nft_composition::*;
pub use nft_loan::*;
pub use nft_move::*;

// ------------------ general event according to standard ------------------- //

// TODO: deprecate this abomination
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NearJsonEvent {
    pub standard: String,
    pub version: String,
    pub event: String,
    pub data: String,
}

impl NearJsonEvent {
    pub fn near_json_event(&self) -> String {
        let json = serde_json::to_string(&self).unwrap();
        format!("EVENT_JSON: {}", &json)
    }
}
