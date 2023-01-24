use mintbase_deps::logging::MbStoreChangeSettingData;
use mintbase_deps::near_assert;
use mintbase_deps::near_sdk::{
    self,
    assert_one_yocto,
    near_bindgen,
    AccountId,
};

use crate::minting::{
    log_grant_minter,
    log_revoke_minter,
};
use crate::*;

#[near_bindgen]
impl MintbaseStore {
    // -------------------------- change methods ---------------------------
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
        near_assert!(
            new_owner != self.owner_id,
            "{} already owns this store",
            new_owner
        );
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
        if unused_deposit > storage_stake::CUSHION {
            near_sdk::Promise::new(self.owner_id.clone())
                .transfer(unused_deposit - storage_stake::CUSHION);
        } else {
            let s = format!(
                "Nothing withdrawn. Unused deposit is less than 0.5N: {}",
                unused_deposit
            );
            env::log_str(s.as_str());
        }
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

    // -------------------------- view methods -----------------------------
    /// Show the current owner of this NFT contract
    pub fn get_owner_id(&self) -> AccountId {
        self.owner_id.clone()
    }

    /// Show the current owner of this NFT contract
    pub fn get_storage_costs(&self) -> StorageCosts {
        self.storage_costs.clone()
    }

    // -------------------------- private methods --------------------------
    // -------------------------- internal methods -------------------------

    /// Validate the caller of this method matches the owner of this `Store`.
    pub(crate) fn assert_store_owner(&self) {
        assert_one_yocto();
        near_assert!(
            self.owner_id == env::predecessor_account_id(),
            "This method can only be called by the store owner"
        );
    }
}

fn log_transfer_store(account_id: &AccountId) {
    env::log_str(
        &MbStoreChangeSettingData {
            new_owner: Some(account_id.to_string()),
            ..MbStoreChangeSettingData::empty()
        }
        .serialize_event(),
    );
}
