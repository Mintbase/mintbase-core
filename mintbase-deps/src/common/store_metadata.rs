use near_contract_standards::non_fungible_token::metadata;
use near_sdk::borsh::{
    self,
    BorshDeserialize,
    BorshSerialize,
};
use near_sdk::json_types::Base64VecU8;
use near_sdk::serde::{
    Deserialize,
    Serialize,
};

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

impl NFTContractMetadata {
    pub fn to_standardized(&self) -> metadata::NFTContractMetadata {
        metadata::NFTContractMetadata {
            spec: self.spec.clone(),
            name: self.name.clone(),
            symbol: self.symbol.clone(),
            icon: self.icon.clone(),
            base_uri: self.base_uri.clone(),
            reference: self.reference.clone(),
            reference_hash: self.reference_hash.clone(),
        }
    }
}

/// ref:
/// https://github.com/near/NEPs/blob/master/specs/Standards/NonFungibleToken/Metadata.md
pub trait NonFungibleContractMetadata {
    /// Get the metadata for this `Store`.
    fn nft_metadata(&self) -> &NFTContractMetadata;
}
