use std::collections::HashMap;
use std::fmt;

use near_sdk::borsh::{
    self,
    BorshDeserialize,
    BorshSerialize,
};
use near_sdk::serde::ser::Serializer;
use near_sdk::serde::{
    Deserialize,
    Serialize,
};
use near_sdk::AccountId;

use crate::common::{
    Royalty,
    SplitOwners,
    TokenKey,
    TokenMetadataCompliant,
};

/// Supports NEP-171, 177, 178, 181. Ref:
/// https://github.com/near/NEPs/blob/master/specs/Standards/NonFungibleToken/Core.md
#[derive(Clone)]
#[cfg_attr(feature = "wasm", derive(BorshDeserialize, BorshSerialize))]
#[derive(Deserialize, Serialize)]
pub struct Token {
    /// The id of this token on this `Store`. Not unique across `Store`s.
    /// `token_id`s count up from 0. Ref: https://github.com/near/NEPs/discussions/171
    pub id: u64,
    /// The current owner of this token. Either an account_id or a token_id (if composed).
    pub owner_id: Owner,
    /// Ref:
    /// https://github.com/near/NEPs/blob/master/specs/Standards/NonFungibleToken/ApprovalManagement.md
    /// Set of accounts that may transfer this token, other than the owner.
    pub approvals: HashMap<AccountId, u64>,
    /// The metadata content for this token is stored in the Contract
    /// `token_metadata` field, to avoid duplication of metadata across tokens.
    /// Use metadata_id to lookup the metadata. `Metadata`s is permanently set
    /// when the token is minted.
    pub metadata_id: u64,
    /// The Royalty for this token is stored in the Contract `token_royalty`
    /// field, to avoid duplication across tokens. Use royalty_id to lookup the
    /// royalty. `Royalty`s are permanently set when the token is minted.
    pub royalty_id: Option<u64>,
    /// Feature for owner of this token to split the token ownership accross
    /// several accounts.
    pub split_owners: Option<SplitOwners>,
    /// The account that minted this token.
    pub minter: AccountId,
    /// Non-nil if Token is loaned out. While token is loaned, disallow
    /// transfers, approvals, revokes, etc. for the token, except from the
    /// approved loan contract. Mark this field with the address of the loan
    /// contract. See neps::loan for more.
    pub loan: Option<Loan>,
    /// Composeablility metrics for this token
    pub composeable_stats: ComposeableStats,
    /// If the token originated on another contract and was `nft_move`d to
    /// this contract, this field will be non-nil.
    pub origin_key: Option<TokenKey>,
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
        self.is_owned_by(&near_sdk::env::predecessor_account_id())
    }

    pub fn is_owned_by(
        &self,
        account_id: &AccountId,
    ) -> bool {
        self.owner_id.to_string() == account_id.to_string()
    }

    pub fn is_loaned(&self) -> bool {
        self.loan.is_some()
    }
}

// Supports NEP-171, 177, 178, 181. Ref:
/// https://github.com/near/NEPs/blob/master/specs/Standards/NonFungibleToken/Core.md
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TokenCompliant {
    /// The id of this token on this `Store`. Not unique across `Store`s.
    /// `token_id`s count up from 0. Ref: https://github.com/near/NEPs/discussions/171
    pub token_id: String,
    /// The current owner of this token. Either an account_id or a token_id (if composed).
    pub owner_id: Owner,
    /// Ref:
    /// https://github.com/near/NEPs/blob/master/specs/Standards/NonFungibleToken/ApprovalManagement.md
    /// Set of accounts that may transfer this token, other than the owner.
    pub approved_account_ids: HashMap<AccountId, u64>,
    /// The metadata content for this token is stored in the Contract
    /// `token_metadata` field, to avoid duplication of metadata across tokens.
    /// Use metadata_id to lookup the metadata. `Metadata`s is permanently set
    /// when the token is minted.
    pub metadata: TokenMetadataCompliant,
    /// The Royalty for this token is stored in the Contract `token_royalty`
    /// field, to avoid duplication across tokens. Use royalty_id to lookup the
    /// royalty. `Royalty`s are permanently set when the token is minted.
    pub royalty: Option<Royalty>,
    /// Feature for owner of this token to split the token ownership accross
    /// several accounts.
    pub split_owners: Option<SplitOwners>,
    /// The account that minted this token.
    pub minter: AccountId,
    /// Non-nil if Token is loaned out. While token is loaned, disallow
    /// transfers, approvals, revokes, etc. for the token, except from the
    /// approved loan contract. Mark this field with the address of the loan
    /// contract. See neps::loan for more.
    pub loan: Option<Loan>,
    /// Composeablility metrics for this token
    pub composeable_stats: ComposeableStats,
    /// If the token originated on another contract and was `nft_move`d to
    /// this contract, this field will be non-nil.
    pub origin_key: Option<TokenKey>,
}

// --------------------------------- owner ---------------------------------- //
// This is mostly kept here to avoid storage migrations, but this should always
// be the `Account` variant.
#[cfg_attr(feature = "wasm", derive(BorshDeserialize, BorshSerialize))]
#[derive(Deserialize, Clone, Debug)]
pub enum Owner {
    /// Standard pattern: owned by a user.
    Account(AccountId),
    /// Compose pattern: owned by a token on this contract.
    TokenId(u64),
    /// Cross-compose pattern: owned by a token on another contract.
    CrossKey(crate::common::TokenKey),
    /// Lock: temporarily locked until some callback returns.
    Lock(AccountId),
}

impl Serialize for Owner {
    fn serialize<S: Serializer>(
        &self,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        // TODO: create string and then clone?
        serializer.serialize_str(&format!("{}", self))
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

// ---------------------------------- loan ---------------------------------- //
// This is only kept here to avoid storage migrations, it is no longer used
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "wasm", derive(BorshDeserialize, BorshSerialize))]
pub struct Loan {
    pub holder: AccountId,
    pub loan_contract: AccountId,
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

// ----------------------------- composability ------------------------------ //
// This is only kept here to avoid storage migrations, it is no longer used
/// To enable recursive composeability, need to track:
/// 1. How many levels deep a token is recursively composed
/// 2. Whether and how many cross-contract children a token has.
///
/// Tracking depth limits potential bugs around recursive ownership
/// consuming excessive amounts of gas.
///
/// Tracking the number of cross-contract children a token has prevents
/// breaking of the Only-One-Cross-Linkage Invariant.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "wasm", derive(BorshDeserialize, BorshSerialize))]
pub struct ComposeableStats {
    /// How deep this token is in a chain of composeability on THIS contract.
    /// If this token is cross-composed, it's depth will STILL be 0. `depth`
    /// equal to the parent's `depth`+1. If this is a top level token, this
    /// number is 0.
    pub local_depth: u8,
    /// How many cross contract children this token has, direct AND indirect.
    /// That is, any parent's `cross_contract_children` value equals the sum
    /// of of its children's values. If this number is non-zero, deny calls
    /// to `nft_cross_compose`.
    pub cross_contract_children: u8,
}

impl ComposeableStats {
    pub(super) fn new() -> Self {
        Self {
            local_depth: 0,
            cross_contract_children: 0,
        }
    }
}
