use near_sdk::borsh::{
    self,
    BorshDeserialize,
    BorshSerialize,
};
use near_sdk::serde::{
    Deserialize,
    Serialize,
};
use near_sdk::AccountId;

use crate::common::NFTContractMetadata;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "wasm", derive(BorshDeserialize, BorshSerialize))]
pub struct StoreInitArgs {
    pub metadata: NFTContractMetadata,
    pub owner_id: AccountId,
}
