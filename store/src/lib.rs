use mintbase_deps::common::{
    NFTContractMetadata,
    Royalty,
    TokenMetadata,
    TokenMetadataCompliant,
};
use mintbase_deps::constants::{
    storage_stake,
    StorageCosts,
    YOCTO_PER_BYTE,
};
use mintbase_deps::near_assert;
use mintbase_deps::near_sdk::borsh::{
    self,
    BorshDeserialize,
    BorshSerialize,
};
use mintbase_deps::near_sdk::collections::{
    LookupMap,
    UnorderedSet,
};
use mintbase_deps::near_sdk::json_types::{
    U128,
    U64,
};
use mintbase_deps::near_sdk::{
    self,
    env,
    ext_contract,
    near_bindgen,
    AccountId,
    StorageUsage,
};
use mintbase_deps::token::{
    Owner,
    Token,
};

/// Implementing approval management as [described in the Nomicon](https://nomicon.io/Standards/NonFungibleToken/ApprovalManagement).
mod approvals;
/// Implementing any methods related to burning.
mod burning;
/// Implementing core functionality of an NFT contract as [described in the Nomicon](https://nomicon.io/Standards/NonFungibleToken/Core).
mod core;
/// Implementing enumeration as [described in the Nomicon](https://nomicon.io/Standards/NonFungibleToken/Enumeration).
mod enumeration;
/// Implementing metadata as [described in the Nomicon](https://nomicon.io/Standards/NonFungibleToken/Metadata).
mod metadata;
/// Implementing any methods related to minting.
mod minting;
/// Implementing any methods related to store ownership.
mod ownership;
/// Implementing payouts as [described in the Nomicon](https://nomicon.io/Standards/NonFungibleToken/Payout).
mod payout;

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

#[near_bindgen]
impl MintbaseStore {
    /// Create a new `Store`. `new` validates the `store_description`.
    ///
    /// The `Store` is initialized with the owner as a `minter`.
    #[init]
    pub fn new(
        metadata: NFTContractMetadata,
        owner_id: AccountId,
    ) -> Self {
        near_assert!(!env::state_exists(), "This store is already initialized!");
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
            storage_costs: StorageCosts::new(YOCTO_PER_BYTE), // 10^19
            allow_moves: true,
        }
    }

    // -------------------------- change methods ---------------------------
    // -------------------------- view methods -----------------------------

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

    // -------------------------- private methods --------------------------

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

    #[private]
    pub fn prepend_base_uri(
        &mut self,
        base_uri: String,
        token_ids_with_media: Vec<(String, Option<String>)>,
    ) {
        for (token_id, media) in token_ids_with_media
            .iter()
            .map(|(id, media)| (id.parse::<u64>().unwrap(), media))
        {
            let metadata_id = self.tokens.get(&token_id).unwrap().metadata_id;
            let (n, mut metadata) = self.token_metadata.get(&metadata_id).unwrap();
            metadata.reference = concat_uri(&base_uri, &metadata.reference);
            metadata.media = concat_uri(&base_uri, &media);
            self.token_metadata.insert(&metadata_id, &(n, metadata));
        }
    }

    #[private]
    pub fn drop_base_uri(&mut self) {
        self.metadata.base_uri = None;
    }

    // -------------------------- internal methods -------------------------

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

fn concat_uri(
    base: &str,
    uri: &Option<String>,
) -> Option<String> {
    match uri {
        None => None,
        Some(uri) if uri.starts_with(base) => Some(uri.to_string()),
        Some(uri) if base.ends_with('/') => Some(format!("{}{}", base, uri)),
        Some(uri) => Some(format!("{}/{}", base, uri)),
    }
}
