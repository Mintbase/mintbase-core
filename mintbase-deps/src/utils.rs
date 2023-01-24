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

// ---------------------------- assertion macros ---------------------------- //
#[macro_export]
macro_rules! near_panic {
    ($msg:literal) => {
        near_sdk::env::panic_str($msg)
    };

    ($msg:literal, $($arg:expr),*) => {
        near_sdk::env::panic_str(format!($msg, $($arg),*).as_str())
    };
}

#[macro_export]
macro_rules! near_assert {
    ($predicate:expr, $msg:literal) => {
        if !$predicate {
            $crate::near_panic!($msg)
        }
    };

    ($predicate:expr, $msg:literal, $($arg:expr),*) => {
        if !$predicate {
            $crate::near_panic!($msg, $($arg),*)
        }
    };
}

#[macro_export]
macro_rules! assert_token_owned_by {
    ($token:expr, $account:expr) => {
        if !$token.is_owned_by($account) {
            $crate::near_panic!(
                "{} is required to own token {} ({}, {}:{})",
                $account,
                $token.id,
                file!(),
                line!(),
                column!()
            );
        }
    };
}

#[macro_export]
macro_rules! assert_token_owned_by_predecessor {
    ($token:expr) => {
        $crate::assert_token_owned_by!($token, &$crate::near_sdk::env::predecessor_account_id())
    };
}

#[macro_export]
macro_rules! assert_token_owned_or_approved {
    ($token:expr, $account:expr, $approval_id:expr) => {
        if !$token.is_owned_by($account) {
            let src = format!("{}, {}:{}", file!(), line!(), column!());
            match ($token.approvals.get($account), $approval_id) {
                (_, None) => {
                    $crate::near_panic!("Disallowing approvals without approval ID! ({})", src)
                },
                (None, _) => {
                    $crate::near_panic!(
                        "{} has no approval for token {} ({})",
                        $account,
                        $token.id,
                        src
                    )
                },
                (Some(a), Some(b)) if *a != b => {
                    $crate::near_panic!(
                        "The current approval ID is {}, but {} has been provided ({})",
                        a,
                        b,
                        src
                    )
                },
                _ => { /* everything ok */ },
            }
        }
    };
}

#[macro_export]
macro_rules! assert_token_unloaned {
    ($token:expr) => {
        if $token.is_loaned() {
            $crate::near_panic!(
                "Token {} must not be loaned ({}, {}:{})",
                $token.id,
                file!(),
                line!(),
                column!()
            );
        }
    };
}

#[macro_export]
macro_rules! assert_storage_deposit {
    ($required:expr) => {
        if env::attached_deposit() < $required {
            $crate::near_panic!(
                "Requires storage deposit of at least {} yoctoNEAR ({}, {}:{})",
                $required,
                file!(),
                line!(),
                column!()
            );
        }
    };
}
