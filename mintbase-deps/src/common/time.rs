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

/// Time duration.
/// This enum used to support other time denominations, which were dropped
/// for simplicity.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "wasm", derive(BorshSerialize, BorshDeserialize,))]
pub enum TimeUnit {
    Hours(u64),
}

/// Time instant, the u64 is in nanoseconds since epoch.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "wasm", derive(BorshDeserialize, BorshSerialize))]
pub struct NearTime(pub u64);

impl NearTime {
    pub fn is_before_timeout(&self) -> bool {
        env::block_timestamp() < self.0
    }

    pub fn new(span: TimeUnit) -> Self {
        match span {
            TimeUnit::Hours(n) => Self::now_plus_n_hours(n),
        }
    }

    pub fn now() -> Self {
        Self(env::block_timestamp())
    }

    fn now_plus_n_hours(n: u64) -> Self {
        crate::near_assert!(n > 0, "Cannot set times into the past");
        crate::near_assert!(
            n < 70_000,
            "Cannot set times more than 70_000 hours into the future (~8 years)"
        );
        let now = env::block_timestamp();
        let hour_ns = 10u64.pow(9) * 3600;
        Self(now + n * hour_ns)
    }
}
