use near_sdk::json_types::U128;
use near_sdk::serde::{
    Deserialize,
    Serialize,
};

/// ref: https://github.com/near-apps/nft-market/blob/main/contracts/market-simple/src/lib.rs#L54
#[derive(Serialize, Deserialize)]
pub struct SaleArgs {
    pub price: U128,
    pub autotransfer: bool,
}
