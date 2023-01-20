use std::collections::HashMap;

use near_sdk::json_types::U128;
use near_sdk::serde::{
    Deserialize,
    Serialize,
};
use near_sdk::AccountId;

/// Whom to pay. Generated from `OwnershipFractions`.
#[derive(Serialize, Deserialize)]
pub struct Payout {
    pub payout: HashMap<AccountId, U128>,
}
