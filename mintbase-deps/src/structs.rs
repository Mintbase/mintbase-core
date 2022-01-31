use std::collections::HashMap;

use near_sdk::json_types::{
    Base64VecU8,
    U128,
    U64,
};
use near_sdk::{
    AccountId,
    *,
};
use serde::*;

use crate::*;

// #[cfg(test)]
// use clap::*;

// #[cfg(test)]
// #[derive(Clap, Debug, Clone)]
// #[cfg(test)]
// pub struct CallCmd {
//     #[clap(short)]
//     pub func: String,
//     #[clap(long,short)]
//     pub args: String,
//     #[clap(long,short,default_value = "mintbase.test.near")]
//     pub r: String,
// }

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

#[cfg_attr(
    feature = "helper-wasm",
    near_sdk::near_bindgen,
    derive(BorshDeserialize, BorshSerialize)
)]
#[cfg(feature = "helper-wasm")]
pub struct HelperWasm {
    pub count: u64,
}

#[cfg_attr(
    feature = "store-wasm",
    near_sdk::near_bindgen,
    derive(BorshDeserialize, BorshSerialize)
)]
#[cfg(feature = "store-wasm")]
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

#[cfg(feature = "all")]
pub struct MyMakeWriter {
    pub stdout: std::io::Stdout,
    pub stderr: std::io::Stderr,
}

#[cfg(feature = "factory-wasm")]
#[near_bindgen]
#[cfg(feature = "factory-wasm")]
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

#[derive(Serialize, Deserialize, Debug)]
pub struct Nep171Event {
    pub standard: String,
    pub version: String,
    #[serde(flatten)]
    pub event_kind: Nep171EventLog,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NearJsonEvent {
    pub standard: String,
    pub version: String,
    pub event: String,
    pub data: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NftStoreCreateLog {
    pub contract_metadata: NFTContractMetadata,
    pub owner_id: String,
    pub id: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NftStringLog {
    pub data: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NftOptionStringLog {
    pub data: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NftMintLog {
    pub owner_id: String,
    pub token_ids: Vec<String>,
    pub memo: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NftBurnLog {
    pub owner_id: String,
    pub authorized_id: Option<String>,
    pub token_ids: Vec<String>,
    pub memo: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NftApproveLog {
    pub token_id: u64,
    pub approval_id: u64,
    pub account_id: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NftRevokeLog {
    pub token_id: u64,
    pub account_id: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NftTransferLog {
    pub authorized_id: Option<String>,
    pub old_owner_id: String,
    pub new_owner_id: String,
    pub token_ids: Vec<String>,
    pub memo: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NftSetSplitOwnerLog {
    pub split_owners: SplitOwners,
    pub token_ids: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NftLoanSetLog {
    pub account_id: Option<String>,
    pub token_id: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NftComposeLog {
    pub token_ids: Vec<U64>,
    // direct parent of token_ids
    pub parent: String,
    // - "t": owned directly by a token on this contract
    // - "k": owned directly by a token on another contract
    pub ttype: String,
    // local root of chain of token_ids
    pub lroot: Option<u64>,
    // holder of local root
    pub holder: String,
    pub depth: u8,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NftUncomposeLog {
    pub token_ids: Vec<U64>,
    pub holder: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NftOnComposeLog {
    pub predecessor: String,
    pub token_id: U64,
    // direct parent of token_ids
    pub cross_child_id: U64,
    // local root of chain of token_ids
    pub lroot: Option<u64>,
    // holder of local root
    pub holder: String,
    pub depth: u8,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NftOnUncomposeLog {
    pub token_id: U64,
    pub holder: String,
    pub child_key: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NftMovedLog {
    pub token_id: U64,
    pub contract_id: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NftOnMoveLog {
    pub token_id: U64,
    pub origin_key: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NftListLog {
    pub list_id: String,
    pub price: String,
    pub token_key: String,
    pub owner_id: String,
    pub autotransfer: bool,
    pub approval_id: String,
    pub token_id: String,
    pub store_id: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NftMintLogMemo {
    pub royalty: Option<Royalty>,
    pub split_owners: Option<SplitOwners>,
    pub meta_id: Option<String>,
    pub meta_extra: Option<String>,
    pub minter: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NftUpdateListLog {
    pub auto_transfer: Option<bool>,
    pub price: Option<String>,
    pub list_id: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NftOfferLog2 {
    pub offer: TokenOffer,
    pub list_id: String,
    pub token_key: String,
    pub offer_num: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NftOfferLog {
    pub price: String,
    pub from: String,
    pub timeout: String,
    pub list_id: String,
    pub token_key: String,
    pub offer_num: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NftUpdateOfferLog {
    pub list_id: String,
    pub offer_num: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NftSaleLog {
    pub list_id: String,
    pub offer_num: u64,
    pub token_key: String,
    pub payout: HashMap<AccountId, U128>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NftMarketLog {
    pub account_id: String,
    pub state: bool,
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
