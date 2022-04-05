pub mod asserts;
pub mod common;
pub mod constants;
pub mod interfaces;
pub mod logging;
pub mod token;
pub mod utils;

// ----------------- re-exports for consistent dependencies ----------------- //
pub use near_sdk::{
    self,
    serde,
    serde_json,
};

// TODO: move module resolution to indexer
#[cfg(feature = "all")]
pub use crate::logging::{
    NearJsonEvent,
    Nep171Event,
    Nep171EventLog,
    NftApproveLog,
    NftBurnLog,
    NftComposeLog,
    NftListLog,
    NftLoanSetLog,
    NftMarketLog,
    NftMintLog,
    NftMintLogMemo,
    NftOfferLog2,
    NftOptionStringLog,
    NftRevokeLog,
    NftSaleLog,
    NftSetSplitOwnerLog,
    NftStoreCreateLog,
    NftStringLog,
    NftTransferLog,
    NftUpdateListLog,
    NftUpdateOfferLog,
};
