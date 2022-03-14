// this is required because of the `ext_contract` macro
#[cfg(feature = "market-wasm")]
pub use market_interfaces::*;

/// Interfaces that we need the market to be aware of
#[cfg(feature = "market-wasm")]
#[allow(clippy::too_many_arguments)]
mod market_interfaces {
    use near_sdk::json_types::{
        U128,
        U64,
    };
    use near_sdk::{
        self,
        ext_contract,
    };

    use crate::common::TokenListing;

    #[ext_contract(ext_self)]
    pub trait ExtSelf {
        fn resolve_nft_payout(
            &mut self,
            token_key: String,
            token: TokenListing,
            others_keep: U128,
        ) -> Promise;
    }

    #[ext_contract(nft_contract)]
    /// Impl of NEP-171. Note that the impl makes the assumption that `TokenId` has
    /// type `String`, where this contract internally uses `u64`, which is more
    /// efficient. ref:
    /// https://github.com/near/NEPs/blob/master/specs/Standards/NonFungibleToken/Core.md
    pub trait NFTContract {
        /// Transfer the token and get the payout data.
        fn nft_transfer_payout(
            &mut self,
            receiver_id: AccountId,
            token_id: U64,
            approval_id: u64,
            balance: U128,
            max_len_payout: u32,
        ) -> Promise;
    }
}

#[cfg(feature = "store-wasm")]
pub use store_interfaces::*;

/// Interfaces that we need the store to be aware of
#[cfg(feature = "store-wasm")]
#[allow(clippy::too_many_arguments)]
mod store_interfaces {
    use near_sdk::json_types::U64;
    use near_sdk::{
        self,
        ext_contract,
    };

    /// Non-Fungible Token Approval NEP 178. Ref:
    /// https://github.com/near/NEPs/blobß/master/specs/Standards/NonFungibleToken/ApprovalManagement.md
    #[ext_contract(ext_on_approve)]
    pub trait NonFungibleOnApprove {
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
        fn nft_on_batch_approve(
            &mut self,
            tokens: Vec<U64>,
            approvals: Vec<U64>,
            owner_id: AccountId,
            msg: String,
        );
    }

    /// Impl of NEP-171 resolve transfer. ref:
    /// https://github.com/near/NEPs/blob/master/specs/Standards/NonFungibleToken/Core.md
    #[ext_contract(ext_on_transfer)]
    pub trait NonFungibleOnTransfer {
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
}

// TODO: Is this used anywhere?
// --------------------------- nft core interface --------------------------- //
use near_sdk::json_types::U64;
use near_sdk::{
    AccountId,
    Promise,
};

use crate::common::Token;

/// Impl of NEP-171. Note that the impl makes the assumption that `TokenId` has
/// type `String`, where this contract internally uses `u64`, which is more
/// efficient. ref:
/// https://github.com/near/NEPs/blob/master/specs/Standards/NonFungibleToken/Core.md
pub trait NonFungibleContractCore {
    /// Simple transfer. Transfer a given `token_id` from current owner to
    /// `receiver_id`.
    ///
    /// Requirements
    /// * Caller of the method must attach a deposit of 1 yoctoⓃ for security purposes
    /// * Contract MUST panic if called by someone other than token owner or,
    ///   if using Approval Management, one of the approved accounts
    /// * `approval_id` is for use with Approval Management extension, see
    ///   that document for full explanation.
    /// * If using Approval Management, contract MUST nullify approved accounts on
    ///   successful transfer.
    ///
    /// Arguments:
    /// * `receiver_id`: the valid NEAR account receiving the token
    /// * `token_id`: the token to transfer
    /// * `approval_id`: expected approval ID. A number smaller than
    ///    2^53, and therefore representable as JSON. See Approval Management
    ///    standard for full explanation.
    /// * `memo` (optional): for use cases that may benefit from indexing or
    ///    providing information for a transfer
    //#[payable]
    fn nft_transfer(
        &mut self,
        receiver_id: AccountId,
        token_id: U64,
        approval_id: Option<U64>,
        memo: Option<String>,
    );

    /// Returns `true` if the token was transferred from the sender's account.
    ///
    /// Transfer token and call a method on a receiver contract. A successful
    /// workflow will end in a success execution outcome to the callback on the NFT
    /// contract at the method `nft_resolve_transfer`.
    ///
    /// You can think of this as being similar to attaching native NEAR tokens to a
    /// function call. It allows you to attach any Non-Fungible Token in a call to a
    /// receiver contract.
    ///
    /// Requirements:
    /// * Caller of the method must attach a deposit of 1 yoctoⓃ for security
    ///   purposes
    /// * Contract MUST panic if called by someone other than token owner or,
    ///   if using Approval Management, one of the approved accounts
    /// * The receiving contract must implement `ft_on_transfer` according to the
    ///   standard. If it does not, FT contract's `ft_resolve_transfer` MUST deal
    ///   with the resulting failed cross-contract call and roll back the transfer.
    /// * Contract MUST implement the behavior described in `ft_resolve_transfer`
    /// * `approval_id` is for use with Approval Management extension, see
    ///   that document for full explanation.
    /// * If using Approval Management, contract MUST nullify approved accounts on
    ///   successful transfer.
    ///
    /// Arguments:
    /// * `receiver_id`: the valid NEAR account receiving the token.
    /// * `token_id`: the token to send.
    /// * `approval_id`: expected approval ID. A number smaller than
    ///    2^53, and therefore representable as JSON. See Approval Management
    ///    standard for full explanation.
    /// * `memo` (optional): for use cases that may benefit from indexing or
    ///    providing information for a transfer.
    /// * `msg`: specifies information needed by the receiving contract in
    ///    order to properly handle the transfer. Can indicate both a function to
    ///    call and the parameters to pass to that function.
    //#[payable]
    fn nft_transfer_call(
        &mut self,
        receiver_id: AccountId,
        token_id: U64,
        approval_id: Option<U64>,
        memo: Option<String>,
        msg: String,
    ) -> Promise;

    /// Returns the token with the given `token_id` or `None` if no such token.
    fn nft_token(
        &self,
        token_id: U64,
    ) -> Option<Token>;
}
