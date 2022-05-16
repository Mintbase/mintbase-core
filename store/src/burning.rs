use mintbase_deps::logging::log_nft_batch_burn;
use mintbase_deps::near_sdk::json_types::U64;
use mintbase_deps::near_sdk::{
    self,
    env,
    near_bindgen,
    AccountId,
};
use mintbase_deps::{
    assert_token_owned_by,
    assert_token_unloaned,
    assert_yocto_deposit,
};

use crate::*;

#[near_bindgen]
impl MintbaseStore {
    // -------------------------- change methods ---------------------------

    /// The token will be permanently removed from this contract. Burn each
    /// token_id in `token_ids`.
    ///
    /// Only the tokens' owner may call this function.
    #[payable]
    pub fn nft_batch_burn(
        &mut self,
        token_ids: Vec<U64>,
    ) {
        assert_yocto_deposit!();
        assert!(!token_ids.is_empty());
        self.burn_triaged(token_ids, env::predecessor_account_id());
    }

    /// A helper to burn tokens. Necessary to satisfy the `nft_move` method,
    /// where the callback prevents the use of
    /// `env::predecessor_account_id()` to determine whether the owner is the
    /// method caller.
    pub fn burn_triaged(
        &mut self,
        token_ids: Vec<U64>,
        account_id: AccountId,
    ) {
        let mut set_owned = self.tokens_per_owner.get(&account_id).expect("none owned");

        token_ids.iter().for_each(|&token_id| {
            let token_id: u64 = token_id.into();
            let token = self.nft_token_internal(token_id);
            // token.assert_unloaned();
            // token.assert_owned_by(&account_id);
            assert_token_unloaned!(token);
            assert_token_owned_by!(token, &account_id);

            // update the counts on token metadata and royalties stored
            let metadata_id = self.nft_token_internal(token_id).metadata_id;
            let (count, metadata) = self.token_metadata.get(&metadata_id).unwrap();
            if count > 1 {
                self.token_metadata
                    .insert(&metadata_id, &(count - 1, metadata));
            } else {
                self.token_metadata.remove(&metadata_id);
            }
            if let Some(royalty_id) = self.nft_token_internal(token_id).royalty_id {
                let (count, royalty) = self.token_royalty.get(&royalty_id).unwrap();
                if count > 1 {
                    self.token_royalty
                        .insert(&royalty_id, &(count - 1, royalty));
                } else {
                    self.token_royalty.remove(&royalty_id);
                }
            }

            set_owned.remove(&token_id);
            self.tokens.remove(&token_id);
        });

        if set_owned.is_empty() {
            self.tokens_per_owner.remove(&account_id);
        } else {
            self.tokens_per_owner.insert(&account_id, &set_owned);
        }
        self.tokens_burned += token_ids.len() as u64;
        log_nft_batch_burn(&token_ids, account_id.to_string());
    }

    /// Get info about the store.
    pub fn get_info(&self) {
        let s = format!("owner: {}", self.owner_id);
        env::log_str(s.as_str());
        let s = format!("minted: {}", self.tokens_minted);
        env::log_str(s.as_str());
        let s = format!("burned: {}", self.tokens_burned);
        env::log_str(s.as_str());
        let s = format!("approved: {}", self.num_approved);
        env::log_str(s.as_str());
        let s = format!("allow_moves: {}", self.allow_moves);
        env::log_str(s.as_str());
    }

    // -------------------------- view methods -----------------------------
    // -------------------------- private methods --------------------------
    // -------------------------- internal methods -------------------------
}
