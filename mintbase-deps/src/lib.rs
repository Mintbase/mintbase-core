/// Panic conditions
// TODO: simplify (`near_assert!` should suffice)
pub mod asserts;
/// Storage costs, gas costs, maximum processable entities
pub mod constants;
/// Function interfaces for cross-contract calls
pub mod interfaces;
/// Holds events
pub mod logging;
/// Blockchain and consumer-facing representation of an NFT
// pub mod token;
/// Commonly used methods
// TODO: make sure this is only used internally?
pub mod utils;

/// Types that the market uses to interface with the blockchain or with callers
#[cfg(feature = "market-wasm")]
pub mod market_data;
/// Types that the store uses to interface with the blockchain or with callers
// #[cfg(any(feature = "market-wasm", feature = "factory-wasm"))]
pub mod store_data;

// ----------------- re-exports for consistent dependencies ----------------- //
pub use near_sdk::{
    self,
    serde,
    serde_json,
};
