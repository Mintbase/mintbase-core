use std::fmt;

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

use crate::utils::split_colon;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "wasm", derive(BorshDeserialize, BorshSerialize))]
pub struct TokenKey {
    pub token_id: u64,
    pub account_id: String,
}

impl TokenKey {
    pub fn new(
        n: u64,
        account_id: AccountId,
    ) -> Self {
        Self {
            token_id: n,
            account_id: account_id.into(),
        }
    }

    pub fn split(self) -> (u64, String) {
        (self.token_id, self.account_id)
    }
}

impl fmt::Display for TokenKey {
    fn fmt(
        &self,
        f: &mut fmt::Formatter,
    ) -> fmt::Result {
        write!(f, "{}:{}", self.token_id, self.account_id)
    }
}

impl From<String> for TokenKey {
    fn from(s: String) -> Self {
        let (id, account_id) = split_colon(&s);
        Self {
            token_id: id.parse::<u64>().unwrap(),
            account_id: account_id.to_string(),
        }
    }
}
