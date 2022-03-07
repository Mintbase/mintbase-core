use near_sdk::env;

#[cfg(feature = "all")]
use crate::logging::{
    NearJsonEvent,
    Nep171Event,
};
#[cfg(feature = "all")]
use crate::tokio_postgres::NoTls;
use crate::*;
#[cfg(feature = "all")]
use crate::{
    tokio,
    tokio_postgres,
};

#[cfg(feature = "all")]
pub async fn get_postgres_conn() -> tokio_postgres::Client {
    let (client, connection) = tokio_postgres::connect(
        "host=localhost user=postgres dbname=postgres password=abc123 port=5433",
        NoTls,
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

/// Split a &str around a dash char
pub fn split_colon(string: &str) -> (&str, &str) {
    let pos = string.find(':').expect("no colon");
    (&string[..pos], &string[(pos + 1)..])
}

pub fn to_yocto(value: &str) -> u128 {
    let vals: Vec<_> = value.split('.').collect();
    let part1 = vals[0].parse::<u128>().unwrap() * 10u128.pow(24);
    if vals.len() > 1 {
        let power = vals[1].len() as u32;
        let part2 = vals[1].parse::<u128>().unwrap() * 10u128.pow(24 - power);
        part1 + part2
    } else {
        part1
    }
}

/// An alias for env::block_timestamp. Note that block_timestamp returns
/// the number of **nanoseconds since Jan 1 1970 UTC**. Note that each day
/// is 8.64*10^14 nanoseconds.
pub fn now() -> NearTime {
    NearTime(env::block_timestamp())
}

pub fn to_near(n: u128) -> u128 {
    n * 10u128.pow(24)
}
