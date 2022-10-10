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

/// A SafeFraction that has been multiplied with another SafeFraction. Denominator is 10^8.
#[cfg_attr(feature = "wasm", derive(BorshDeserialize, BorshSerialize))]
#[derive(Clone, Debug, Deserialize, Serialize, Copy)]
pub struct MultipliedSafeFraction {
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

impl std::ops::Sub for SafeFraction {
    type Output = SafeFraction;

    fn sub(
        self,
        rhs: Self,
    ) -> Self::Output {
        crate::near_assert!(
            self.numerator >= rhs.numerator,
            "Subtraction result cannot be negative"
        );
        Self {
            numerator: self.numerator - rhs.numerator,
        }
    }
}

impl std::ops::SubAssign for SafeFraction {
    fn sub_assign(
        &mut self,
        rhs: Self,
    ) {
        crate::near_assert!(
            self.numerator >= rhs.numerator,
            "Subtraction result cannot be negative"
        );
        self.numerator -= rhs.numerator;
    }
}

impl std::ops::Mul for SafeFraction {
    type Output = MultipliedSafeFraction;

    fn mul(
        self,
        rhs: Self,
    ) -> Self::Output {
        MultipliedSafeFraction {
            numerator: self.numerator * rhs.numerator,
        }
    }
}

impl From<SafeFraction> for MultipliedSafeFraction {
    fn from(f: SafeFraction) -> Self {
        MultipliedSafeFraction {
            numerator: f.numerator * 10_000,
        }
    }
}

impl std::ops::Add for MultipliedSafeFraction {
    type Output = Self;

    fn add(
        self,
        rhs: Self,
    ) -> Self::Output {
        MultipliedSafeFraction {
            numerator: self.numerator + rhs.numerator,
        }
    }
}

impl MultipliedSafeFraction {
    /// Fractionalize a balance.
    pub fn multiply_balance(
        &self,
        value: Balance,
    ) -> Balance {
        value / 100_000_000u128 * self.numerator as u128
    }
}
