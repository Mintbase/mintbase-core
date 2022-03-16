use near_sdk::borsh::{
    self,
    BorshDeserialize,
    BorshSerialize,
};
use near_sdk::env;
use serde::{
    Deserialize,
    Serialize,
};

/// This enum used to support other time denominations, which were dropped
/// for simplicity.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "wasm", derive(BorshSerialize, BorshDeserialize,))]
pub enum TimeUnit {
    Hours(u64),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "wasm", derive(BorshDeserialize, BorshSerialize))]
pub struct NearTime(pub u64);

impl NearTime {
    pub fn is_before_timeout(&self) -> bool {
        now().0 < self.0
    }

    pub fn new(span: TimeUnit) -> Self {
        match span {
            TimeUnit::Hours(n) => Self::now_plus_n_hours(n),
        }
    }

    fn now_plus_n_hours(n: u64) -> Self {
        assert!(n > 0);
        assert!(
            n < 70_000,
            "maximum argument for hours is 70,000 (~8 years)"
        );
        let now = env::block_timestamp();
        let hour_ns = 10u64.pow(9) * 3600;
        Self(now + n * hour_ns)
    }
}

/// An alias for env::block_timestamp. Note that block_timestamp returns
/// the number of **nanoseconds since Jan 1 1970 UTC**. Note that each day
/// is 8.64*10^14 nanoseconds.
pub fn now() -> NearTime {
    NearTime(env::block_timestamp())
}
