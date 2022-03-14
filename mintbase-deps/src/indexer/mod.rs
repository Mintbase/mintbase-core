// previously `mintbase_std`
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

pub use crate::logging::{
    NearJsonEvent,
    Nep171Event,
    Nep171EventLog,
    NftApproveLog,
    NftBurnLog,
    NftComposeLog,
    NftListLog,
    NftLoanSetLog,
    NftMarketLog,
    NftMintLog,
    NftMintLogMemo,
    NftOfferLog2,
    NftOptionStringLog,
    NftRevokeLog,
    NftSaleLog,
    NftSetSplitOwnerLog,
    NftStoreCreateLog,
    NftStringLog,
    NftTransferLog,
    NftUpdateListLog,
    NftUpdateOfferLog,
};

// ---------------------------- helper functions ---------------------------- //

pub async fn get_postgres_conn() -> tokio_postgres::Client {
    let (client, connection) = tokio_postgres::connect(
        "host=localhost user=postgres dbname=postgres password=abc123 port=5433",
        tokio_postgres::NoTls,
    )
    .await
    .unwrap();

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        } else {
            println!("done");
        }
    });

    client
}

#[cfg(feature = "all")]
pub fn near_json_event_from_str(s: &str) -> Result<NearJsonEvent, serde_json::Error> {
    let s = s.replace("EVENT_JSON:", "");
    let s = s.replace("EVENT_JSON", "");
    let event = serde_json::from_str::<NearJsonEvent>(s.as_str())?;
    Ok(event)
}

#[cfg(feature = "all")]
pub fn near_nep171_event_from_str(s: &str) -> Result<Nep171Event, serde_json::Error> {
    let s = s.replace("EVENT_JSON:", "");
    let s = s.replace("EVENT_JSON", "");
    let event = serde_json::from_str::<Nep171Event>(s.as_str())?;
    Ok(event)
}

#[cfg(feature = "all")]
pub fn indexer_home_dir() -> std::path::PathBuf {
    near_indexer::get_default_home()
}

#[cfg(feature = "all")]
pub fn indexer_pk() -> std::path::PathBuf {
    let mut home_dir = indexer_home_dir();
    home_dir.push("validator_key.json");
    home_dir
}
#[cfg(feature = "all")]
pub fn clear_dir() {
    let dir = indexer_home_dir();
    println!("clearing {:?}", dir);
    std::fs::remove_dir_all(&dir).unwrap();
}

// ------------------------------ STDIO locks ------------------------------- //

pub enum StdioLock<'a> {
    Stdout(std::io::StdoutLock<'a>),
    Stderr(std::io::StderrLock<'a>),
}

impl<'a> std::io::Write for StdioLock<'a> {
    fn write(
        &mut self,
        buf: &[u8],
    ) -> std::io::Result<usize> {
        match self {
            StdioLock::Stdout(lock) => lock.write(buf),
            StdioLock::Stderr(lock) => lock.write(buf),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            StdioLock::Stdout(lock) => lock.flush(),
            StdioLock::Stderr(lock) => lock.flush(),
        }
    }

    fn write_all(
        &mut self,
        buf: &[u8],
    ) -> std::io::Result<()> {
        match self {
            StdioLock::Stdout(lock) => lock.write_all(buf),
            StdioLock::Stderr(lock) => lock.write_all(buf),
        }
    }
}

pub struct MyMakeWriter {
    pub stdout: std::io::Stdout,
    pub stderr: std::io::Stderr,
}

impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for MyMakeWriter {
    type Writer = StdioLock<'a>;

    fn make_writer(&'a self) -> Self::Writer {
        // We must have an implementation of `make_writer` that makes
        // a "default" writer without any configuring metadata. Let's
        // just return stdout in that case.
        StdioLock::Stdout(self.stdout.lock())
    }

    fn make_writer_for(
        &'a self,
        meta: &tracing::Metadata<'_>,
    ) -> Self::Writer {
        // Here's where we can implement our special behavior. We'll
        // check if the metadata's verbosity level is WARN or ERROR,
        // and return stderr in that case.
        if meta.level() <= &tracing::Level::WARN {
            return StdioLock::Stderr(self.stderr.lock());
        }

        // Otherwise, we'll return stdout.
        StdioLock::Stdout(self.stdout.lock())
    }
}

// TODO: no usage, deprecated?
pub type DynamicError = Box<dyn std::error::Error>;
