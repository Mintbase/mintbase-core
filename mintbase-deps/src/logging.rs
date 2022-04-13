use std::convert::TryFrom;
use std::str::FromStr;

use near_sdk::serde::{
    Deserialize,
    Serialize,
};

mod market;
mod nft_approvals;
mod nft_core;
mod nft_misc;
mod nft_payouts;
pub use market::*;
pub use nft_approvals::*;
pub use nft_core::*;
pub use nft_misc::*;
pub use nft_payouts::*;

// TODO: probably unused -> deprecate?
mod nft_composition;
mod nft_loan;
mod nft_move;
pub use nft_composition::*;
pub use nft_loan::*;
pub use nft_move::*;

// ----------------------------- various types ------------------------------ //

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(untagged)]
pub enum NftEvent {
    NftCreateStore(NftStoreCreateLog),
    NftStringEvent(NftStringLog),
    NftCreate(Vec<NftMintLog>),
    NftDelete(Vec<NftBurnLog>),
    NftCreateApproval(Vec<NftApproveLog>),
    NftRevoke(NftRevokeLog),
    NftUpdate(Vec<NftTransferLog>),
    NftUpdateSplitOwner(NftSetSplitOwnerLog),
    NftUpdateLoan(NftLoanSetLog),
    NftCreateCompose(NftComposeLog),
    NftDeleteCompose(NftUncomposeLog),
    NftOnCreateCompose(NftOnComposeLog),
    NftOnDeleteCompose(NftOnUncomposeLog),
    NftOnMove(NftOnMoveLog),
    NftMoved(NftMovedLog),
    NftCreateList(Vec<NftListLog>),
    NftCreateOffer(NftOfferLog),
    NftUpdateOffer(NftUpdateOfferLog),
    NftCreateSale(NftSaleLog),
    NftUpdateMarket(NftMarketLog),
    NftUpdateIcon(NftOptionStringLog),
    NftUpdateList(NftUpdateListLog),
}

impl TryFrom<&str> for NftEvent {
    type Error = serde_json::error::Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        // ne.map_err(|x|NftEventError(x.to_string()))
        serde_json::from_str::<NftEvent>(s)
    }
}

// ------------------ general event according to standard ------------------- //

// TODO: deprecate this abomination
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NearJsonEvent {
    pub standard: String,
    pub version: String,
    pub event: String,
    pub data: String,
}

impl FromStr for NearJsonEvent {
    type Err = serde_json::error::Error;

    fn from_str(_s: &str) -> Result<Self, Self::Err> {
        todo!()
    }
}

impl From<NftEvent> for NearJsonEvent {
    fn from(ne: NftEvent) -> Self {
        let json = serde_json::to_string(&ne).unwrap();
        Self {
            standard: "nep171".to_string(),
            version: "1.0.0".to_string(),
            event: "".to_string(),
            data: json,
        }
    }
}

impl NearJsonEvent {
    pub fn near_json_event(&self) -> String {
        let json = serde_json::to_string(&self).unwrap();
        format!("EVENT_JSON: {}", &json)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NftStringLog {
    pub data: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NftOptionStringLog {
    pub data: Option<String>,
}

#[derive(Debug, Clone)]
pub struct NftEventError(pub String);

impl std::fmt::Display for NftEventError {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter,
    ) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
