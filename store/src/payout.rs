use mintbase_deps::common::{
    OwnershipFractions,
    Payout,
    Royalty,
    SplitBetweenUnparsed,
    SplitOwners,
};
use mintbase_deps::constants::MAX_LEN_PAYOUT;
use mintbase_deps::logging::log_set_split_owners;
use mintbase_deps::near_sdk::json_types::{
    U128,
    U64,
};
use mintbase_deps::near_sdk::{
    self,
    env,
    near_bindgen,
    AccountId,
};
use mintbase_deps::token::Owner;
use mintbase_deps::{
    assert_storage_deposit,
    assert_token_owned_by_predecessor,
    assert_token_unloaned,
    assert_yocto_deposit,
    near_assert,
};

use crate::*;

// ---------------------- standardized payout methods ----------------------- //
#[near_bindgen]
impl MintbaseStore {
    // -------------------------- change methods ---------------------------
    /// Transfer and return payout according to [NEP-199](https://nomicon.io/Standards/Tokens/NonFungibleToken/Payout)
    #[payable]
    pub fn nft_transfer_payout(
        &mut self,
        receiver_id: AccountId,
        token_id: U64,
        approval_id: u64,
        balance: near_sdk::json_types::U128,
        max_len_payout: u32,
    ) -> Payout {
        assert_yocto_deposit!();
        let payout = self.nft_payout(token_id, balance, max_len_payout);
        self.nft_transfer(receiver_id, token_id, Some(approval_id), None);
        payout
    }

    // -------------------------- view methods -----------------------------
    /// Show payout according to [NEP-199](https://nomicon.io/Standards/Tokens/NonFungibleToken/Payout)
    pub fn nft_payout(
        &self,
        token_id: U64,
        balance: U128,
        max_len_payout: u32,
    ) -> Payout {
        let token = self.nft_token(token_id).expect("no token");
        match token.owner_id {
            Owner::Account(_) => {},
            _ => env::panic_str("token is composed"),
        }
        let payout = OwnershipFractions::new(
            &token.owner_id.to_string(),
            &self.get_token_royalty(token_id),
            &token.split_owners,
        )
        .into_payout(balance.into());
        let payout_len = payout.payout.len();
        if max_len_payout < payout_len as u32 {
            near_sdk::env::panic_str(format!("payout too long: {}", payout_len).as_str());
        }
        payout
    }
}

// -------------------- non-standardized payout methods --------------------- //
#[near_bindgen]
impl MintbaseStore {
    // -------------------------- change methods ---------------------------

    /// The `SplitOwners` of the token each receive some percentage of the _next_
    /// sale of the token. After the token is transferred, the SplitOwners field
    /// will be marked `None`, but may be set again by the next owner of the
    /// token. This method may only be called if the current `SplitOwners` field
    /// is `None`.
    ///
    /// Only the token owner may call this function.
    #[payable]
    pub fn set_split_owners(
        &mut self,
        token_ids: Vec<U64>,
        split_between: SplitBetweenUnparsed,
    ) {
        near_assert!(!token_ids.is_empty(), "Requires token IDs");
        // near_assert!(
        //     split_between.len() >= 2,
        //     "Requires at least two accounts to split between"
        // );
        assert_storage_deposit!(
            (self.storage_costs.common * split_between.len() as u128) * token_ids.len() as u128
        );
        let splits = SplitOwners::new(split_between);

        token_ids.iter().for_each(|&token_id| {
            let mut token = self.nft_token_internal(token_id.into());
            // token.assert_unloaned();
            // token.assert_owned_by_predecessor();
            assert_token_unloaned!(token);
            assert_token_owned_by_predecessor!(token);

            // TODO: Can splits not be overwritten? Why not?
            near_assert!(
                token.split_owners.is_none(),
                "Cannot overwrite split owners"
            );
            let roy_len = match token.royalty_id {
                Some(royalty_id) => self
                    .token_royalty
                    .get(&royalty_id)
                    .unwrap()
                    .1
                    .split_between
                    .len(),
                None => 0,
            };
            near_assert!(
                splits.split_between.len() + roy_len <= MAX_LEN_PAYOUT as usize,
                "Number of payout addresses may not exceed {}",
                MAX_LEN_PAYOUT
            );

            token.split_owners = Some(splits.clone());
            self.tokens.insert(&token_id.into(), &token);
        });
        log_set_split_owners(token_ids, splits);
    }

    // -------------------------- view methods -----------------------------

    /// Get the Royalty for a Token. The `Royalty` structure is not stored on the
    /// token, as this would lead to duplication of `Royalty`s across tokens.
    /// Instead, the `Royalty` is stored in a Contract `LookupMap`.
    pub fn get_token_royalty(
        &self,
        token_id: U64,
    ) -> Option<Royalty> {
        let royalty_id = self.nft_token_internal(token_id.into()).royalty_id;
        match royalty_id {
            Some(id) => self.token_royalty.get(&id).map(|(_, r)| r),
            None => None,
        }
    }

    // -------------------------- private methods --------------------------
    // -------------------------- internal methods -------------------------
}
