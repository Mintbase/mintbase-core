use mintbase_deps::constants::gas;
use mintbase_deps::interfaces::ext_on_approve;
use mintbase_deps::logging::{
    NftApproveData,
    NftApproveLog,
    NftRevokeAllData,
    NftRevokeData,
};
use mintbase_deps::near_sdk::json_types::U64;
use mintbase_deps::near_sdk::{
    self,
    env,
    near_bindgen,
    AccountId,
    Promise,
};
use mintbase_deps::store_data::Token;
use mintbase_deps::{
    assert_storage_deposit,
    assert_token_owned_by_predecessor,
    assert_token_unloaned,
    assert_yocto_deposit,
};

use crate::*;

// --------------------- standardized approval methods ---------------------- //
#[near_bindgen]
impl MintbaseStore {
    // -------------------------- change methods ---------------------------
    /// Granting NFT transfer approval as specified by
    /// [NEP-178](https://nomicon.io/Standards/Tokens/NonFungibleToken/ApprovalManagement)
    #[payable]
    pub fn nft_approve(
        &mut self,
        token_id: U64,
        account_id: AccountId,
        msg: Option<String>,
    ) -> Option<Promise> {
        // Note: This method only guarantees that the store-storage is covered.
        // The market may still reject.
        assert_storage_deposit!(self.storage_costs.common);
        let token_idu64 = token_id.into();
        // validates owner and loaned
        let approval_id = self.approve_internal(token_idu64, &account_id);
        log_approve(token_idu64, approval_id, &account_id);

        if let Some(msg) = msg {
            ext_on_approve::ext(account_id)
                .with_static_gas(gas::NFT_ON_APPROVE)
                .nft_on_approve(token_id, env::predecessor_account_id(), approval_id, msg)
                .into()
        } else {
            None
        }
    }

    /// Revokes NFT transfer approval as specified by
    /// [NEP-178](https://nomicon.io/Standards/Tokens/NonFungibleToken/ApprovalManagement)
    #[payable]
    pub fn nft_revoke(
        &mut self,
        token_id: U64,
        account_id: AccountId,
    ) {
        let token_idu64 = token_id.into();
        let mut token = self.nft_token_internal(token_idu64);
        assert_token_unloaned!(token);
        assert_token_owned_by_predecessor!(token);
        assert_yocto_deposit!();

        if token.approvals.remove(&account_id).is_some() {
            self.tokens.insert(&token_idu64, &token);
            log_revoke(token_idu64, &account_id);
        }
        // TODO: refund storage deposit
    }

    /// Revokes all NFT transfer approvals as specified by
    /// as specified by [NEP-178](https://nomicon.io/Standards/Tokens/NonFungibleToken/ApprovalManagement)
    #[payable]
    pub fn nft_revoke_all(
        &mut self,
        token_id: U64,
    ) {
        let token_idu64 = token_id.into();
        let mut token = self.nft_token_internal(token_idu64);
        assert_token_unloaned!(token);
        assert_token_owned_by_predecessor!(token);
        assert_yocto_deposit!();

        if !token.approvals.is_empty() {
            token.approvals.clear();
            self.tokens.insert(&token_idu64, &token);
            log_revoke_all(token_idu64);
        }
        // TODO: refund storage deposit
    }

    // -------------------------- view methods -----------------------------
    pub fn nft_is_approved(
        &self,
        token_id: U64,
        approved_account_id: AccountId,
        approval_id: Option<u64>,
    ) -> bool {
        self.nft_is_approved_internal(
            &self.nft_token_internal(token_id.into()),
            &approved_account_id,
            approval_id,
        )
    }
}

// ------------------- non-standardized approval methods -------------------- //
#[near_bindgen]
impl MintbaseStore {
    // -------------------------- change methods ---------------------------
    /// Like `nft_approve`, but it allows approving multiple tokens in one call.
    /// The `msg` argument will be forwarded towards a `nft_on_batch_approve`.
    /// As this is not standardized and only supported by the legacy Mintbase
    /// market.
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
        let storage_stake = self.storage_costs.common * tlen;
        // Note: This method only guarantees that the store-storage is covered.
        // The financial contract may still reject.
        assert_storage_deposit!(storage_stake);
        let approval_ids: Vec<U64> = token_ids
            .iter()
            // validates owner and loaned
            .map(|&token_id| self.approve_internal(token_id.into(), &account_id).into())
            .collect();
        log_batch_approve(&token_ids, &approval_ids, &account_id);

        if let Some(msg) = msg {
            ext_on_approve::ext(account_id)
                .with_attached_deposit(env::attached_deposit() - storage_stake)
                .with_static_gas(gas::NFT_BATCH_APPROVE)
                .nft_on_batch_approve(token_ids, approval_ids, env::predecessor_account_id(), msg)
                .into()
        } else {
            None
        }
    }

    // -------------------------- view methods -----------------------------
    /// Returns the most recent `approval_id` for `account_id` on `token_id`.
    /// If the account doesn't have approval on the token, it will return
    /// `None`.
    ///
    /// Panics if the token doesn't exist.
    pub fn nft_approval_id(
        &self,
        token_id: U64,
        account_id: AccountId,
    ) -> Option<u64> {
        let token = self.nft_token_internal(token_id.into());
        token.approvals.get(&account_id).cloned()
    }

    // -------------------------- private methods --------------------------
    // -------------------------- internal methods -------------------------

    /// Called from nft_approve and nft_batch_approve.
    fn approve_internal(
        &mut self,
        token_idu64: u64,
        account_id: &AccountId,
    ) -> u64 {
        let mut token = self.nft_token_internal(token_idu64);
        // token.assert_unloaned();
        // token.assert_owned_by_predecessor();
        assert_token_unloaned!(token);
        assert_token_owned_by_predecessor!(token);

        let approval_id = self.num_approved;
        self.num_approved += 1;
        token.approvals.insert(account_id.clone(), approval_id);
        self.tokens.insert(&token_idu64, &token);
        approval_id
    }

    /// Same as `nft_is_approved`, but uses internal u64 (u64) typing for
    /// Copy-efficiency.
    pub(crate) fn nft_is_approved_internal(
        &self,
        token: &Token,
        approved_account_id: &AccountId,
        approval_id: Option<u64>,
    ) -> bool {
        if approved_account_id.to_string() == token.owner_id.to_string() {
            true
        } else {
            let approval_id = approval_id.expect("approval_id required");
            let stored_approval = token.approvals.get(approved_account_id);
            match stored_approval {
                None => false,
                Some(&stored_approval_id) => stored_approval_id == approval_id,
            }
        }
    }
}

fn log_approve(
    token_id: u64,
    approval_id: u64,
    account_id: &AccountId,
) {
    let data = NftApproveData(vec![NftApproveLog {
        token_id: token_id.into(),
        approval_id,
        account_id: account_id.to_string(),
    }]);
    env::log_str(&data.serialize_event());
}

fn log_batch_approve(
    tokens: &[U64],
    approvals: &[U64],
    account_id: &AccountId,
) {
    let data = NftApproveData(
        approvals
            .iter()
            .zip(tokens.iter())
            .map(|(approval_id, token_id)| NftApproveLog {
                token_id: *token_id,
                approval_id: approval_id.0,
                account_id: account_id.to_string(),
            })
            .collect::<Vec<_>>(),
    );
    env::log_str(&data.serialize_event());
}

fn log_revoke(
    token_id: u64,
    account_id: &AccountId,
) {
    env::log_str(
        &NftRevokeData {
            token_id: token_id.into(),
            account_id: account_id.to_string(),
        }
        .serialize_event(),
    );
}

fn log_revoke_all(token_id: u64) {
    env::log_str(
        &NftRevokeAllData {
            token_id: token_id.into(),
        }
        .serialize_event(),
    );
}
