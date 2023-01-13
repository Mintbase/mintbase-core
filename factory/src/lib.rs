use std::convert::TryFrom;
use std::str::FromStr;

use mintbase_deps::common::{
    NFTContractMetadata,
    StoreInitArgs,
};
use mintbase_deps::constants::{
    gas,
    storage_bytes,
    storage_stake,
    YOCTO_PER_BYTE,
};
use mintbase_deps::interfaces::factory_self;
use mintbase_deps::logging::MbStoreDeployData;
use mintbase_deps::near_sdk::borsh::{
    self,
    BorshDeserialize,
    BorshSerialize,
};
use mintbase_deps::near_sdk::collections::LookupSet;
use mintbase_deps::near_sdk::json_types::U128;
use mintbase_deps::near_sdk::{
    self,
    assert_one_yocto,
    env,
    is_promise_success,
    near_bindgen,
    AccountId,
    Balance,
    Promise,
    PublicKey,
};
use mintbase_deps::serde_json;
// ------------------------------- constants -------------------------------- //

// ----------------------------- smart contract ----------------------------- //
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct MintbaseStoreFactory {
    /// The `Store`s this `Factory` has produced.
    pub stores: LookupSet<String>,
    /// Fee taken by Mintbase for `Store` deployment.
    pub mintbase_fee: Balance,
    /// The owner may update the `mintbase_fee`.
    pub owner_id: AccountId,
    /// The Near-denominated price-per-byte of storage. As of April 2021, the
    /// price per bytes is set by default to 10^19, but this may change in the
    /// future, thus this future-proofing field.
    pub storage_price_per_byte: u128,
    /// Cost in yoctoNear to deploy a store. Changes if `storage_price_per_byte`
    /// changes.
    pub store_cost: u128,
    /// The public key to give a full access key to
    pub admin_public_key: PublicKey,
}

// ----------------------- contract interface modules ----------------------- //
impl Default for MintbaseStoreFactory {
    fn default() -> Self {
        env::panic_str("Not initialized yet.");
    }
}

#[near_bindgen]
impl MintbaseStoreFactory {
    /// Panics if not called by the factory owner
    fn assert_only_owner(&self) {
        assert_one_yocto();
        assert_eq!(
            env::predecessor_account_id(),
            self.owner_id,
            "Only contract owner can call this method"
        );
    }

    /// Panics if a store with the requested ID already exists
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
        (storage_bytes::STORE as u128 * self.storage_price_per_byte + self.mintbase_fee).into()
    }

    /// Public key that will be attached to any created store.
    pub fn get_admin_public_key(&self) -> &PublicKey {
        &self.admin_public_key
    }

    /// Retrieve the storage price per byte in yocotNEAR currently registered
    /// with the factory.
    pub fn get_storage_price_per_byte(&self) -> U128 {
        self.storage_price_per_byte.into()
    }

    /// Retrieve the store cost in yocotNEAR currently registered with the
    /// factory.
    pub fn get_store_cost(&self) -> U128 {
        self.store_cost.into()
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
        self.store_cost = self.storage_price_per_byte * storage_bytes::STORE as u128;
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
            env::log_str(
                &MbStoreDeployData {
                    contract_metadata: metadata,
                    owner_id: owner_id.to_string(),
                    store_id: store_account_id.to_string(),
                }
                .serialize_event(),
            );
            Promise::new(self.owner_id.to_string().parse().unwrap())
                .transfer(attached_deposit - self.store_cost);
            // #[cfg(feature = "panic-test")]
            // env::panic_str("event.near_json_event().as_str()");
        } else {
            // Refunding store cost creation to the store creator
            Promise::new(store_creator_id).transfer(attached_deposit - self.store_cost);
            env::log_str("failed store deployment");
        }
    }

    /// Initialization
    #[init(ignore_state)]
    pub fn new() -> Self {
        assert!(!env::state_exists());
        let storage_price_per_byte = YOCTO_PER_BYTE; // 10^19
        Self {
            stores: LookupSet::new(b"t".to_vec()),
            mintbase_fee: 0, // 0 by default
            owner_id: env::predecessor_account_id(),
            storage_price_per_byte,
            store_cost: storage_stake::STORE,
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
        assert!(env::attached_deposit() >= self.store_cost);
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
            .function_call("new".to_string(), init_args, 0, gas::CREATE_STORE)
            .then(
                factory_self::ext(env::current_account_id())
                    .with_static_gas(gas::ON_CREATE_CALLBACK)
                    .on_create(
                        env::predecessor_account_id(),
                        metadata,
                        owner_id,
                        store_account_id,
                        env::attached_deposit().into(),
                    ),
            )
    }
}

// ------------------------ impls on external types ------------------------- //
// TODO: Why the trait? -> to be able to impl it in this crate
pub trait New {
    fn new(arg: Self) -> Self;
}

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
