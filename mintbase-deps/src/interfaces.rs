use near_sdk::json_types::{
    U128,
    U64,
};
use near_sdk::{
    self,
    ext_contract,
    AccountId,
    Promise,
};

#[ext_contract(ext_old_market)]
pub trait ExtOldMarket {
    fn resolve_nft_payout(
        &mut self,
        token_key: String,
        token: crate::market_data::TokenListing,
        others_keep: U128,
        market_keeps: U128,
    ) -> Promise;
}

// #[ext_contract(ext_new_market)]
// pub trait ExtNewMarket {
//     // FIXME: correct signature!
//     fn resolve_nft_payout(
//         &mut self,
//         token_key: String,
//         token: TokenListing,
//         others_keep: U128,
//         market_keeps: U128,
//     ) -> Promise;
// }

#[ext_contract(ext_nft)]
pub trait ExtNft {
    /// Transfer the token and get the payout data.
    fn nft_transfer_payout(
        &mut self,
        receiver_id: AccountId,
        token_id: U64,
        approval_id: u64,
        balance: U128,
        max_len_payout: u32,
    ) -> Promise;
    // TODO: resolve_transfer?
}

#[ext_contract(ext_nft_on_approve)]
pub trait ExtNftOnApprove {
    /// Approved Account Contract Interface If a contract that gets approved to
    /// transfer NFTs wants to, it can implement nft_on_approve to update its own
    /// state when granted approval for a token: Respond to notification that
    /// contract has been granted approval for a token.
    ///
    /// Notes
    /// * Contract knows the token contract ID from `predecessor_account_id`
    ///
    /// Arguments:
    /// * `token_id`: the token to which this contract has been granted approval
    /// * `owner_id`: the owner of the token
    /// * `approval_id`: the approval ID stored by NFT contract for this approval.
    ///   Expected to be a number within the 2^53 limit representable by JSON.
    /// * `msg`: specifies information needed by the approved contract in order to
    ///    handle the approval. Can indicate both a fn to call and the
    ///    parameters to pass to that fn.
    fn nft_on_approve(
        &mut self,
        token_id: U64,
        owner_id: AccountId,
        approval_id: u64,
        msg: String,
    );
    /// Batched version of `nft_on_approve`, not standardized!
    fn nft_on_batch_approve(
        &mut self,
        tokens: Vec<U64>,
        approvals: Vec<U64>,
        owner_id: AccountId,
        msg: String,
    );
}

#[ext_contract(ext_nft_on_transfer)]
pub trait ExtNftOnTransfer {
    /// Take some action after receiving a non-fungible token.
    ///
    /// Requirements: * Contract MUST restrict calls to this function to a set of
    /// allow-listed NFT contracts.
    ///
    /// Arguments:
    /// * `sender_id`: the sender of `nft_transfer_call`.
    /// * `previous_owner_id`: the account that owned the NFT prior to it being
    ///   transfered to this contract, which can differ from `sender_id` if using
    ///   Approval Management extension.
    /// * `token_id`: the `token_id` argument given to `nft_transfer_call`
    /// * `msg`: information necessary for this contract to know how to process the
    ///   request. This may include method names and/or arguments.
    ///
    /// Returns true if token should be returned to `sender_id`.
    fn nft_on_transfer(
        &mut self,
        sender_id: AccountId,
        previous_owner_id: AccountId,
        token_id: U64,
        msg: String,
    ) -> Promise;
}

#[ext_contract(ext_factory)]
pub trait ExtFactory {
    fn on_create(
        &mut self,
        store_creator_id: AccountId,
        metadata: crate::store_data::NFTContractMetadata,
        owner_id: AccountId,
        store_account_id: AccountId,
        attached_deposit: U128,
    );
}
