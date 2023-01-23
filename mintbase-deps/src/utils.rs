use near_sdk::borsh::{
    self,
    BorshDeserialize,
    BorshSerialize,
};
use near_sdk::serde::{
    Deserialize,
    Serialize,
};
use near_sdk::Balance;

/// Split a &str on the first colon
pub fn split_colon(string: &str) -> (&str, &str) {
    let pos = string.find(':').expect("no colon");
    (&string[..pos], &string[(pos + 1)..])
}

/// Near denominated units are in 10^24
#[cfg(feature = "market-wasm")]
pub const fn ntoy(near_amount: Balance) -> Balance {
    near_amount * 10u128.pow(24)
}

/// A provisional safe fraction type, borrowed and modified from:
/// https://github.com/near/core-contracts/blob/master/staking-pool/src/lib.rs#L127
/// The numerator is a value between 0 and 10,000. The denominator is
/// assumed to be 10,000.
#[derive(Debug, Clone, PartialEq, Eq, Copy)]
#[cfg_attr(feature = "wasm", derive(BorshDeserialize, BorshSerialize))]
#[derive(Deserialize, Serialize)]
pub struct SafeFraction {
    pub numerator: u32,
}

impl SafeFraction {
    /// Take a u32 numerator to a 10^4 denominator.
    ///
    /// Upper limit is 10^4 so as to prevent multiplication with overflow.
    pub fn new(numerator: u32) -> Self {
        crate::near_assert!(
            (0..=10000).contains(&numerator),
            "{} must be between 0 and 10_000",
            numerator
        );
        SafeFraction { numerator }
    }

    /// Fractionalize a balance.
    pub fn multiply_balance(
        &self,
        value: Balance,
    ) -> Balance {
        value / 10_000u128 * self.numerator as u128
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "wasm", derive(BorshDeserialize, BorshSerialize))]
pub struct TokenKey {
    pub token_id: u64,
    pub account_id: String,
}

impl std::fmt::Display for TokenKey {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter,
    ) -> std::fmt::Result {
        write!(f, "{}:{}", self.token_id, self.account_id)
    }
}

impl From<&str> for TokenKey {
    fn from(s: &str) -> Self {
        let (id, account_id) = split_colon(s);
        Self {
            token_id: id.parse::<u64>().unwrap(),
            account_id: account_id.to_string(),
        }
    }
}
