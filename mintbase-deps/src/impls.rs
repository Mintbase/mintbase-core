use std::collections::HashMap;
use std::convert::{
    TryFrom,
    TryInto,
};
use std::fmt;

#[cfg(feature = "wasm")]
pub use near_sdk::{
    borsh::{
        self,
        BorshDeserialize,
        BorshSerialize,
    },
    collections::*,
    json_types::*,
    *,
};

use crate::*;

impl NearTime {
    pub fn is_before_timeout(&self) -> bool {
        now().0 < self.0
    }

    pub fn new(span: TimeUnit) -> Self {
        match span {
            TimeUnit::Hours(n) => Self::now_plus_n_hours(n),
        }
    }

    fn now_plus_n_hours(n: u64) -> Self {
        assert!(n > 0);
        assert!(
            n < 70_000,
            "maximum argument for hours is 70,000 (~8 years)"
        );
        let now = env::block_timestamp();
        let hour_ns = 10u64.pow(9) * 3600;
        Self(now + n * hour_ns)
    }
}

impl StorageCostsMarket {
    pub fn new(storage_price_per_byte: u128) -> Self {
        Self {
            storage_price_per_byte,
            list: storage_price_per_byte * LIST_STORAGE as u128,
        }
    }
}

impl TokenOffer {
    /// Timeout is in days.
    pub fn new(
        price: u128,
        timeout: TimeUnit,
        id: u64,
    ) -> Self {
        let timeout = NearTime::new(timeout);
        Self {
            id,
            price,
            from: env::predecessor_account_id(),
            timestamp: now(),
            timeout,
        }
    }

    /// An offer is active if it has yet to timeout.
    pub fn is_active(&self) -> bool {
        self.timeout.is_before_timeout()
    }
}

impl TokenListing {
    /// Check that the given `account_id` is valid before instantiating a
    /// `Token`. Note that all input validation for `Token` functions should
    /// be performed at the `Marketplace` level.
    pub fn new(
        owner_id: AccountId,
        store_id: AccountId,
        id: u64,
        approval_id: u64,
        autotransfer: bool,
        asking_price: U128,
    ) -> Self {
        Self {
            id,
            owner_id,
            store_id,
            approval_id,
            autotransfer,
            asking_price,
            current_offer: None,
            num_offers: 0,
            locked: false,
        }
    }

    /// Unique identifier of the Token.
    pub fn get_token_key(&self) -> TokenKey {
        TokenKey::new(self.id, self.store_id.to_string().try_into().unwrap())
    }

    /// Unique identifier of the Token, which is also unique across
    /// relistings of the Token.
    pub fn get_list_id(&self) -> String {
        format!("{}:{}:{}", self.id, self.approval_id, self.store_id)
    }

    pub fn assert_not_locked(&self) {
        assert!(!self.locked);
    }
}

impl StorageCosts {
    pub fn new(storage_price_per_byte: u128) -> Self {
        Self {
            storage_price_per_byte,
            common: storage_price_per_byte * 80_u64 as u128,
            token: storage_price_per_byte * 360_u64 as u128,
        }
    }
}

impl NearJsonEvent {
    pub fn near_json_event(&self) -> String {
        let json = serde_json::to_string(&self).unwrap();
        format!("EVENT_JSON: {}", &json)
    }
}

impl Nep171Event {
    pub fn near_json_event(&self) -> String {
        let json = serde_json::to_string(&self).unwrap();
        format!("EVENT_JSON: {}", &json)
    }
}

impl fmt::Display for NftEventError {
    fn fmt(
        &self,
        f: &mut fmt::Formatter,
    ) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(feature = "factory-wasm")]
impl New for NFTContractMetadata {
    fn new(args: NFTContractMetadata) -> Self {
        let store_account = format!("{}.{}", args.name, env::current_account_id());
        assert!(
            env::is_valid_account_id(store_account.as_bytes()),
            "Invalid character in store id"
        );
        assert!(args.symbol.len() <= 6);

        Self {
            spec: args.spec,
            name: args.name,
            symbol: args.symbol,
            icon: args.icon,
            base_uri: args.base_uri,
            reference: args.reference,
            reference_hash: args.reference_hash,
        }
    }
}

#[cfg(feature = "factory-wasm")]
impl Default for MintbaseStoreFactory {
    fn default() -> Self {
        env::panic_str("Not initialized yet.");
    }
}

#[cfg(feature = "helper-wasm")]
#[cfg_attr(feature = "helper-wasm", near_bindgen)]
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

////////////////
// Core Logic //
////////////////
#[cfg(feature = "factory-wasm")]
#[cfg_attr(feature = "factory-wasm", near_bindgen)]
impl MintbaseStoreFactory {
    pub fn assert_only_owner(&self) {
        assert_one_yocto();
        assert_eq!(
            env::predecessor_account_id(),
            self.owner_id,
            "Only contract owner can call this method"
        );
    }

    /// Sufficient attached deposit is defined as enough to deploy a `Store`,
    /// plus enough left over for the mintbase deployment cost.
    pub fn assert_sufficient_attached_deposit(&self) {
        let min = STORE_STORAGE as u128 * self.storage_price_per_byte + self.mintbase_fee;
        assert!(
            env::attached_deposit() >= min,
            "Not enough attached deposit to complete store deployment. Need: {}, got: {}",
            min,
            env::attached_deposit()
        );
    }

    pub fn assert_no_store_with_id(
        &self,
        store_id: String,
    ) {
        assert!(
            !self.check_contains_store(store_id),
            "Store with that ID already exists"
        );
    }

    /// If a `Store` with `store_id` has been produced by this `Factory`, return `true`.
    pub fn check_contains_store(
        &self,
        store_id: String,
    ) -> bool {
        self.stores.contains(&store_id)
    }

    /// Get the `owner_id` of this `Factory`.
    pub fn get_owner(&self) -> &AccountId {
        &self.owner_id
    }

    /// Get the `mintbase_fee` of this `Factory`.
    pub fn get_mintbase_fee(&self) -> U128 {
        self.mintbase_fee.into()
    }

    /// The sum of `mintbase_fee` and `STORE_STORAGE`.
    pub fn get_minimum_attached_balance(&self) -> U128 {
        (STORE_STORAGE as u128 * self.storage_price_per_byte + self.mintbase_fee).into()
    }

    /// The sum of `mintbase_fee` and `STORE_STORAGE`.
    pub fn get_admin_public_key(&self) -> &PublicKey {
        &self.admin_public_key
    }

    /// The Near Storage price per byte has changed in the past, and may change in
    /// the future. This method may never be used.
    #[payable]
    pub fn set_storage_price_per_byte(
        &mut self,
        new_price: U128,
    ) {
        self.assert_only_owner();
        self.storage_price_per_byte = new_price.into();
        self.store_cost = self.storage_price_per_byte * STORE_STORAGE as u128;
    }

    /// Set amount of Near tokens taken by Mintbase for making `Store`s. Provide an
    /// amount denominated in units of yoctoNear, ie. 1 = 10^-24 Near.
    #[payable]
    pub fn set_mintbase_factory_fee(
        &mut self,
        amount: U128,
    ) {
        self.assert_only_owner();
        self.mintbase_fee = amount.into()
    }

    /// Set a new `owner_id` for `Factory`.
    #[payable]
    pub fn set_mintbase_factory_owner(
        &mut self,
        account_id: AccountId,
    ) {
        self.assert_only_owner();
        let account_id = account_id;
        assert_ne!(account_id, env::predecessor_account_id());
        self.owner_id = account_id;
    }

    /// Set the admin public key. If `public_key` is None, use the signer's
    /// public key.
    #[payable]
    pub fn set_admin_public_key(
        &mut self,
        public_key: Option<String>,
    ) {
        self.assert_only_owner();
        match public_key {
            None => {
                assert_ne!(env::signer_account_pk(), self.admin_public_key);
                self.admin_public_key = env::signer_account_pk();
            },
            Some(public_key) => {
                let public_key = public_key.as_bytes().to_vec();
                assert_ne!(public_key, self.admin_public_key.as_bytes());
                self.admin_public_key = PublicKey::try_from(public_key).unwrap();
            },
        }
    }

    /// Handle callback of store creation.
    #[private]
    pub fn on_create(
        &mut self,
        store_creator_id: AccountId,
        metadata: NFTContractMetadata,
        owner_id: AccountId,
        store_account_id: AccountId,
        attached_deposit: U128,
    ) {
        let attached_deposit: u128 = attached_deposit.into();
        if is_promise_success() {
            // pay out self and update contract state
            self.stores.insert(&metadata.name);
            let nscl = NftStoreCreateLog {
                contract_metadata: metadata,
                owner_id: owner_id.to_string(),
                id: store_account_id.to_string(),
            };
            let event = NearJsonEvent {
                standard: "nep171".to_string(),
                version: "1.0.0".to_string(),
                event: "nft_store_creation".to_string(),
                data: serde_json::to_string(&nscl).unwrap(),
            };
            env::log_str(event.near_json_event().as_str());
            Promise::new(self.owner_id.to_string().parse().unwrap())
                .transfer(attached_deposit - self.store_cost);
            #[cfg(feature = "panic-test")]
            env::panic_str("event.near_json_event().as_str()");
        } else {
            // Refunding store cost creation to the store creator
            Promise::new(store_creator_id).transfer(attached_deposit - self.store_cost);
            env::log_str("failed store deployment");
        }
    }

    #[init(ignore_state)]
    pub fn new() -> Self {
        assert!(!env::state_exists());
        let storage_price_per_byte = 10_000_000_000_000_000_000; // 10^19
        Self {
            stores: LookupSet::new(b"t".to_vec()),
            mintbase_fee: 0, // 0 by default
            owner_id: env::predecessor_account_id(),
            storage_price_per_byte,
            store_cost: STORE_STORAGE as u128 * storage_price_per_byte,
            admin_public_key: env::signer_account_pk(),
        }
    }

    /// Contract metadata and methods in the API may be updated. All other
    /// elements of the state should be copied over. This method may only be
    /// called by the holder of the contract private key.
    #[private]
    #[init(ignore_state)]
    pub fn migrate() -> Self {
        let old = env::state_read().expect("ohno ohno state");
        Self { ..old }
    }

    /// `create_store` checks that the attached deposit is sufficient before
    /// parsing the given store_id, validating no such store subaccount exists yet
    /// and generates a new store from the store metadata.
    #[payable]
    pub fn create_store(
        &mut self,
        metadata: NFTContractMetadata,
        owner_id: AccountId,
    ) -> Promise {
        self.assert_sufficient_attached_deposit();
        self.assert_no_store_with_id(metadata.name.clone());
        assert_ne!(&metadata.name, "market"); // marketplace lives here
        assert_ne!(&metadata.name, "loan"); // loan lives here
        let metadata = NFTContractMetadata::new(metadata);
        let init_args = serde_json::to_vec(&StoreInitArgs {
            metadata: metadata.clone(),
            owner_id: owner_id.clone(),
        })
        .unwrap();
        // StoreId is only the subaccount. store_account_id is the full near qualified name.
        // Note, validity checked in `NFTContractMetadata::new;` above.

        let store_account_id =
            AccountId::from_str(&*format!("{}.{}", metadata.name, env::current_account_id()))
                .unwrap();
        Promise::new(store_account_id.clone())
            .create_account()
            .transfer(self.store_cost)
            .add_full_access_key(self.admin_public_key.clone())
            .deploy_contract(include_bytes!("../../wasm/store.wasm").to_vec())
            .function_call("new".to_string(), init_args, 0, GAS_CREATE_STORE)
            .then(factory_self::on_create(
                env::predecessor_account_id(),
                metadata,
                owner_id,
                store_account_id,
                env::attached_deposit().into(),
                env::current_account_id(),
                NO_DEPOSIT,
                GAS_ON_CREATE_CALLBACK,
            ))
    }
}

impl FromStr for NearJsonEvent {
    type Err = serde_json::error::Error;

    fn from_str(_s: &str) -> Result<Self, Self::Err> {
        todo!()
    }
}

impl From<NftEvent> for NearJsonEvent {
    fn from(ne: NftEvent) -> Self {
        let json = serde_json::to_string(&ne).unwrap();
        Self {
            standard: "nep171".to_string(),
            version: "1.0.0".to_string(),
            event: "".to_string(),
            data: json,
        }
    }
}

impl TryFrom<&str> for NftEvent {
    type Error = serde_json::error::Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        // ne.map_err(|x|NftEventError(x.to_string()))
        serde_json::from_str::<NftEvent>(s)
    }
}

/// Stable
impl Royalty {
    /// Validates all arguments. Addresses must be valid and percentages must be
    /// within accepted values. Hashmap percentages must add to 10000.
    pub fn new(royalty_args: RoyaltyArgs) -> Self {
        assert!(!royalty_args.split_between.is_empty());
        let percentage = royalty_args.percentage;
        let split_between = royalty_args.split_between;

        assert!(
            percentage <= ROYALTY_UPPER_LIMIT,
            "percentage: {} must be <= 5000",
            percentage
        );
        assert!(percentage > 0, "percentage cannot be zero");
        assert!(!split_between.is_empty(), "royalty mapping is empty");

        let mut sum: u32 = 0;
        let split_between: SplitBetween = split_between
            .into_iter()
            .map(|(addr, numerator)| {
                assert!(AccountId::try_from(addr.to_string()).is_ok());
                // assert!(env::is_valid_account_id(addr.as_bytes()));
                assert!(numerator > 0, "percentage cannot be zero");
                let sf = SafeFraction::new(numerator);
                sum += sf.numerator;
                (addr, sf)
            })
            .collect();
        assert_eq!(sum, 10_000, "fractions don't add to 10,000");

        Self {
            percentage: SafeFraction::new(percentage),
            split_between,
        }
    }
}

impl Default for NFTContractMetadata {
    fn default() -> Self {
        Self {
            spec: "".to_string(),
            name: "".to_string(),
            symbol: "".to_string(),
            icon: None,
            base_uri: None,
            reference: None,
            reference_hash: None,
        }
    }
}

impl Default for NftStoreCreateLog {
    fn default() -> Self {
        Self {
            contract_metadata: Default::default(),
            owner_id: "".to_string(),
            id: "".to_string(),
        }
    }
}

impl TokenMetadata {
    /// Get the metadata and its size in bytes.
    pub fn from_with_size(
        args: TokenMetadata,
        copies: u64,
    ) -> (Self, u64) {
        if args.media_hash.is_some() {
            assert!(args.media.is_some());
        }

        if args.reference_hash.is_some() {
            assert!(args.reference.is_some());
        }

        let metadata = Self {
            title: args.title,
            description: args.description,
            media: args.media,
            media_hash: args.media_hash,
            copies: (copies as u16).into(),
            expires_at: args.expires_at,
            starts_at: args.starts_at,
            extra: args.extra,
            reference: args.reference,
            reference_hash: args.reference_hash,
        };

        let size = serde_json::to_vec(&metadata).unwrap().len();

        // let size = metadata.try_to_vec().unwrap().len();

        (metadata, size as u64)
    }
}

/// default must be implemented for wasm compilation.
#[cfg(feature = "helper-wasm")]
impl Default for HelperWasm {
    fn default() -> Self {
        Self { count: 0 }
    }
}
#[cfg(feature = "store-wasm")]
impl Default for MintbaseStore {
    fn default() -> Self {
        env::panic_str("no default")
    }
}
#[cfg(feature = "store-wasm")]
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

#[cfg_attr(feature = "store-wasm", near_bindgen)]
#[cfg(feature = "store-wasm")]
impl NonFungibleContractMetadata for MintbaseStore {
    fn nft_metadata(&self) -> &NFTContractMetadata {
        &self.metadata
    }
}

//////////////////////////////
// Store Owner Only Methods //
//////////////////////////////
/// Only the Owner of this `Store` may call these methods.

#[cfg_attr(feature = "store-wasm", near_bindgen)]
#[cfg(feature = "store-wasm")]
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

    pub fn nft_revoke(
        &mut self,
        token_id: U64,
        account_id: AccountId,
    ) {
        let token_idu64 = token_id.into();
        let mut token = self.nft_token_internal(token_idu64);
        assert!(!token.is_loaned());
        assert!(token.is_pred_owner());

        if token.approvals.remove(&account_id).is_some() {
            self.tokens.insert(&token_idu64, &token);
            log_revoke(token_idu64, &account_id);
        }
    }

    pub fn nft_revoke_all(
        &mut self,
        token_id: U64,
    ) {
        let token_idu64 = token_id.into();
        let mut token = self.nft_token_internal(token_idu64);
        assert!(!token.is_loaned());
        assert!(token.is_pred_owner());

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
            .skip(from_index.unwrap_or_else(|| 0.to_string()).parse().unwrap())
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
            self.minters.clear();
        }
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
    pub fn approve_internal(
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

    pub fn nft_token_internal(
        &self,
        token_id: u64,
    ) -> Token {
        self.tokens
            .get(&token_id)
            .unwrap_or_else(|| panic!("token: {} doesn't exist", token_id))
    }

    pub fn nft_token_compliant_internal(
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
    pub fn transfer_internal(
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
    pub fn nft_is_approved_internal(
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
    pub fn update_tokens_per_owner(
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

    /// Internal
    /// update the set of tokens composed underneath parent. If insert is
    /// true, insert token_id; if false, try to remove it.
    pub fn update_composed_sets(
        &mut self,
        child: String,
        parent: String,
        insert: bool,
    ) {
        let mut set = self.get_or_new_composed(parent.to_string());
        if insert {
            set.insert(&child);
        } else {
            set.remove(&child);
        }
        if set.is_empty() {
            self.composeables.remove(&parent);
        } else {
            self.composeables.insert(&parent, &set);
        }
    }

    /// Internal
    /// update the set of tokens composed underneath parent. If insert is
    /// true, insert token_id; if false, try to remove it.
    pub(crate) fn get_or_new_composed(
        &mut self,
        parent: String,
    ) -> UnorderedSet<String> {
        self.composeables.get(&parent).unwrap_or_else(|| {
            let mut prefix: Vec<u8> = vec![b'h'];
            prefix.extend_from_slice(parent.to_string().as_bytes());
            UnorderedSet::new(prefix)
        })
    }

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
    pub fn lock_token(
        &mut self,
        token: &mut Token,
    ) {
        if let Owner::Account(ref s) = token.owner_id {
            token.owner_id = Owner::Lock(s.clone());
            self.tokens.insert(&token.id, token);
        }
    }

    /// Internal
    pub fn unlock_token(
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

impl OwnershipFractions {
    /// Generate a mapping of who receives what from a token's Royalty,
    /// SplitOwners, and normal owner data.
    pub fn new(
        owner_id: &str,
        royalty: &Option<Royalty>,
        split_owners: &Option<SplitOwners>,
    ) -> Self {
        let roy_len = royalty.as_ref().map(|r| r.split_between.len()).unwrap_or(0);
        let split_len = split_owners
            .as_ref()
            .map(|r| r.split_between.len())
            .unwrap_or(1);
        assert!((roy_len + split_len) as u32 <= MAX_LEN_PAYOUT);

        let mut payout: HashMap<AccountId, MultipliedSafeFraction> = Default::default();
        let percentage_not_taken_by_royalty = match royalty {
            Some(royalty) => {
                let (split_between, percentage) =
                    (royalty.split_between.clone(), royalty.percentage);
                split_between.iter().for_each(|(receiver, &rel_perc)| {
                    let abs_perc: MultipliedSafeFraction = percentage * rel_perc;
                    payout.insert(receiver.to_string().parse().unwrap(), abs_perc);
                });
                SafeFraction::new(10_000 - percentage.numerator)
            },
            None => SafeFraction::new(10_000u32),
        };

        match split_owners {
            Some(ref split_owners) => {
                split_owners
                    .split_between
                    .iter()
                    .for_each(|(receiver, &rel_perc)| {
                        let abs_perc: MultipliedSafeFraction =
                            percentage_not_taken_by_royalty * rel_perc;
                        // If an account is already in the payout map, update their take.
                        if let Some(&roy_perc) = payout.get(receiver) {
                            payout.insert(receiver.clone(), abs_perc + roy_perc);
                        } else {
                            payout.insert(receiver.clone(), abs_perc);
                        }
                    });
            },
            None => {
                if let Some(&roy_perc) = payout.get(&AccountId::new_unchecked(owner_id.to_string()))
                {
                    payout.insert(
                        owner_id.to_string().parse().unwrap(),
                        MultipliedSafeFraction::from(percentage_not_taken_by_royalty) + roy_perc,
                    );
                } else {
                    payout.insert(
                        owner_id.to_string().parse().unwrap(),
                        MultipliedSafeFraction::from(percentage_not_taken_by_royalty),
                    );
                }
            },
        };
        Self { fractions: payout }
    }

    pub fn into_payout(
        self,
        balance: Balance,
    ) -> Payout {
        Payout {
            payout: self
                .fractions
                .into_iter()
                .map(|(k, v)| (k, v.multiply_balance(balance).into()))
                .collect(),
        }
    }
}

impl Token {
    /// - `metadata` validation performed in `TokenMetadataArgs::new`
    /// - `royalty` validation performed in `Royalty::new`
    pub fn new(
        owner_id: AccountId,
        token_id: u64,
        metadata_id: u64,
        royalty_id: Option<u64>,
        split_owners: Option<SplitOwners>,
        minter: AccountId,
    ) -> Self {
        Self {
            owner_id: Owner::Account(owner_id),
            id: token_id,
            metadata_id,
            royalty_id,
            split_owners,
            approvals: HashMap::new(),
            minter,
            loan: None,
            composeable_stats: ComposeableStats::new(),
            origin_key: None,
        }
    }

    /// If the token is loaned, return the loaner as the owner.
    pub fn get_owner_or_loaner(&self) -> Owner {
        self.loan
            .as_ref()
            .map(|l| Owner::Account(l.holder.clone()))
            .unwrap_or_else(|| self.owner_id.clone())
    }

    pub fn is_pred_owner(&self) -> bool {
        self.owner_id.to_string() == near_sdk::env::predecessor_account_id().to_string()
    }

    pub fn is_loaned(&self) -> bool {
        self.loan.is_some()
    }
}

impl fmt::Display for Owner {
    fn fmt(
        &self,
        f: &mut fmt::Formatter,
    ) -> fmt::Result {
        match self {
            Owner::Account(s) => write!(f, "{}", s),
            Owner::TokenId(n) => write!(f, "{}", n),
            Owner::CrossKey(key) => write!(f, "{}", key),
            Owner::Lock(_) => panic!("locked"),
        }
    }
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

impl ComposeableStats {
    fn new() -> Self {
        Self {
            local_depth: 0,
            cross_contract_children: 0,
        }
    }
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

impl SafeFraction {
    /// Take a u32 numerator to a 10^4 denominator.
    ///
    /// Upper limit is 10^4 so as to prevent multiplication with overflow.
    pub fn new(numerator: u32) -> Self {
        assert!(
            (0..=10000).contains(&numerator),
            "{} not between 0 and 10,000",
            numerator
        );
        SafeFraction { numerator }
    }

    /// Fractionalize a balance.
    pub fn multiply_balance(
        &self,
        value: Balance,
    ) -> Balance {
        value / 10_000u128 * self.numerator as u128
    }
}

impl std::ops::Sub for SafeFraction {
    type Output = SafeFraction;

    fn sub(
        self,
        rhs: Self,
    ) -> Self::Output {
        assert!(self.numerator >= rhs.numerator);
        Self {
            numerator: self.numerator - rhs.numerator,
        }
    }
}

impl std::ops::SubAssign for SafeFraction {
    fn sub_assign(
        &mut self,
        rhs: Self,
    ) {
        assert!(self.numerator >= rhs.numerator);
        self.numerator -= rhs.numerator;
    }
}

impl std::ops::Mul for SafeFraction {
    type Output = MultipliedSafeFraction;

    fn mul(
        self,
        rhs: Self,
    ) -> Self::Output {
        MultipliedSafeFraction {
            numerator: self.numerator * rhs.numerator,
        }
    }
}

impl From<SafeFraction> for MultipliedSafeFraction {
    fn from(f: SafeFraction) -> Self {
        MultipliedSafeFraction {
            numerator: f.numerator * 10_000,
        }
    }
}

impl std::ops::Add for MultipliedSafeFraction {
    type Output = Self;

    fn add(
        self,
        rhs: Self,
    ) -> Self::Output {
        MultipliedSafeFraction {
            numerator: self.numerator + rhs.numerator,
        }
    }
}

impl MultipliedSafeFraction {
    /// Fractionalize a balance.
    pub fn multiply_balance(
        &self,
        value: Balance,
    ) -> Balance {
        value / 100_000_000u128 * self.numerator as u128
    }
}

#[cfg(feature = "all")]
impl<'a> std::io::Write for StdioLock<'a> {
    fn write(
        &mut self,
        buf: &[u8],
    ) -> std::io::Result<usize> {
        match self {
            StdioLock::Stdout(lock) => lock.write(buf),
            StdioLock::Stderr(lock) => lock.write(buf),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            StdioLock::Stdout(lock) => lock.flush(),
            StdioLock::Stderr(lock) => lock.flush(),
        }
    }

    fn write_all(
        &mut self,
        buf: &[u8],
    ) -> std::io::Result<()> {
        match self {
            StdioLock::Stdout(lock) => lock.write_all(buf),
            StdioLock::Stderr(lock) => lock.write_all(buf),
        }
    }
}

#[cfg(feature = "all")]
impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for MyMakeWriter {
    type Writer = StdioLock<'a>;

    fn make_writer(&'a self) -> Self::Writer {
        // We must have an implementation of `make_writer` that makes
        // a "default" writer without any configuring metadata. Let's
        // just return stdout in that case.
        StdioLock::Stdout(self.stdout.lock())
    }

    fn make_writer_for(
        &'a self,
        meta: &tracing::Metadata<'_>,
    ) -> Self::Writer {
        // Here's where we can implement our special behavior. We'll
        // check if the metadata's verbosity level is WARN or ERROR,
        // and return stderr in that case.
        if meta.level() <= &tracing::Level::WARN {
            return StdioLock::Stderr(self.stderr.lock());
        }

        // Otherwise, we'll return stdout.
        StdioLock::Stdout(self.stdout.lock())
    }
}
