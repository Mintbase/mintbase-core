use std::collections::HashMap;

use near_sdk::json_types::{
    Base64VecU8,
    U128,
};
use near_sdk::{
    AccountId,
    *,
};
use serde::*;

use crate::*;

pub type SplitBetweenUnparsed = HashMap<near_sdk::AccountId, u32>;
#[cfg(feature = "wasm")]
use near_sdk::borsh::{
    self,
    BorshDeserialize,
    BorshSerialize,
};

/// Take the Royalty and SplitOwner information for a token, and return a Vector
/// of proportional payouts.
#[derive(Serialize, Deserialize)]
pub struct OwnershipFractions {
    pub fractions: HashMap<AccountId, MultipliedSafeFraction>,
}

/// Whom to pay. Generated from `OwnershipFractions`.
#[derive(Serialize, Deserialize)]
pub struct Payout {
    pub payout: HashMap<AccountId, U128>,
}

/// A representation of the splitting of ownership of the Token. Percentages
/// must add to 1. On purchase of the `Token`, the value of the transaction
/// (minus royalty percentage) will be paid out to each account in `SplitOwners`
/// mapping. The `SplitOwner` field on the `Token` will be set to `None` after
/// each transfer of the token.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "wasm", derive(BorshDeserialize, BorshSerialize))]
pub struct SplitOwners {
    pub split_between: HashMap<AccountId, SafeFraction>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "wasm", derive(BorshDeserialize, BorshSerialize))]
pub struct Loan {
    pub holder: AccountId,
    pub loan_contract: AccountId,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "wasm", derive(BorshDeserialize, BorshSerialize))]
pub struct TokenKey {
    pub token_id: u64,
    pub account_id: String,
}

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

/// A SafeFraction that has been multiplied with another SafeFraction. Denominator is 10^8.
#[cfg_attr(feature = "wasm", derive(BorshDeserialize, BorshSerialize))]
#[derive(Clone, Debug, Deserialize, Serialize, Copy)]
pub struct MultipliedSafeFraction {
    pub numerator: u32,
}

/// A provisional safe fraction type, borrowed and modified from:
/// https://github.com/near/core-contracts/blob/master/staking-pool/src/lib.rs#L127
/// The numerator is a value between 0 and 10,000. The denominator is
/// assumed to be 10,000.
#[derive(Debug, Clone, PartialEq, Copy)]
#[cfg_attr(feature = "wasm", derive(BorshDeserialize, BorshSerialize))]
#[derive(Deserialize, Serialize)]
pub struct SafeFraction {
    pub numerator: u32,
}

// NON-COMPLIANT https://github.com/near/NEPs/blob/master/specs/Standards/NonFungibleToken/Metadata.md
/// ref:
/// https://github.com/near/NEPs/blob/master/specs/Standards/NonFungibleToken/Metadata.md
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "wasm", derive(BorshDeserialize, BorshSerialize))]
pub struct TokenMetadata {
    /// the Title for this token. ex. "Arch Nemesis: Mail Carrier" or "Parcel 5055"
    pub title: Option<String>,
    /// free-form description of this token.
    pub description: Option<String>,
    /// URL to associated media, preferably to decentralized, content-addressed storage
    pub media: Option<String>,
    /// Base64-encoded sha256 hash of content referenced by the `media` field.
    /// Required if `media` is included.
    pub media_hash: Option<Base64VecU8>,
    /// number of copies of this set of metadata in existence when token was minted.
    pub copies: Option<u16>,
    /// ISO 8601 datetime when token expires.
    pub expires_at: Option<String>,
    /// ISO 8601 datetime when token starts being valid.
    pub starts_at: Option<String>,
    /// When token was last updated, Unix epoch in milliseconds
    pub extra: Option<String>,
    /// URL to an off-chain JSON file with more info. The Mintbase Indexer refers
    /// to this field as `thing_id` or sometimes, `meta_id`.
    pub reference: Option<String>,
    /// Base64-encoded sha256 hash of JSON from reference field. Required if
    /// `reference` is included.
    pub reference_hash: Option<Base64VecU8>,
}

/// A representation of permanent partial ownership of a Token's revenues.
/// Percentages must add to 10,000. On purchase of the `Token`, a percentage of
/// the value of the transaction will be paid out to each account in the
/// `Royalty` mapping. `Royalty` field once set can NEVER change for this
/// `Token`, even if removed and re-added.
#[derive(PartialEq)]
#[cfg_attr(feature = "wasm", derive(BorshDeserialize, BorshSerialize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Royalty {
    /// Mapping of addresses to relative percentages of the overall royalty percentage
    pub split_between: HashMap<near_sdk::AccountId, SafeFraction>,
    /// The overall royalty percentage taken
    pub percentage: SafeFraction,
}

/// Unparsed pre-image of a Royalty struct. Used in `Store::mint_tokens`.
#[derive(Clone, Deserialize, Serialize)]
pub struct RoyaltyArgs {
    pub split_between: SplitBetweenUnparsed,
    pub percentage: u32,
}

// Supports NEP-171, 177, 178, 181. Ref:
/// https://github.com/near/NEPs/blob/master/specs/Standards/NonFungibleToken/Core.md
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TokenCompliant {
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

#[derive(Clone, Debug)]
#[cfg_attr(feature = "all", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "wasm", derive(BorshDeserialize, BorshSerialize))]
pub struct StorageCosts {
    /// The Near-denominated price-per-byte of storage. As of April 2021, the
    /// price per bytes is set by default to 10^19, but this may change in the
    /// future, thus this future-proofing field.
    pub storage_price_per_byte: u128,
    /// 80 bytes as a Near price. Used for:
    /// - a single royalty
    /// - a single approval
    /// - adding a new entry to the `tokens_per_account` map
    /// - adding a new entry to the `composeables` map
    pub common: u128,
    pub token: u128,
}

#[cfg_attr(feature = "market-wasm", derive(BorshDeserialize, BorshSerialize))]
pub struct StorageCostsMarket {
    /// The Near-denominated price-per-byte of storage. As of April 2021, the
    /// price per bytes is set by default to 10^19, but this may change in the
    /// future, thus this future-proofing field.
    pub storage_price_per_byte: u128,
    pub list: u128,
}

// NON-COMPLIANT https://github.com/near/NEPs/blob/master/specs/Standards/NonFungibleToken/Metadata.md
/// ref:
/// https://github.com/near/NEPs/blob/master/specs/Standards/NonFungibleToken/Metadata.md
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TokenMetadataCompliant {
    /// the Title for this token. ex. "Arch Nemesis: Mail Carrier" or "Parcel 5055"
    pub title: Option<String>,
    /// free-form description of this token.
    pub description: Option<String>,
    /// URL to associated media, preferably to decentralized, content-addressed storage
    pub media: Option<String>,
    /// Base64-encoded sha256 hash of content referenced by the `media` field.
    /// Required if `media` is included.
    pub media_hash: Option<Base64VecU8>,
    /// number of copies of this set of metadata in existence when token was minted.
    pub copies: Option<u16>,
    /// When token was issued or minted, Unix epoch in milliseconds
    pub issued_at: Option<String>,
    /// ISO 8601 datetime when token expires.
    pub expires_at: Option<String>,
    /// ISO 8601 datetime when token starts being valid.
    pub starts_at: Option<String>,
    /// When token was last updated, Unix epoch in milliseconds
    pub updated_at: Option<String>,
    /// Brief description of what this thing is. Used by the mintbase indexer as "memo".
    pub extra: Option<String>,
    /// URL to an off-chain JSON file with more info. The Mintbase Indexer refers
    /// to this field as `thing_id` or sometimes, `meta_id`.
    pub reference: Option<String>,
    /// Base64-encoded sha256 hash of JSON from reference field. Required if
    /// `reference` is included.
    pub reference_hash: Option<Base64VecU8>,
}

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

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "wasm", derive(BorshDeserialize, BorshSerialize))]
pub struct StoreInitArgs {
    pub metadata: NFTContractMetadata,
    pub owner_id: AccountId,
}

#[cfg(feature = "all")]
pub struct MyMakeWriter {
    pub stdout: std::io::Stdout,
    pub stderr: std::io::Stderr,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "wasm", derive(BorshDeserialize, BorshSerialize))]
pub struct NFTContractMetadata {
    /// a version like "nft-1.0.0"
    pub spec: String,
    /// Subaccount of this `Store`. `Factory` is the super-account.
    pub name: String,
    /// Symbol of the Store. Up to 6 chars.
    pub symbol: String,
    /// a small image associated with this `Store`.
    pub icon: Option<String>,
    /// Centralized gateway known to have reliable access to decentralized storage
    /// assets referenced by `reference` or `media` URLs
    pub base_uri: Option<String>,
    /// URL to a JSON file with more info
    pub reference: Option<String>,
    /// Base64-encoded sha256 hash of the JSON file pointed at by the reference
    /// field. Required if `reference` is included.
    pub reference_hash: Option<Base64VecU8>,
}

#[derive(Debug, Clone)]
pub struct NftEventError(pub String);

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "wasm", derive(BorshDeserialize, BorshSerialize))]
pub struct NearTime(pub u64);

/// ref: https://github.com/near-apps/nft-market/blob/main/contracts/market-simple/src/lib.rs#L54
#[derive(Serialize, Deserialize)]
pub struct SaleArgs {
    pub price: U128,
    pub autotransfer: bool,
}

/// Type representing an offer for a `Token` the marketplace
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "wasm", derive(BorshDeserialize, BorshSerialize))]
pub struct TokenOffer {
    /// The id of this `Offer` is the num of the previous `Offer` + 1. Generated
    /// from the field `Token::num_offers`.
    pub id: u64,
    /// The price the Offerer has posted.
    pub price: u128,
    /// The account who originated the `Offer`.
    pub from: AccountId,
    /// When the `Offer` was made.
    pub timestamp: NearTime,
    /// When the `Offer` will expire.
    pub timeout: NearTime,
}

/// A Token for sale on the Marketplace.
#[derive(Deserialize, Serialize, Debug)]
#[cfg_attr(feature = "wasm", derive(BorshDeserialize, BorshSerialize))]
pub struct TokenListing {
    /// Id of this `Token`.
    pub id: u64,
    /// Owner of this `Token`.
    pub owner_id: AccountId,
    /// `Store` that originated this `Token`.
    pub store_id: AccountId,
    /// If `autotransfer` is enabled, the Token will automatically be
    /// transferred to an Offerer if their `Offer::price` is greater than the
    /// `asking_price`. Note that enabling `autotransfer` does not
    /// retroactively trigger on the presently held `current_offer`
    pub autotransfer: bool,
    /// The price set by the owner of this Token.
    pub asking_price: U128,
    /// The `approval_id` of the Token allows the Marketplace to transfer the
    /// Token, if purchased. The `approval_id` is also used to generate
    /// unique identifiers for Token-listings.
    pub approval_id: u64,
    /// The current `Offer` for this listing. This `Offer` may have timed
    /// out; if the `Marketplace::min_offer_hours` has transpired, the
    /// `Offer` may be withdrawn by the account in `Offer::from`.
    pub current_offer: Option<TokenOffer>,
    /// The number of `Offer`s that have been made on this listing. Used to
    /// generate Offer `id`s.
    pub num_offers: u64,
    /// When the transfer process is initiated, the token is locked, and no
    /// further changes may be made on the token.
    pub locked: bool,
}
