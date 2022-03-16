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

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "wasm", derive(BorshDeserialize, BorshSerialize))]
pub struct Loan {
    pub holder: AccountId,
    pub loan_contract: AccountId,
}

impl Loan {
    pub fn new(
        holder: AccountId,
        loan_contract: AccountId,
    ) -> Self {
        Self {
            holder,
            loan_contract,
        }
    }
}
