use mintbase_deps::near_sdk::json_types::U64;
use mintbase_deps::near_sdk::{
    self,
    near_bindgen,
    AccountId,
};
use mintbase_deps::store_data::TokenCompliant;

use crate::*;

// -------------------- standardized enumeration methods -------------------- //
#[near_bindgen]
impl MintbaseStore {
    /// Total number of available NFTs on this smart contract according to
    /// [NEP-181](https://nomicon.io/Standards/Tokens/NonFungibleToken/Enumeration)
    pub fn nft_total_supply(&self) -> U64 {
        (self.tokens_minted - self.tokens_burned).into()
    }

    /// List NFTs according to
    /// [NEP-181](https://nomicon.io/Standards/Tokens/NonFungibleToken/Enumeration)
    pub fn nft_tokens(
        &self,
        from_index: Option<String>, // default: "0"
        limit: Option<u64>,         // default: = self.nft_total_supply()
    ) -> Vec<TokenCompliant> {
        let from_index: u64 = from_index
            .unwrap_or_else(|| "0".to_string())
            .parse()
            .unwrap();
        let limit = limit.unwrap_or(self.tokens_minted);
        (from_index..limit)
            .into_iter()
            .flat_map(|token_id| self.nft_token_compliant_internal(token_id))
            .collect()
    }

    /// Total number of available NFTs for specified owner according to
    /// [NEP-181](https://nomicon.io/Standards/Tokens/NonFungibleToken/Enumeration)
    pub fn nft_supply_for_owner(
        &self,
        account_id: AccountId,
    ) -> U64 {
        self.tokens_per_owner
            .get(&account_id)
            .map(|v| v.len())
            .unwrap_or(0)
            .into()
    }

    /// List NFTs for specified owner according to
    /// [NEP-181](https://nomicon.io/Standards/Tokens/NonFungibleToken/Enumeration)
    pub fn nft_tokens_for_owner(
        &self,
        account_id: AccountId,
        from_index: Option<String>,
        limit: Option<usize>,
    ) -> Vec<TokenCompliant> {
        self.tokens_per_owner
            .get(&account_id)
            .expect("no tokens")
            .iter()
            .skip(
                from_index
                    .unwrap_or_else(|| "0".to_string())
                    .parse()
                    .unwrap(),
            )
            .take(limit.unwrap_or(10))
            .flat_map(|x| self.nft_token_compliant_internal(x))
            .collect::<Vec<_>>()
    }
}
