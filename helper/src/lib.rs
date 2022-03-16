use mintbase_deps::near_sdk::borsh::{
    self,
    BorshDeserialize,
    BorshSerialize,
};
use mintbase_deps::near_sdk::{
    self,
    env,
    near_bindgen,
    AccountId,
    PromiseOrValue,
};

// ----------------------------- smart contract ----------------------------- //
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct HelperWasm {
    pub count: u64,
}

/// default must be implemented for wasm compilation.
impl Default for HelperWasm {
    fn default() -> Self {
        Self { count: 0 }
    }
}

#[near_bindgen]
impl HelperWasm {
    #[init(ignore_state)]
    pub fn new() -> Self {
        Self { count: 0 }
    }

    pub fn nft_on_transfer(
        &mut self,
        sender_id: AccountId,
        previous_owner_id: AccountId,
        token_id: String,
        msg: String,
    ) -> PromiseOrValue<bool> {
        env::log_str(
            format!(
                "in nft_on_transfer; sender_id={}, previous_owner_id={}, token_id={}, msg={}",
                &sender_id, &previous_owner_id, &token_id, msg
            )
            .as_str(),
        );
        match msg.as_str() {
            "true" => PromiseOrValue::Value(true),
            "false" => PromiseOrValue::Value(false),
            _ => env::panic_str("unsupported msg"),
        }
    }
}
