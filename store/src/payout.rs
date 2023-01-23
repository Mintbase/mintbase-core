use std::collections::HashMap;

use mintbase_deps::common::{
    Payout,
    Royalty,
    SplitBetweenUnparsed,
    SplitOwners,
};
use mintbase_deps::constants::MAX_LEN_PAYOUT;
use mintbase_deps::near_sdk::json_types::{
    U128,
    U64,
};
use mintbase_deps::near_sdk::{
    self,
    env,
    near_bindgen,
    AccountId,
    Balance,
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
        approval_id: Option<u64>,
        memo: Option<String>,
        balance: near_sdk::json_types::U128,
        max_len_payout: Option<u32>,
    ) -> Payout {
        assert_yocto_deposit!();
        let payout = self.nft_payout(token_id, balance, max_len_payout);
        self.nft_transfer(receiver_id, token_id, approval_id, memo);
        payout
    }

    // -------------------------- view methods -----------------------------
    /// Show payout according to [NEP-199](https://nomicon.io/Standards/Tokens/NonFungibleToken/Payout)
    pub fn nft_payout(
        &self,
        token_id: U64,
        balance: U128,
        max_len_payout: Option<u32>,
    ) -> Payout {
        let token = self.nft_token(token_id).expect("no token");
        let owner_id = match token.owner_id {
            Owner::Account(id) => id,
            _ => env::panic_str("token is composed"),
        };

        OwnershipFractions::new(
            owner_id,
            self.get_token_royalty(token_id),
            token.split_owners,
        )
        .into_payout(balance.into(), max_len_payout)
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

/// This struct is a helper used for computing payouts from stored
/// payouts/splits fractions to actual balances, given a token total price and
/// maybe a max length of the payouts.
struct OwnershipFractions {
    fractions: HashMap<AccountId, u32>,
    remaining: u32,
    royalty_percentage: u32,
    split_percentage: u32,
}

impl OwnershipFractions {
    fn new(
        owner_id: AccountId,
        royalty: Option<Royalty>,
        split_owners: Option<SplitOwners>,
    ) -> Self {
        let royalty_percentage = royalty
            .as_ref()
            .map(|r| r.percentage.numerator)
            .unwrap_or(0);
        let split_percentage = 10_000 - royalty_percentage;
        let mut fractions = OwnershipFractions {
            fractions: HashMap::new(),
            remaining: 10_000,
            royalty_percentage,
            split_percentage,
        };

        if let Some(Royalty {
            mut split_between,
            percentage: _,
        }) = royalty
        {
            for (owner_id, percentage) in split_between.drain() {
                fractions.add_royalty_owner(owner_id, percentage.numerator);
            }
        }

        if let Some(SplitOwners { mut split_between }) = split_owners {
            for (owner_id, percentage) in split_between.drain() {
                fractions.add_split_owner(owner_id, percentage.numerator);
            }
        } else {
            fractions.fill_owner(owner_id);
        }

        fractions
    }

    fn add_royalty_owner(
        &mut self,
        owner_id: AccountId,
        percentage: u32,
    ) {
        let p = percentage * self.royalty_percentage / 10_000;
        // No need to check existence because royalty owners are inserted first
        self.fractions.insert(owner_id, p);
        self.remaining -= p;
    }

    fn add_split_owner(
        &mut self,
        owner_id: AccountId,
        percentage: u32,
    ) {
        let p = percentage * self.split_percentage / 10_000;
        let entry = self.fractions.entry(owner_id).or_insert(0);
        *entry += p;
        self.remaining -= p;
    }

    fn fill_owner(
        &mut self,
        owner_id: AccountId,
    ) {
        let entry = self.fractions.entry(owner_id).or_insert(0);
        *entry += self.remaining;
        self.remaining = 0;
    }

    fn into_payout(
        mut self,
        balance: Balance,
        max_len: Option<u32>,
    ) -> Payout {
        let balances_iter = self
            .fractions
            .drain()
            .map(|(owner_id, percentage)| (owner_id, percentage as Balance * balance / 10_000));
        let payout = match max_len {
            None => balances_iter
                .map(|(owner_id, balance)| (owner_id, balance.into()))
                .collect::<HashMap<AccountId, U128>>(),
            Some(max_len) => {
                let mut v = balances_iter.collect::<Vec<(AccountId, Balance)>>();
                v.sort_by(|(_, balance_a), (_, balance_b)| balance_b.cmp(balance_a));
                v.into_iter()
                    .take(max_len as usize)
                    .map(|(owner_id, balance)| (owner_id, balance.into()))
                    .collect::<HashMap<AccountId, U128>>()
            },
        };

        Payout { payout }
    }
}

fn log_set_split_owners(
    token_ids: Vec<U64>,
    mut split_owners: mintbase_deps::common::SplitOwners,
) {
    env::log_str(
        &mintbase_deps::logging::NftSetSplitOwnerData {
            token_ids,
            split_owners: split_owners
                .split_between
                .drain()
                .map(|(acc, fraction)| (acc, fraction.numerator as u16))
                .collect(),
        }
        .serialize_event(),
    );
}
