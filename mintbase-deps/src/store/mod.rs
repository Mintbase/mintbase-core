// misc
use std::collections::HashMap;
use std::convert::TryFrom;

use near_sdk::borsh::{
    self,
    BorshDeserialize,
    BorshSerialize,
};
use near_sdk::collections::{
    LookupMap,
    UnorderedSet,
};
use near_sdk::json_types::{
    U128,
    U64,
};
use near_sdk::{
    self,
    assert_one_yocto,
    env,
    ext_contract,
    near_bindgen,
    AccountId,
    Balance,
    Gas,
    Promise,
    PromiseResult,
    StorageUsage,
};

// contract interface modules
use crate::common::{
    NFTContractMetadata,
    NewSplitOwner,
    NonFungibleContractMetadata,
    Owner,
    OwnershipFractions,
    Payout,
    Royalty,
    RoyaltyArgs,
    SafeFraction,
    SplitBetweenUnparsed,
    SplitOwners,
    StorageCosts,
    Token,
    TokenCompliant,
    TokenMetadata,
    TokenMetadataCompliant,
};
use crate::consts::{
    GAS_NFT_BATCH_APPROVE,
    GAS_NFT_TRANSFER_CALL,
    MAX_LEN_PAYOUT,
    MINIMUM_CUSHION,
    NO_DEPOSIT,
};
use crate::interfaces::{
    ext_on_approve,
    ext_on_transfer,
};
// logging functions
use crate::logging::{
    log_approve,
    log_batch_approve,
    log_grant_minter,
    log_nft_batch_burn,
    log_nft_batch_mint,
    log_nft_batch_transfer,
    log_nft_transfer,
    log_revoke,
    log_revoke_all,
    log_revoke_minter,
    log_set_base_uri,
    log_set_icon_base64,
    log_set_split_owners,
    log_transfer_store,
};
use crate::utils::ntot;

// ------------------------------- constants -------------------------------- //
const GAS_PASS_TO_APPROVED: Gas = ntot(Gas(25));

// ----------------------------- smart contract ----------------------------- //

// TODO: shouldn't this be PanicOnDefault?
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct MintbaseStore {
    /// Accounts that are allowed to mint tokens on this Store.
    pub minters: UnorderedSet<AccountId>,
    /// Initial deployment data of this Store.
    pub metadata: NFTContractMetadata,
    /// If a Minter mints more than one token at a time, all tokens will
    /// share the same `TokenMetadata`. It's more storage-efficient to store
    /// that `TokenMetadata` once, rather than to copy the data on each
    /// Token. The key is generated from `tokens_minted`. The map keeps count
    /// of how many copies of this token remain, so that the element may be
    /// dropped when the number reaches zero (ie, when tokens are burnt).
    pub token_metadata: LookupMap<u64, (u16, TokenMetadata)>,
    /// If a Minter mints more than one token at a time, all tokens will
    /// share the same `Royalty`. It's more storage-efficient to store that
    /// `Royalty` once, rather than to copy the data on each Token. The key
    /// is generated from `tokens_minted`. The map keeps count of how many
    /// copies of this token remain, so that the element may be dropped when
    /// the number reaches zero (ie, when tokens are burnt).
    pub token_royalty: LookupMap<u64, (u16, Royalty)>,
    /// Tokens this Store has minted, excluding those that have been burned.
    pub tokens: LookupMap<u64, Token>,
    /// A mapping from each user to the tokens owned by that user. The owner
    /// of the token is also stored on the token itself.
    pub tokens_per_owner: LookupMap<AccountId, UnorderedSet<u64>>,
    /// A map from a token_id of a token on THIS contract to a set of tokens,
    /// that may be on ANY contract. If the owned-token is on this contract,
    /// the id will have format "<u64>". If the token is on another contract,
    /// the token will have format "<u64>:account_id"
    pub composeables: LookupMap<String, UnorderedSet<String>>,
    /// The number of tokens this `Store` has minted. Used to generate
    /// `TokenId`s.
    pub tokens_minted: u64,
    /// The number of tokens this `Store` has burned.
    pub tokens_burned: u64,
    /// The number of tokens approved (listed) by this `Store`. Used to index
    /// listings and approvals. List ID format: `list_nonce:token_key`
    pub num_approved: u64,
    /// The owner of the Contract.
    pub owner_id: AccountId,
    /// The Near-denominated price-per-byte of storage, and associated
    /// contract storage costs. As of April 2021, the price per bytes is set
    /// to 10^19, but this may change in the future, thus this
    /// future-proofing field.
    pub storage_costs: StorageCosts,
    /// If false, disallow users to call `nft_move`.
    pub allow_moves: bool,
}

impl Default for MintbaseStore {
    fn default() -> Self {
        env::panic_str("no default")
    }
}

impl NonFungibleContractMetadata for MintbaseStore {
    fn nft_metadata(&self) -> &NFTContractMetadata {
        &self.metadata
    }
}

#[near_bindgen]
impl MintbaseStore {
    pub fn nft_tokens(
        &self,
        from_index: Option<String>, // default: "0"
        limit: Option<u64>,         // default: = self.nft_total_supply()
    ) -> Vec<TokenCompliant> {
        let from_index: u64 = from_index
            .unwrap_or_else(|| "0".to_string())
            .parse()
            .unwrap();
        let limit = limit.unwrap_or(self.nft_total_supply().0);
        (from_index..limit)
            .into_iter()
            .map(|token_id| self.nft_token_compliant_internal(token_id))
            .collect()
    }

    #[payable]
    pub fn nft_batch_approve(
        &mut self,
        token_ids: Vec<U64>,
        account_id: AccountId,
        msg: Option<String>,
    ) -> Option<Promise> {
        let tlen = token_ids.len() as u128;
        assert!(tlen > 0);
        assert!(tlen <= 70);
        let store_approval_storage = self.storage_costs.common * tlen;
        // Note: This method only guarantees that the store-storage is covered.
        // The financial contract may still reject.
        assert!(
            env::attached_deposit() > store_approval_storage,
            "deposit less than: {}",
            store_approval_storage
        );
        let approval_ids: Vec<U64> = token_ids
            .iter()
            // validates owner and loaned
            .map(|&token_id| self.approve_internal(token_id.into(), &account_id).into())
            .collect();
        log_batch_approve(&token_ids, &approval_ids, &account_id);

        if let Some(msg) = msg {
            ext_on_approve::nft_on_batch_approve(
                token_ids,
                approval_ids,
                env::predecessor_account_id(),
                msg,
                account_id,
                env::attached_deposit() - store_approval_storage,
                GAS_NFT_BATCH_APPROVE,
            )
            .into()
        } else {
            None
        }
    }

    #[payable]
    pub fn nft_batch_transfer(
        &mut self,
        token_ids: Vec<(U64, AccountId)>,
    ) {
        near_sdk::assert_one_yocto();
        assert!(!token_ids.is_empty());
        let pred = env::predecessor_account_id();
        let mut set_owned = self.tokens_per_owner.get(&pred).expect("none owned");
        let (tokens, accounts, old_owners) = token_ids
            .into_iter()
            .map(|(token_id, account_id)| {
                let token_idu64 = token_id.into();
                let mut token = self.nft_token_internal(token_idu64);
                let old_owner = token.owner_id.to_string();
                assert!(!token.is_loaned());
                assert!(token.is_pred_owner());
                assert_ne!(account_id.to_string(), token.owner_id.to_string()); // can't transfer to self
                self.transfer_internal(&mut token, account_id.clone(), false);
                set_owned.remove(&token_idu64);
                (token_id, account_id, old_owner)
            })
            .fold((vec![], vec![], vec![]), |mut acc, (tid, aid, oid)| {
                acc.0.push(tid);
                acc.1.push(aid);
                acc.2.push(oid);
                acc
            });
        self.tokens_per_owner.insert(&pred, &set_owned);
        log_nft_batch_transfer(&tokens, &accounts, old_owners);
    }

    /// Get the holder of the token. The token may be owned by:
    /// - a normal account: return that account.
    /// - a lent out account : in that case, return the loan holder.
    /// - a token on this contract: recursively search for the root token and
    /// return its owner
    /// - a token on another contract. Return: "PARENT_TOKEN_ID:CONTRACT_ID".
    pub fn nft_holder(
        &self,
        token_id: U64,
    ) -> String {
        let token = self.nft_token_internal(token_id.into());
        match token.get_owner_or_loaner() {
            Owner::Account(owner) => owner.to_string(),
            Owner::TokenId(id) => self.nft_holder(id.into()),
            Owner::CrossKey(key) => (key.to_string()),
            Owner::Lock(_) => (env::panic_str("token locked")),
        }
    }

    #[payable]
    pub fn nft_approve(
        &mut self,
        token_id: U64,
        account_id: AccountId,
        msg: Option<String>,
    ) -> Option<Promise> {
        // Note: This method only guarantees that the store-storage is covered. The
        // market may still reject.
        assert!(env::attached_deposit() > self.storage_costs.common);
        let token_idu64 = token_id.into();
        // validates owner and loaned
        let approval_id = self.approve_internal(token_idu64, &account_id);
        log_approve(token_idu64, approval_id, &account_id);

        if let Some(msg) = msg {
            ext_on_approve::nft_on_approve(
                token_id,
                env::predecessor_account_id(),
                approval_id,
                msg,
                account_id,
                0,
                GAS_PASS_TO_APPROVED,
            )
            .into()
        } else {
            None
        }
    }

    #[payable]
    pub fn nft_revoke(
        &mut self,
        token_id: U64,
        account_id: AccountId,
    ) {
        let token_idu64 = token_id.into();
        let mut token = self.nft_token_internal(token_idu64);
        assert!(!token.is_loaned());
        assert!(token.is_pred_owner());
        assert_one_yocto();

        if token.approvals.remove(&account_id).is_some() {
            self.tokens.insert(&token_idu64, &token);
            log_revoke(token_idu64, &account_id);
        }
    }

    #[payable]
    pub fn nft_revoke_all(
        &mut self,
        token_id: U64,
    ) {
        let token_idu64 = token_id.into();
        let mut token = self.nft_token_internal(token_idu64);
        assert!(!token.is_loaned());
        assert!(token.is_pred_owner());
        assert_one_yocto();

        if !token.approvals.is_empty() {
            token.approvals.clear();
            self.tokens.insert(&token_idu64, &token);
            log_revoke_all(token_idu64);
        }
    }

    pub fn nft_is_approved(
        &self,
        token_id: U64,
        approved_account_id: AccountId,
        approval_id: Option<u64>,
    ) -> bool {
        self.nft_is_approved_internal(
            &self.nft_token_internal(token_id.into()),
            approved_account_id,
            approval_id,
        )
    }

    /// Create a new `Store`. `new` validates the `store_description`.
    ///
    /// The `Store` is initialized with the owner as a `minter`.
    #[init]
    pub fn new(
        metadata: NFTContractMetadata,
        owner_id: AccountId,
    ) -> Self {
        assert!(!env::state_exists(), "Already, initialized");
        let mut minters = UnorderedSet::new(b"a".to_vec());
        minters.insert(&owner_id);

        Self {
            minters,
            metadata,
            token_metadata: LookupMap::new(b"b".to_vec()),
            token_royalty: LookupMap::new(b"c".to_vec()),
            tokens: LookupMap::new(b"d".to_vec()),
            tokens_per_owner: LookupMap::new(b"e".to_vec()),
            composeables: LookupMap::new(b"f".to_vec()),
            tokens_minted: 0,
            tokens_burned: 0,
            num_approved: 0,
            owner_id,
            storage_costs: StorageCosts::new(10_000_000_000_000_000_000), // 10^19
            allow_moves: true,
        }
    }

    #[private]
    pub fn nft_resolve_transfer(
        &mut self,
        owner_id: AccountId,
        receiver_id: AccountId,
        token_id: String,
        // NOTE: might borsh::maybestd::collections::HashMap be more appropriate?
        approved_account_ids: Option<HashMap<AccountId, u64>>,
    ) -> bool {
        let l = format!(
            "owner_id={} receiver_id={} token_id={} approved_ids={:?} pred={}",
            owner_id,
            receiver_id,
            token_id,
            approved_account_ids,
            env::predecessor_account_id()
        );
        env::log_str(l.as_str());
        let token_id_u64 = token_id.parse::<u64>().unwrap();
        let mut token = self.nft_token_internal(token_id_u64);
        self.unlock_token(&mut token);
        assert_eq!(env::promise_results_count(), 1);
        // Get whether token should be returned
        let must_revert = match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(value) => {
                if let Ok(yes_or_no) = near_sdk::serde_json::from_slice::<bool>(&value) {
                    yes_or_no
                } else {
                    true
                }
            },
            PromiseResult::Failed => true,
        };
        if !must_revert {
            true
        } else {
            self.transfer_internal(&mut token, receiver_id.clone(), true);
            log_nft_transfer(&receiver_id, token_id_u64, &None, owner_id.to_string());
            false
        }
    }

    /// A non-indexed implementation. `from_index` and `limit are removed, so as
    /// to support the:
    ///
    /// `tokens_per_owner: LookupMap<AccountId, UnorderedSet<TokenId>>`
    ///
    /// type. They may be used in an implementation if the type is instead:
    ///
    /// `tokens_per_owner: LookupMap<AccountId, Vector<TokenId>>`
    pub fn nft_tokens_for_owner_set(
        &self,
        account_id: AccountId,
    ) -> Vec<u64> {
        self.tokens_per_owner
            .get(&account_id)
            .expect("no tokens")
            .iter()
            .collect()
    }

    pub fn nft_total_supply(&self) -> U64 {
        self.tokens_minted.into()
    }

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
            .map(|x| self.nft_token_compliant_internal(x))
            .collect::<Vec<_>>()
    }

    /// Get the on-contract metadata for a Token. Note that on-contract metadata
    /// is only a small subset of the metadata stored at the `token_uri`, which
    /// can be retrieved by calling `get_token_uri`. The metadata structure is not
    /// stored on the token, as this would lead to duplication of Metadata across
    /// tokens. Instead, the Metadata is stored in a Contract `LookupMap`.
    pub fn nft_token_metadata(
        &self,
        token_id: U64,
    ) -> TokenMetadata {
        self.token_metadata
            .get(&self.nft_token_internal(token_id.into()).metadata_id)
            .expect("bad metadata_id")
            .1
    }

    /// Get the number of unburned copies of the token in existance.
    pub fn get_token_remaining_copies(
        &self,
        token_id: U64,
    ) -> u16 {
        self.token_metadata
            .get(&self.nft_token_internal(token_id.into()).metadata_id)
            .expect("bad metadata_id")
            .0
    }

    /// The core `Store` function. `mint_token` mints `num_to_mint` copies of
    /// a token.
    ///
    /// Restrictions:
    /// - Only minters may call this function.
    /// - `owner_id` must be a valid Near address.
    /// - Because of logging limits, this method may mint at most 99 tokens per call.
    /// - 1.0 >= `royalty_f` >= 0.0. `royalty_f` is ignored if `royalty` is `None`.
    /// - If a `royalty` is provided, percentages **must** be non-negative and add to one.
    /// - The maximum length of the royalty mapping is 50.
    ///
    /// This method is the most significant increase of storage costs on this
    /// contract. Minters are expected to manage their own storage costs.
    #[payable]
    pub fn nft_batch_mint(
        &mut self,
        owner_id: AccountId,
        metadata: TokenMetadata,
        num_to_mint: u64,
        royalty_args: Option<RoyaltyArgs>,
        split_owners: Option<SplitBetweenUnparsed>,
    ) {
        assert!(num_to_mint > 0);
        assert!(num_to_mint <= 125); // upper gas limit
        assert!(env::attached_deposit() >= 1);
        let minter_id = env::predecessor_account_id();
        assert!(
            self.minters.contains(&minter_id),
            "{} not a minter",
            minter_id.as_ref()
        );

        // Calculating storage consuption upfront saves gas if the transaction
        // were to fail later.
        let covered_storage = env::account_balance()
            - (env::storage_usage() as u128 * self.storage_costs.storage_price_per_byte);
        let (metadata, md_size) = TokenMetadata::from_with_size(metadata, num_to_mint);
        let roy_len = royalty_args
            .as_ref()
            .map(|pre_roy| {
                let len = pre_roy.split_between.len();
                len as u32
            })
            .unwrap_or(0);
        let split_len = split_owners
            .as_ref()
            .map(|pre_split| {
                let len = pre_split.len();
                len as u32
            })
            // if there is no split map, there still is an owner, thus default to 1
            .unwrap_or(1);
        assert!(roy_len + split_len <= MAX_LEN_PAYOUT);
        let expected_storage_consumption: Balance =
            self.storage_cost_to_mint(num_to_mint, md_size, roy_len, split_len);
        assert!(
            covered_storage >= expected_storage_consumption,
            "covered: {}; need: {}",
            covered_storage,
            expected_storage_consumption
        );

        let checked_royalty = royalty_args.map(Royalty::new);
        let checked_split = split_owners.map(SplitOwners::new);

        let mut owned_set = self.get_or_make_new_owner_set(&owner_id);

        // Lookup Id is used by the token to lookup Royalty and Metadata fields on
        // the contract (to avoid unnecessary duplication)
        let lookup_id: u64 = self.tokens_minted;
        let royalty_id = checked_royalty.clone().map(|royalty| {
            self.token_royalty
                .insert(&lookup_id, &(num_to_mint as u16, royalty));
            lookup_id
        });

        let meta_ref = metadata.reference.as_ref().map(|s| s.to_string());
        let meta_extra = metadata.extra.as_ref().map(|s| s.to_string());
        self.token_metadata
            .insert(&lookup_id, &(num_to_mint as u16, metadata));

        // Mint em up hot n fresh with a side of vegan bacon
        (0..num_to_mint).for_each(|i| {
            let token_id = self.tokens_minted + i;
            let token = Token::new(
                owner_id.clone(),
                token_id,
                lookup_id,
                royalty_id,
                checked_split.clone(),
                minter_id.clone(),
            );
            owned_set.insert(&token_id);
            self.tokens.insert(&token_id, &token);
        });
        self.tokens_minted += num_to_mint;
        self.tokens_per_owner.insert(&owner_id, &owned_set);

        let minted = self.tokens_minted;
        log_nft_batch_mint(
            minted - num_to_mint,
            minted - 1,
            minter_id.as_ref(),
            owner_id.as_ref(),
            &checked_royalty,
            &checked_split,
            &meta_ref,
            &meta_extra,
        );
    }

    /// The token will be permanently removed from this contract. Burn each
    /// token_id in `token_ids`.
    ///
    /// Only the tokens' owner may call this function.
    #[payable]
    pub fn nft_batch_burn(
        &mut self,
        token_ids: Vec<U64>,
    ) {
        near_sdk::assert_one_yocto();
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
            assert!(!token.is_loaned());
            assert_eq!(token.owner_id.to_string(), account_id.to_string());

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

    /// Check if `account_id` is a minter.
    pub fn check_is_minter(
        &self,
        account_id: AccountId,
    ) -> bool {
        self.minters.contains(&account_id)
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

    /// The Token URI is generated to index the token on whatever distributed
    /// storage platform this `Store` uses. Mintbase publishes token data on
    /// Arweave. `Store` owners may opt to use their own storage platform.
    pub fn nft_token_uri(
        &self,
        token_id: U64,
    ) -> String {
        let base = &self.metadata.base_uri.as_ref().expect("no base_uri");
        let metadata_reference = self
            .nft_token_metadata(token_id)
            .reference
            .expect("no reference");
        format!("{}/{}", base, metadata_reference)
    }

    /// Get the `token_key` for `token_id`. The `token_key` is the
    /// combination of the token's `token_id` (unique within this `Store`),
    /// and the `Store` address (unique across all contracts). The String is
    /// unique across `Store`s. The String is used by other Mintbase
    /// contracts as a permanent unique identifier for tokens.
    pub fn nft_token_key(
        &self,
        token_id: U64,
    ) -> String {
        let id: u64 = token_id.into();
        format!("{}:{}", id, env::current_account_id())
    }

    /// Owner of this `Store` may call to withdraw Near deposited onto
    /// contract for storage. Contract storage deposit must maintain a
    /// cushion of at least 50kB (0.5 Near) beyond that necessary for storage
    /// usage.
    ///
    /// Only the store owner may call this function.
    #[payable]
    pub fn withdraw_excess_storage_deposits(&mut self) {
        self.assert_store_owner();
        let unused_deposit: u128 = env::account_balance()
            - env::storage_usage() as u128 * self.storage_costs.storage_price_per_byte;
        if unused_deposit > MINIMUM_CUSHION {
            near_sdk::Promise::new(self.owner_id.clone())
                .transfer(unused_deposit - MINIMUM_CUSHION);
        } else {
            let s = format!(
                "Nothing withdrawn. Unused deposit is less than 0.5N: {}",
                unused_deposit
            );
            env::log_str(s.as_str());
        }
    }

    /// If allow_moves is false, disallow token owners from calling
    /// `nft_move` on this contract, AND on other contracts targetting this
    /// contract. `nft_move` allows the user to burn a token they own on one
    /// contract, and re-mint it on another contract.
    #[payable]
    pub fn set_allow_moves(
        &mut self,
        state: bool,
    ) {
        self.assert_store_owner();
        self.allow_moves = state;
    }

    /// The Near Storage price per byte has changed in the past, and may
    /// change in the future. This method may never be used.
    ///
    /// Only the store owner may call this function.
    #[payable]
    pub fn set_storage_price_per_byte(
        &mut self,
        new_price: U128,
    ) {
        self.assert_store_owner();
        self.storage_costs = StorageCosts::new(new_price.into())
    }

    /// Modify the minting privileges of `account_id`. Minters are able to
    /// mint tokens on this `Store`.
    ///
    /// Only the store owner may call this function.
    ///
    /// This method increases storage costs of the contract.
    #[payable]
    pub fn grant_minter(
        &mut self,
        account_id: AccountId,
    ) {
        self.assert_store_owner();
        let account_id: AccountId = account_id;
        // does nothing if account_id is already a minter
        if self.minters.insert(&account_id) {
            log_grant_minter(&account_id);
        }
    }

    /// Modify the minting privileges of `account_id`. Minters are able to
    /// mint tokens on this `Store`. The current `Store` owner cannot revoke
    /// themselves.
    ///
    /// Only the store owner may call this function.
    #[payable]
    pub fn revoke_minter(
        &mut self,
        account_id: AccountId,
    ) {
        self.assert_store_owner();
        assert_ne!(account_id, self.owner_id, "can't revoke owner");
        if !self.minters.remove(&account_id) {
            env::panic_str("not a minter")
        } else {
            log_revoke_minter(&account_id);
        }
    }

    pub fn list_minters(&self) -> Vec<AccountId> {
        self.minters.iter().collect()
    }

    /// Transfer ownership of `Store` to a new owner. Setting
    /// `keep_old_minters=true` allows all existing minters (including the
    /// prior owner) to keep their minter status.
    ///
    /// Only the store owner may call this function.
    #[payable]
    pub fn transfer_store_ownership(
        &mut self,
        new_owner: AccountId,
        keep_old_minters: bool,
    ) {
        self.assert_store_owner();
        let new_owner = new_owner;
        assert_ne!(new_owner, self.owner_id, "can't can't transfer to self");
        if !keep_old_minters {
            for minter in self.minters.iter() {
                log_revoke_minter(&minter);
            }
            self.minters.clear();
        }
        log_grant_minter(&new_owner);
        // add the new_owner to the minter set (insert does nothing if they already are a minter).
        self.minters.insert(&new_owner);
        log_transfer_store(&new_owner);
        self.owner_id = new_owner;
    }

    /// `icon_base64` is best understood as the `Store` logo/icon.
    ///
    /// Only the store owner may call this function.
    #[payable]
    pub fn set_icon_base64(
        &mut self,
        icon: Option<String>,
    ) {
        self.assert_store_owner();
        assert!(icon.as_ref().map(|b| b.len() <= 100).unwrap_or(true));
        log_set_icon_base64(&icon);
        self.metadata.icon = icon;
    }

    /// The `base_uri` for the `Store` is the identifier used to look up the
    /// `Store` on Arweave. Changing the `base_uri` requires the `Store`
    /// owner to be responsible for making sure their `Store` location is
    /// maintained by their preferred storage provider.
    ///
    /// Only the `Store` owner may call this function.
    #[payable]
    pub fn set_base_uri(
        &mut self,
        base_uri: String,
    ) {
        self.assert_store_owner();
        assert!(base_uri.len() <= 100);
        log_set_base_uri(&base_uri);
        self.metadata.base_uri = Some(base_uri);
    }

    /// Validate the caller of this method matches the owner of this `Store`.
    fn assert_store_owner(&self) {
        assert_one_yocto();
        assert_eq!(
            self.owner_id,
            env::predecessor_account_id(),
            "caller not the owner"
        );
    }

    /// Contract metadata and methods in the API may be updated. All other
    /// elements of the state should be copied over. This method may only be
    /// called by the holder of the Store public key, in this case the
    /// Factory.
    #[private]
    #[init(ignore_state)]
    pub fn migrate(metadata: NFTContractMetadata) -> Self {
        let old = env::state_read().expect("ohno ohno state");
        Self { metadata, ..old }
    }

    /// Financial contracts may query the NFT contract for the address(es) to pay
    /// out.
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

    #[payable]
    pub fn nft_transfer_payout(
        &mut self,
        receiver_id: AccountId,
        token_id: U64,
        approval_id: u64,
        balance: near_sdk::json_types::U128,
        max_len_payout: u32,
    ) -> Payout {
        assert_one_yocto();
        let payout = self.nft_payout(token_id, balance, max_len_payout);
        self.nft_transfer(receiver_id, token_id, Some(approval_id), None);
        payout
    }

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
        assert!(!token_ids.is_empty());
        assert!(split_between.len() >= 2, "split len must be >= 2");
        let storage_cost =
            (self.storage_costs.common * split_between.len() as u128) * token_ids.len() as u128;
        assert!(
            env::attached_deposit() >= storage_cost,
            "insuf. deposit. Need: {}",
            storage_cost
        );
        let splits = SplitOwners::new(split_between);

        token_ids.iter().for_each(|&token_id| {
            let mut token = self.nft_token_internal(token_id.into());
            assert!(!token.is_loaned());
            assert!(token.is_pred_owner());
            assert!(token.split_owners.is_none());
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
            assert!(splits.split_between.len() + roy_len <= MAX_LEN_PAYOUT as usize);

            token.split_owners = Some(splits.clone());
            self.tokens.insert(&token_id.into(), &token);
        });
        log_set_split_owners(&token_ids, &splits);
    }

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

    /// Called from nft_approve and nft_batch_approve.
    fn approve_internal(
        &mut self,
        token_idu64: u64,
        account_id: &AccountId,
    ) -> u64 {
        let mut token = self.nft_token_internal(token_idu64);
        assert!(!token.is_loaned());
        assert!(token.is_pred_owner());
        let approval_id = self.num_approved;
        self.num_approved += 1;
        token.approvals.insert(account_id.clone(), approval_id);
        self.tokens.insert(&token_idu64, &token);
        approval_id
    }

    fn nft_token_internal(
        &self,
        token_id: u64,
    ) -> Token {
        self.tokens
            .get(&token_id)
            .unwrap_or_else(|| panic!("token: {} doesn't exist", token_id))
    }

    fn nft_token_compliant_internal(
        &self,
        token_id: u64,
    ) -> TokenCompliant {
        self.tokens
            .get(&token_id)
            .map(|x| {
                let metadata = self.nft_token_metadata(U64(x.id));
                let royalty = self.get_token_royalty(U64(x.id));
                let metadata = TokenMetadataCompliant {
                    title: metadata.title,
                    description: metadata.description,
                    media: metadata.media,
                    media_hash: metadata.media_hash,
                    copies: metadata.copies,
                    issued_at: None,
                    expires_at: metadata.expires_at,
                    starts_at: metadata.starts_at,
                    updated_at: None,
                    extra: metadata.extra,
                    reference: metadata.reference,
                    reference_hash: metadata.reference_hash,
                };
                TokenCompliant {
                    id: x.id,
                    owner_id: x.owner_id,
                    approvals: x.approvals,
                    metadata,
                    royalty,
                    split_owners: x.split_owners,
                    minter: x.minter,
                    loan: x.loan,
                    composeable_stats: x.composeable_stats,
                    origin_key: x.origin_key,
                }
            })
            .unwrap_or_else(|| panic!("token: {} doesn't exist", token_id))
    }

    /// Set the owner of `token` to `to` and clear the approvals on the
    /// token. Update the `tokens_per_owner` sets. `remove_prior` is an
    /// optimization on batch removal, in particular useful for batch sending
    /// of tokens.
    ///
    /// If remove prior is true, expect that the token is not composed, and
    /// remove the token owner from self.tokens_per_owner.
    fn transfer_internal(
        &mut self,
        token: &mut Token,
        to: AccountId,
        remove_prior: bool,
    ) {
        let update_set = if remove_prior {
            Some(AccountId::try_from(token.owner_id.to_string()).unwrap())
        } else {
            None
        };
        token.split_owners = None;
        self.update_tokens_per_owner(token.id, update_set, Some(to.clone()));
        token.owner_id = Owner::Account(to);
        token.approvals.clear();
        self.tokens.insert(&token.id, token);
    }

    /// Same as `nft_is_approved`, but uses internal u64 (u64) typing for
    /// Copy-efficiency.
    fn nft_is_approved_internal(
        &self,
        token: &Token,
        approved_account_id: AccountId,
        approval_id: Option<u64>,
    ) -> bool {
        if approved_account_id.to_string() == token.owner_id.to_string() {
            true
        } else {
            let approval_id = approval_id.expect("approval_id required");
            let stored_approval = token.approvals.get(&approved_account_id);
            match stored_approval {
                None => false,
                Some(&stored_approval_id) => stored_approval_id == approval_id,
            }
        }
    }

    /// Internal
    /// Transfer a token_id from one account's owned-token-set to another's.
    /// Callers of this method MUST validate that `from` owns the token before
    /// calling this method.
    ///
    /// If `to` is None, the tokens are either being burned or composed.
    ///
    /// If `from` is None, the tokens are being uncomposed.
    ///
    /// If neither are None, the tokens are being transferred.
    fn update_tokens_per_owner(
        &mut self,
        token_id: u64,
        from: Option<AccountId>,
        to: Option<AccountId>,
    ) {
        if let Some(from) = from {
            let mut old_owner_owned_set = self.tokens_per_owner.get(&from).unwrap();
            old_owner_owned_set.remove(&token_id);
            if old_owner_owned_set.is_empty() {
                self.tokens_per_owner.remove(&from);
            } else {
                self.tokens_per_owner.insert(&from, &old_owner_owned_set);
            }
        }
        if let Some(to) = to {
            let mut new_owner_owned_set = self.get_or_make_new_owner_set(&to);
            new_owner_owned_set.insert(&token_id);
            self.tokens_per_owner.insert(&to, &new_owner_owned_set);
        }
    }

    // TODO: unused, deprecated?
    // /// Internal
    // /// update the set of tokens composed underneath parent. If insert is
    // /// true, insert token_id; if false, try to remove it.
    // fn update_composed_sets(
    //     &mut self,
    //     child: String,
    //     parent: String,
    //     insert: bool,
    // ) {
    //     let mut set = self.get_or_new_composed(parent.to_string());
    //     if insert {
    //         set.insert(&child);
    //     } else {
    //         set.remove(&child);
    //     }
    //     if set.is_empty() {
    //         self.composeables.remove(&parent);
    //     } else {
    //         self.composeables.insert(&parent, &set);
    //     }
    // }

    // TODO: unused, deprecated?
    // /// Internal
    // /// update the set of tokens composed underneath parent. If insert is
    // /// true, insert token_id; if false, try to remove it.
    // pub(crate) fn get_or_new_composed(
    //     &mut self,
    //     parent: String,
    // ) -> UnorderedSet<String> {
    //     self.composeables.get(&parent).unwrap_or_else(|| {
    //         let mut prefix: Vec<u8> = vec![b'h'];
    //         prefix.extend_from_slice(parent.to_string().as_bytes());
    //         UnorderedSet::new(prefix)
    //     })
    // }

    /// If an account_id has never owned tokens on this store, we must
    /// construct an `UnorderedSet` for them. If they have owned tokens on
    /// this store, get that set.
    /// Internal
    pub(crate) fn get_or_make_new_owner_set(
        &self,
        account_id: &AccountId,
    ) -> UnorderedSet<u64> {
        self.tokens_per_owner.get(account_id).unwrap_or_else(|| {
            let mut prefix: Vec<u8> = vec![b'j'];
            prefix.extend_from_slice(account_id.as_bytes());
            UnorderedSet::new(prefix)
        })
    }

    /// Get the storage in bytes to mint `num_tokens` each with
    /// `metadata_storage` and `len_map` royalty receivers.
    /// Internal
    pub fn storage_cost_to_mint(
        &self,
        num_tokens: u64,
        metadata_storage: StorageUsage,
        num_royalties: u32,
        num_splits: u32,
    ) -> near_sdk::Balance {
        // create an entry in tokens_per_owner
        self.storage_costs.common
            // create a metadata record
            + metadata_storage as u128 * self.storage_costs.storage_price_per_byte
            // create a royalty record
            + num_royalties as u128 * self.storage_costs.common
            // create n tokens each with splits stored on-token
            + num_tokens as u128 * (self.storage_costs.token + num_splits as u128 * self.storage_costs.common)
    }

    /// Internal
    fn lock_token(
        &mut self,
        token: &mut Token,
    ) {
        if let Owner::Account(ref s) = token.owner_id {
            token.owner_id = Owner::Lock(s.clone());
            self.tokens.insert(&token.id, token);
        }
    }

    /// Internal
    fn unlock_token(
        &mut self,
        token: &mut Token,
    ) {
        if let Owner::Lock(ref s) = token.owner_id {
            token.owner_id = Owner::Account(s.clone());
            self.tokens.insert(&token.id, token);
        }
    }

    #[payable]
    pub fn nft_transfer(
        &mut self,
        receiver_id: AccountId,
        token_id: U64,
        approval_id: Option<u64>,
        memo: Option<String>,
    ) {
        assert_one_yocto();
        let token_idu64 = token_id.into();
        let mut token = self.nft_token_internal(token_idu64);
        let old_owner = token.owner_id.to_string();
        assert!(!token.is_loaned());
        if !token.is_pred_owner() {
            assert!(self.nft_is_approved_internal(
                &token,
                env::predecessor_account_id(),
                approval_id
            ));
        }

        self.transfer_internal(&mut token, receiver_id.clone(), true);
        log_nft_transfer(&receiver_id, token_idu64, &memo, old_owner);
    }

    #[payable]
    pub fn nft_transfer_call(
        &mut self,
        receiver_id: AccountId,
        token_id: U64,
        approval_id: Option<u64>,
        msg: String,
    ) -> Promise {
        assert_one_yocto();
        let token_idu64 = token_id.into();
        let mut token = self.nft_token_internal(token_idu64);
        assert!(!token.is_loaned());
        let pred = env::predecessor_account_id();
        if !token.is_pred_owner() {
            // check if pred has an approval
            let approval_id: Option<u64> = approval_id;
            assert!(self.nft_is_approved_internal(&token, pred.clone(), approval_id));
        }
        // prevent race condition, temporarily lock-replace owner
        let owner_id = AccountId::new_unchecked(token.owner_id.to_string());
        self.lock_token(&mut token);

        ext_on_transfer::nft_on_transfer(
            pred,
            owner_id.clone(),
            token_id,
            msg,
            receiver_id.clone(),
            NO_DEPOSIT,
            Gas(GAS_NFT_TRANSFER_CALL),
        )
        .then(store_self::nft_resolve_transfer(
            owner_id,
            receiver_id,
            token_id.0.to_string(),
            None,
            env::current_account_id(),
            NO_DEPOSIT,
            Gas(GAS_NFT_TRANSFER_CALL),
        ))
    }

    pub fn nft_token(
        &self,
        token_id: U64,
    ) -> Option<TokenCompliant> {
        Some(self.nft_token_compliant_internal(token_id.0))
    }
}

// ----------------------- contract interface modules ----------------------- //

#[ext_contract(store_self)]
pub trait NonFungibleResolveTransfer {
    /// Finalize an `nft_transfer_call` chain of cross-contract calls.
    ///
    /// The `nft_transfer_call` process:
    ///
    /// 1. Sender calls `nft_transfer_call` on FT contract
    /// 2. NFT contract transfers token from sender to receiver
    /// 3. NFT contract calls `nft_on_transfer` on receiver contract
    /// 4+. [receiver contract may make other cross-contract calls]
    /// N. NFT contract resolves promise chain with `nft_resolve_transfer`, and may
    ///    transfer token back to sender
    ///
    /// Requirements:
    /// * Contract MUST forbid calls to this function by any account except self
    /// * If promise chain failed, contract MUST revert token transfer
    /// * If promise chain resolves with `true`, contract MUST return token to
    ///   `sender_id`
    ///
    /// Arguments:
    /// * `sender_id`: the sender of `ft_transfer_call`
    /// * `token_id`: the `token_id` argument given to `ft_transfer_call`
    /// * `approved_token_ids`: if using Approval Management, contract MUST provide
    ///   set of original approved accounts in this argument, and restore these
    ///   approved accounts in case of revert.
    ///
    /// Returns true if token was successfully transferred to `receiver_id`.
    ///
    /// Mild modifications from core standard, commented where applicable.
    #[private]
    fn nft_resolve_transfer(
        &mut self,
        owner_id: AccountId,
        receiver_id: AccountId,
        token_id: String,
        approved_account_ids: Option<Vec<String>>,
    );
}

// ------------------------ impls on external types ------------------------- //
impl NewSplitOwner for SplitOwners {
    fn new(split_between: HashMap<near_sdk::AccountId, u32>) -> Self {
        assert!(split_between.len() >= 2);
        // validate args
        let mut sum: u32 = 0;
        let split_between: HashMap<AccountId, SafeFraction> = split_between
            .into_iter()
            .map(|(addr, numerator)| {
                assert!(env::is_valid_account_id(addr.as_bytes()));
                let sf = SafeFraction::new(numerator);
                sum += sf.numerator;
                (addr, sf)
            })
            .collect();
        assert!(sum == 10_000, "sum not 10_000: {}", sum);

        Self { split_between }
    }
}
// --------------------------- logging functions ---------------------------- //
// TODO: move those here :)

// ---------------------------------- misc ---------------------------------- //
