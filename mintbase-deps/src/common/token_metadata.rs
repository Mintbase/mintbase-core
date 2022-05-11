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

impl TokenMetadata {
    /// Get the metadata and its size in bytes.
    pub fn from_with_size(
        args: TokenMetadata,
        copies: u64,
    ) -> (Self, u64) {
        // if args.media.is_some() {
        //     crate::near_assert!(
        //         args.media_hash.is_some(),
        //         "Cannot specificy metadata.media without metadata.media_hash"
        //     );
        // }

        // if args.reference.is_some() {
        //     crate::near_assert!(
        //         args.reference_hash.is_some(),
        //         "Cannot specificy metadata.reference without metadata.reference_hash"
        //     );
        // }

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
