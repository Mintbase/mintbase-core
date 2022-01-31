mod consts;
mod enums;
mod fns;
mod impls;
mod structs;
mod traits;

use std::convert::TryFrom;
use std::path::PathBuf;
use std::str::FromStr;

pub use consts::*;
pub use enums::*;
pub use fns::*;
pub use impls::*;
pub use structs::*;
pub use traits::*;

#[cfg(feature = "all")]
mod mintbase_std {
    pub use {
        bigdecimal,
        chrono,
        dotenv,
        futures,
        hyper,
        near_account_id,
        near_client_primitives,
        near_crypto,
        near_indexer,
        near_jsonrpc_client,
        near_jsonrpc_primitives,
        near_primitives,
        near_sdk,
        openssl_probe,
        rayon,
        reqwest,
        tokio,
        tokio_postgres,
        tokio_stream,
        tower,
        tracing,
        tracing_appender,
        tracing_subscriber,
        uuid,
    };
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
