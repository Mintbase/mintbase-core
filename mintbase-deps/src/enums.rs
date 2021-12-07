use crate::*;
#[cfg(feature = "wasm")]
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{
    env,
    json_types::Base64VecU8,
    serde::{Deserialize, Serialize},
    AccountId,
};
use serde::*;

/// This enum used to support other time denominations, which were dropped
/// for simplicity.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature="wasm",derive(BorshSerialize, BorshDeserialize,))]
pub enum TimeUnit {
    Hours(u64),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "event", content = "data")]
#[serde(rename_all = "snake_case")]
pub enum Nep171EventLog{
    NftMint(Vec<NftMintLog>),
    NftBurn(Vec<NftBurnLog>),
    NftTransfer(Vec<NftTransferLog>)
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(untagged)]
pub enum NftEvent {
    NftCreateStore(NftStoreCreateLog),
    NftStringEvent(NftStringLog),
    NftCreate(Vec<NftMintLog>),
    NftDelete(Vec<NftBurnLog>),
    NftCreateApproval(Vec<NftApproveLog>),
    NftRevoke(NftRevokeLog),
    NftUpdate(Vec<NftTransferLog>),
    NftUpdateSplitOwner(NftSetSplitOwnerLog),
    NftUpdateLoan(NftLoanSetLog),
    NftCreateCompose(NftComposeLog),
    NftDeleteCompose(NftUncomposeLog),
    NftOnCreateCompose(NftOnComposeLog),
    NftOnDeleteCompose(NftOnUncomposeLog),
    NftOnMove(NftOnMoveLog),
    NftMoved(NftMovedLog),
    NftCreateList(Vec<NftListLog>),
    NftCreateOffer(NftOfferLog),
    NftUpdateOffer(NftUpdateOfferLog),
    NftCreateSale(NftSaleLog),
    NftUpdateMarket(NftMarketLog),
    NftUpdateIcon(NftOptionStringLog),
    NftUpdateList(NftUpdateListLog),
}

#[cfg_attr(feature = "wasm", derive(BorshDeserialize, BorshSerialize))]
#[derive(Deserialize, Serialize, Clone, Debug)]
pub enum Owner {
    /// Standard pattern: owned by a user.
    Account(AccountId),
    /// Compose pattern: owned by a token on this contract.
    TokenId(u64),
    /// Cross-compose pattern: owned by a token on another contract.
    CrossKey(TokenKey),
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