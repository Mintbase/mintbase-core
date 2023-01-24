// TODO: specify in near events so import is not required here
#[cfg(feature = "factory-wasm")]
use near_sdk::serde::Serialize;

#[cfg(feature = "market-wasm")]
pub mod market_events;
#[cfg(feature = "store-wasm")]
pub mod store_events;

#[cfg(feature = "market-wasm")]
pub use market_events::*;
#[cfg(feature = "store-wasm")]
pub use store_events::*;

// ----------------------------- Factory event ------------------------------ //
#[cfg(feature = "factory-wasm")]
#[near_events::near_event_data(standard = "mb_store", version = "0.1.0", event = "deploy")]
pub struct MbStoreDeployData {
    pub contract_metadata: crate::store_data::NFTContractMetadata,
    pub owner_id: String,
    pub store_id: String,
}
