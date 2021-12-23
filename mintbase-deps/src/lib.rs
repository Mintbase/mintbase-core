mod consts;
mod enums;
mod fns;
mod impls;
mod structs;
mod traits;

pub use consts::*;
pub use enums::*;
pub use fns::*;
pub use impls::*;
pub use structs::*;
pub use traits::*;

use std::{
    env::var,
    path::PathBuf,
    str::FromStr,
};

#[cfg(feature = "all")]
mod mintbase_std {
    pub use bigdecimal;
    pub use chrono;
    pub use dotenv;
    pub use futures;
    pub use hyper;
    pub use near_account_id;
    pub use near_client_primitives;
    pub use near_crypto;
    pub use near_indexer;
    pub use near_jsonrpc_client;
    pub use near_jsonrpc_primitives;
    pub use near_primitives;
    pub use near_sdk;
    pub use openssl_probe;
    pub use rayon;
    pub use reqwest;
    pub use tokio;
    pub use tokio_postgres;
    pub use tokio_stream;
    pub use tower;
    pub use tracing;
    pub use tracing_appender;
    pub use tracing_subscriber;
    pub use uuid;
}
#[cfg(feature = "all")]
pub use mintbase_std::*;

#[cfg(feature = "wasm")]
pub use near_sdk;

// use clap::*;

// #[cfg_attr(not(feature = "smart-contracts"), derive(Clap))]
// // #[clap(
// //     version = "0.1.0",
// //     author = "evergreen trading systems <4870868+evergreen-trading-systems@users.noreply.github.com>"
// // )]
// // #[clap(setting = AppSettings::ColoredHelp)]
// pub struct LeaderboardClap {
//     #[cfg_attr(not(feature = "smart-contracts"), clap(long))]
//     pub top_stores: bool,
// }
