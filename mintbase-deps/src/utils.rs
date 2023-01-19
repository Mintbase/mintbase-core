#[cfg(feature = "market-wasm")]
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
