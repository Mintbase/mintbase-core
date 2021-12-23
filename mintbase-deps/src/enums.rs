#[cfg(feature = "wasm")]
use near_sdk::{
    borsh::{
        self,
        BorshDeserialize,
        BorshSerialize,
    },
    env,
    json_types::Base64VecU8,
    serde::{
        Deserialize,
        Serialize,
    },
    AccountId,
};

/// This enum used to support other time denominations, which were dropped
/// for simplicity.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "wasm", derive(BorshSerialize, BorshDeserialize,))]
pub enum TimeUnit {
    Hours(u64),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "event", content = "data")]
#[serde(rename_all = "snake_case")]
pub enum Nep171EventLog {
    NftMint(Vec<crate::NftMintLog>),
    NftBurn(Vec<crate::NftBurnLog>),
    NftTransfer(Vec<crate::NftTransferLog>),
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(untagged)]
pub enum NftEvent {
    NftCreateStore(crate::NftStoreCreateLog),
    NftStringEvent(crate::NftStringLog),
    NftCreate(Vec<crate::NftMintLog>),
    NftDelete(Vec<crate::NftBurnLog>),
    NftCreateApproval(Vec<crate::NftApproveLog>),
    NftRevoke(crate::NftRevokeLog),
    NftUpdate(Vec<crate::NftTransferLog>),
    NftUpdateSplitOwner(crate::NftSetSplitOwnerLog),
    NftUpdateLoan(crate::NftLoanSetLog),
    NftCreateCompose(crate::NftComposeLog),
    NftDeleteCompose(crate::NftUncomposeLog),
    NftOnCreateCompose(crate::NftOnComposeLog),
    NftOnDeleteCompose(crate::NftOnUncomposeLog),
    NftOnMove(crate::NftOnMoveLog),
    NftMoved(crate::NftMovedLog),
    NftCreateList(Vec<crate::NftListLog>),
    NftCreateOffer(crate::NftOfferLog),
    NftUpdateOffer(crate::NftUpdateOfferLog),
    NftCreateSale(crate::NftSaleLog),
    NftUpdateMarket(crate::NftMarketLog),
    NftUpdateIcon(crate::NftOptionStringLog),
    NftUpdateList(crate::NftUpdateListLog),
}

#[cfg_attr(feature = "wasm", derive(BorshDeserialize, BorshSerialize))]
#[derive(Deserialize, Serialize, Clone, Debug)]
pub enum Owner {
    /// Standard pattern: owned by a user.
    Account(AccountId),
    /// Compose pattern: owned by a token on this contract.
    TokenId(u64),
    /// Cross-compose pattern: owned by a token on another contract.
    CrossKey(crate::TokenKey),
    /// Lock: temporarily locked until some callback returns.
    Lock(AccountId),
}

#[cfg(feature = "all")]
pub enum StdioLock<'a> {
    Stdout(std::io::StdoutLock<'a>),
    Stderr(std::io::StderrLock<'a>),
}

// #[cfg(feature = "all")]
// pub enum AppError {
//     IndexerError,
// }

pub type DynamicError = Box<dyn std::error::Error>;
