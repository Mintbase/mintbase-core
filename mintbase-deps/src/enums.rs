#[cfg(feature = "wasm")]
use near_sdk::{
    borsh::{
        self,
        BorshDeserialize,
        BorshSerialize,
    },
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

pub type DynamicError = Box<dyn std::error::Error>;
