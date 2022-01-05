use crate::*;
// #[cfg(feature = "test")]
// use near_indexer_test_framework::*;
#[cfg(feature = "all")]
use crate::tokio_postgres::NoTls;
#[cfg(feature = "all")]
use crate::{
    tokio,
    tokio_postgres,
};
// #[cfg(feature = "test")]
// use near_indexer_test_framework::NearState;
#[cfg(feature = "all")]
use crate::near_indexer::IndexerExecutionOutcomeWithReceipt;
use near_sdk::{
    env,
    json_types::{
        U128,
        U64,
    },
    AccountId,
};
use serde_json::{
    self,
};
use std::{
    collections::HashMap,
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
pub fn indexer_home_dir() -> PathBuf {
    std::path::PathBuf::from(near_indexer::get_default_home())
}

#[cfg(feature = "all")]
pub fn indexer_pk() -> PathBuf {
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

#[cfg(feature = "factory-wasm")]
pub fn log_factory_new(store: &NFTContractMetadata, store_account_id: &str, owner_id: &str) {
    let nscl = NftStoreCreateLog {
        contract_metadata: store.clone(),
        owner_id: owner_id.to_string(),
        id: store_account_id.to_string(),
    };
    let event = NearJsonEvent {
        standard: "nep171".to_string(),
        version: "1.0.0".to_string(),
        event: "nft_store_creation".to_string(),
        data: serde_json::to_string(&nscl).unwrap(),
    };
    env::log_str(event.near_json_event().as_str());
}

// todo - change to testd, use better editor to highlight
// #[cfg(feature = "all")]
// pub async fn latest_nonce(s:near_account_id::AccountId) ->u64{
//
//     let signer = root_signer();
//     let near_cli = near_jsonrpc_client::JsonRpcClient::connect("http://localhost:3030");
//
//     let rbreq = near_jsonrpc_client::methods::query::RpcQueryRequest {
//         block_reference: near_primitives::types::BlockReference::latest(),
//         request: near_primitives::views::QueryRequest::ViewAccessKey {
//             account_id: s.clone(),
//             public_key: signer.public_key.clone(),
//         },
//     };
//
//     let rbres = near_cli.call(rbreq).await.unwrap();
//     match rbres.kind {
//         near_jsonrpc_primitives::types::query::QueryResponseKind::AccessKey(v) => v.nonce + 1,
//         _ => unreachable!(),
//     }
// }

// todo - change to testd, use better editor to highlight
#[cfg(feature = "all")]
pub fn root_signer() -> near_crypto::InMemorySigner {
    let root_account = std::env::var("INDEXER_ACCOUNT_ID").unwrap();
    let root_p = std::env::var("NEAR_ROOT_PUBLIC").unwrap();
    let root_s = std::env::var("NEAR_ROOT_SECRET").unwrap();
    let account = near_account_id::AccountId::from_str("test.near").unwrap();
    near_crypto::InMemorySigner::from_secret_key(
        account,
        near_crypto::SecretKey::from_str(root_s.as_str()).unwrap(),
    )
}

#[cfg(feature = "all")]
pub async fn process_event(outcome: &IndexerExecutionOutcomeWithReceipt) {
    println!("yoo");
    let event = &outcome.execution_outcome.outcome.logs[0];
    println!("hello {}", event);

    let event = serde_json::from_str::<NearJsonEvent>(event.as_str()).unwrap();

    let event_data = NftEvent::try_from(event.data.as_str());

    // match event_data {
    //     Ok(v) => {
    //         v.handle_nft_event(outcome.receipt.receipt_id).await;
    //     }
    //     Err(v) => {
    //         println!("sorry");
    //         eprintln!("{} {:?}", v, event)
    //     }
    // };

    // let json:Value =  serde_json::from_str(event.as_str()).unwrap();
    // let event_name = json.get("type");
    // match event_name {
    //     None => {
    //         println!("error: {}",json);
    //     }
    //     Some(v) => {
    //         let event_name = v.as_str().unwrap();
    //         let params = json.get("params").unwrap();
    //         let contract_account = outcome.execution_outcome.outcome.executor_id.to_string();
    //         println!("type: {:?} args: {:?}", v, params);
    //         match event_name {
    //             "store_creation" => Event::StoreCreation.event_handler(params),
    //             x => {
    //                 println!("{}",x);
    //             }
    //         }
    //     }
    // }

    // execute_log2(
    //   &client,
    //   &json["type"],
    //   &json["params"],
    //   contract_account,
    //   x.receipt.receipt_id.to_string(),
    // )
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

pub fn log_grant_minter(account_id: &AccountId) {
    let log = NftStringLog {
        data: account_id.to_string(),
    };
    let event = NearJsonEvent {
        standard: "nep171".to_string(),
        version: "1.0.0".to_string(),
        event: "nft_grant_minter".to_string(),
        data: serde_json::to_string(&log).unwrap(),
    };
    env::log_str(event.near_json_event().as_str());
}

pub fn log_revoke_minter(account_id: &AccountId) {
    // env::log_str(
    //     json!({
    //       "type": "revoke_minter".to_string(),
    //       "params": {
    //         "account": account_id,
    //       }
    //     })
    //     .to_string()
    //     .as_str(),
    // );
    let log = NftStringLog {
        data: account_id.to_string(),
    };
    let event = NearJsonEvent {
        standard: "nep171".to_string(),
        version: "1.0.0".to_string(),
        event: "nft_revoke_minter".to_string(),
        data: serde_json::to_string(&log).unwrap(),
    };
    env::log_str(event.near_json_event().as_str());
}

pub fn log_transfer_store(to: &AccountId) {
    // env::log_str(
    //     json!({
    //       "type": "transfer_store".to_string(),
    //       "params": {
    //         "account": to
    //       }
    //     })
    //         .to_string()
    //         .as_str(),
    // );
    let log = NftStringLog {
        data: to.to_string(),
    };
    let event = NearJsonEvent {
        standard: "nep171".to_string(),
        version: "1.0.0".to_string(),
        event: "nft_transfer_store".to_string(),
        data: serde_json::to_string(&log).unwrap(),
    };
    env::log_str(event.near_json_event().as_str());
}

pub fn log_set_icon_base64(base64: &Option<String>) {
    // env::log_str(
    //     json!({
    //       "type": "set_icon_base64".to_string(),
    //       "params": {
    //         "base64": base64,
    //       }
    //     })
    //         .to_string()
    //         .as_str(),
    // );
    let log = NftOptionStringLog {
        data: base64.clone(),
    };
    let event = NearJsonEvent {
        standard: "nep171".to_string(),
        version: "1.0.0".to_string(),
        event: "nft_set_icon_base64".to_string(),
        data: serde_json::to_string(&log).unwrap(),
    };
    env::log_str(event.near_json_event().as_str());
}

pub fn log_set_base_uri(base_uri: &str) {
    // env::log_str(
    //     json!({
    //       "type": "set_base_uri".to_string(),
    //       "params": {
    //         "uri": base_uri,
    //       }
    //     })
    //         .to_string()
    //         .as_str(),
    // );
    let log = NftStringLog {
        data: base_uri.to_string(),
    };
    let event = NearJsonEvent {
        standard: "nep171".to_string(),
        version: "1.0.0".to_string(),
        event: "nft_set_base_uri".to_string(),
        data: serde_json::to_string(&log).unwrap(),
    };
    env::log_str(event.near_json_event().as_str());
}

#[allow(clippy::too_many_arguments)]
pub fn log_nft_batch_mint(
    first_token_id: u64,
    last_token_id: u64,
    minter: &str,
    owner: &str,
    royalty: &Option<Royalty>,
    split_owners: &Option<SplitOwners>,
    meta_ref: &Option<String>,
    meta_extra: &Option<String>,
) {
    let memo = serde_json::to_string(&NftMintLogMemo {
        royalty: royalty.clone(),
        split_owners: split_owners.clone(),
        meta_id: meta_ref.clone(),
        meta_extra: meta_extra.clone(),
        minter: minter.to_string(),
    })
    .unwrap();
    let token_ids = (first_token_id..=last_token_id)
        .map(|x| x.to_string())
        .collect::<Vec<_>>();
    let log = vec![NftMintLog {
        owner_id: owner.to_string(),
        token_ids,
        memo: Option::from(memo),
    }];
    let event = Nep171Event {
        standard: "nep171".to_string(),
        version: "1.0.0".to_string(),
        event_kind: Nep171EventLog::NftMint(log),
    };

    env::log_str(event.near_json_event().as_str());
    // env::log_str(
    //     json!({
    //       "type": "nft_batch_mint".to_string(),
    //       "params": {
    //         "minter": minter,
    //         "owner_id": owner,
    //         "first_token_id": first_token_id,
    //         "last_token_id": last_token_id,
    //         "royalty": royalty,
    //         "split_owners": split_owners,
    //         "meta_id": meta_ref,
    //         "meta_extra": meta_extra,
    //       }
    //     })
    //         .to_string()
    //         .as_str(),
    // );
}

pub fn log_nft_batch_burn(token_ids: &[U64], owner_id: String) {
    // env::log_str(
    //     json!({
    //       "type": "nft_batch_burn".to_string(),
    //       "params": {
    //         "token_ids": token_ids,
    //       }
    //     })
    //     .to_string()
    //     .as_str(),
    // );
    let token_ids = token_ids
        .iter()
        .map(|x| x.0.to_string())
        .collect::<Vec<_>>();
    let log = vec![NftBurnLog {
        owner_id,
        authorized_id: None,
        token_ids,
        memo: None,
    }];
    let event = Nep171Event {
        standard: "nep171".to_string(),
        version: "1.0.0".to_string(),
        event_kind: Nep171EventLog::NftBurn(log),
    };
    env::log_str(event.near_json_event().as_str());
}

//////////
// NEPs //
//////////

// Approval
pub fn log_approve(token_id: u64, approval_id: u64, account_id: &AccountId) {
    let log = vec![NftApproveLog {
        token_id,
        approval_id,
        account_id: account_id.to_string(),
    }];
    let event = NearJsonEvent {
        standard: "nep171".to_string(),
        version: "1.0.0".to_string(),
        event: "nft_approve".to_string(),
        data: serde_json::to_string(&log).unwrap(),
    };
    env::log_str(event.near_json_event().as_str());
    // env::log_str(
    //     json!({
    //       "type": "1_approve".to_string(),
    //       "params": {
    //         "token_id": token_id,
    //         "approval_id": approval_id,
    //         "account": account_id,
    //       }
    //     })
    //         .to_string()
    //         .as_str(),
    // );
}

pub fn log_batch_approve(tokens: &[U64], approvals: &[U64], account_id: &AccountId) {
    let log = approvals
        .iter()
        .enumerate()
        .map(|(u, x)| NftApproveLog {
            token_id: tokens[u].0,
            approval_id: x.0,
            account_id: account_id.to_string(),
        })
        .collect::<Vec<_>>();
    let event = NearJsonEvent {
        standard: "nep171".to_string(),
        version: "1.0.0".to_string(),
        event: "nft_approve".to_string(),
        data: serde_json::to_string(&log).unwrap(),
    };
    env::log_str(event.near_json_event().as_str());
    // env::log_str(
    //     json!({
    //       "type": "batch_approve".to_string(),
    //       "params": {
    //         "tokens": tokens,
    //         "approvals": approvals,
    //         "account": account_id,
    //       }
    //     })
    //         .to_string()
    //         .as_str(),
    // );
}

pub fn log_revoke(token_id: u64, account_id: &AccountId) {
    let log = NftRevokeLog {
        token_id,
        account_id: account_id.to_string(),
    };
    let event = NearJsonEvent {
        standard: "nep171".to_string(),
        version: "1.0.0".to_string(),
        event: "nft_revoke".to_string(),
        data: serde_json::to_string(&log).unwrap(),
    };
    env::log_str(event.near_json_event().as_str());
    // env::log_str(
    //     json!({
    //       "type": "revoke".to_string(),
    //       "params": {
    //         "token_id": token_id,
    //         "account": account_id,
    //       }
    //     })
    //         .to_string()
    //         .as_str(),
    // );
}

pub fn log_revoke_all(token_id: u64) {
    let log = NftStringLog {
        data: token_id.to_string(),
    };
    let event = NearJsonEvent {
        standard: "nep171".to_string(),
        version: "1.0.0".to_string(),
        event: "nft_revoke_all".to_string(),
        data: serde_json::to_string(&log).unwrap(),
    };
    env::log_str(event.near_json_event().as_str());
    // env::log_str(
    //     json!({
    //       "type": "revoke_all".to_string(),
    //       "params": {
    //         "token_id": token_id,
    //       }
    //     })
    //         .to_string()
    //         .as_str(),
    // );
}

// Core
pub fn log_nft_transfer(to: &AccountId, token_id: u64, memo: &Option<String>, old_owner: String) {
    let log = vec![NftTransferLog {
        authorized_id: None,
        old_owner_id: old_owner,
        new_owner_id: to.to_string(),
        token_ids: vec![token_id.to_string()],
        memo: memo.clone(),
    }];
    let event = Nep171Event {
        standard: "nep171".to_string(),
        version: "1.0.0".to_string(),
        event_kind: Nep171EventLog::NftTransfer(log),
    };
    env::log_str(event.near_json_event().as_str());
    //     json!({
    //       "type": "nft_transfer".to_string(),
    //       "params": {
    //         "to": to,
    //         "token_id": token_id,
    //         "memo": memo,
    //       }
    //     })
    //         .to_string()
    //         .as_str(),
    // );
}

pub fn log_nft_batch_transfer(tokens: &[U64], accounts: &[AccountId], old_owners: Vec<String>) {
    let log = accounts
        .iter()
        .enumerate()
        .map(|(u, x)| NftTransferLog {
            authorized_id: None,
            old_owner_id: old_owners[u].clone(),
            new_owner_id: x.to_string(),
            token_ids: vec![tokens[u].0.to_string()],
            memo: None,
        })
        .collect::<Vec<_>>();
    let event = Nep171Event {
        standard: "nep171".to_string(),
        version: "1.0.0".to_string(),
        event_kind: Nep171EventLog::NftTransfer(log),
    };
    env::log_str(event.near_json_event().as_str());
    //       "type": "nft_batch_transfer".to_string(),
    //       "params": {
    //         "tokens": tokens,
    //         "accounts": accounts,
    //       }
    //     })
    //         .to_string()
    //         .as_str(),
    // );
}

// payout
pub fn log_set_split_owners(token_ids: &[U64], split_owners: &SplitOwners) {
    let token_ids = token_ids
        .iter()
        .map(|x| x.0.to_string())
        .collect::<Vec<_>>();

    let log = NftSetSplitOwnerLog {
        split_owners: split_owners.clone(),
        token_ids,
    };
    let event = NearJsonEvent {
        standard: "nep171".to_string(),
        version: "1.0.0".to_string(),
        event: "nft_set_split_owners".to_string(),
        data: serde_json::to_string(&log).unwrap(),
    };
    env::log_str(event.near_json_event().as_str());
    //         .to_string()
    //         .as_str(),
    // );
}

pub fn log_nft_loan_set(token_id: u64, account_id: &Option<AccountId>) {
    let log = NftLoanSetLog {
        account_id: account_id.as_ref().map(|x| x.to_string()),
        token_id,
    };
    let event = NearJsonEvent {
        standard: "nep171".to_string(),
        version: "1.0.0".to_string(),
        event: "nft_loan_set".to_string(),
        data: serde_json::to_string(&log).unwrap(),
    };
    env::log_str(event.near_json_event().as_str());
    // env::log_str(

    //         .to_string()
    //         .as_str(),
    // );
}
// compose
pub fn log_nfts_compose(
    token_ids: &[U64],
    // direct parent of token_ids
    parent: &str,
    // - "t": owned directly by a token on this contract
    // - "k": owned directly by a token on another contract
    ttype: String,
    // local root of chain of token_ids
    lroot: Option<u64>,
    // holder of local root
    holder: String,
    depth: u8,
) {
    let log = NftComposeLog {
        token_ids: token_ids.to_vec(),
        parent: parent.to_string(),
        ttype,
        lroot,
        holder,
        depth,
    };
    let event = NearJsonEvent {
        standard: "nep171".to_string(),
        version: "1.0.0".to_string(),
        event: "nft_compose".to_string(),
        data: serde_json::to_string(&log).unwrap(),
    };
    env::log_str(event.near_json_event().as_str());
    // env::log_str(
    //     json!({
    //       "type": "nfts_compose".to_string(),
    //       "params": {
    //         "token_ids": token_ids,
    //         "parent": parent,
    //         "type": ttype,
    //         "lroot": lroot,
    //         "holder": holder,
    //         "depth": depth,
    //       }
    //     })
    //         .to_string()
    //         .as_str(),
    // );
}

pub fn log_nfts_uncompose(token_ids: &[U64], holder: AccountId) {
    let log = NftUncomposeLog {
        token_ids: token_ids.to_vec(),
        holder: holder.to_string(),
    };
    let event = NearJsonEvent {
        standard: "nep171".to_string(),
        version: "1.0.0".to_string(),
        event: "nft_uncompose".to_string(),
        data: serde_json::to_string(&log).unwrap(),
    };
    env::log_str(event.near_json_event().as_str());
    // env::log_str(
    //     json!({
    //       "type": "nfts_uncompose".to_string(),
    //       "params": {
    //         "token_ids": token_ids,
    //         "holder": holder,
    //       }
    //     })
    //         .to_string()
    //         .as_str(),
    // );
}

pub fn log_on_compose(
    predecessor: AccountId,
    token_id: U64,
    // direct parent of token_ids
    cross_child_id: U64,
    // local root of chain of token_ids
    lroot: Option<u64>,
    // holder of local root
    holder: String,
    depth: u8,
) {
    let log = NftOnComposeLog {
        predecessor: predecessor.to_string(),
        token_id,
        cross_child_id,
        lroot,
        holder,
        depth,
    };
    let event = NearJsonEvent {
        standard: "nep171".to_string(),
        version: "1.0.0".to_string(),
        event: "nft_on_compose".to_string(),
        data: serde_json::to_string(&log).unwrap(),
    };
    env::log_str(event.near_json_event().as_str());
    // env::log_str(
    //     json!({
    //       "type": "on_compose".to_string(),
    //       "params": {
    //         "predecessor": predecessor,
    //         "token_id": token_id,
    //         "cross_child_id": cross_child_id,
    //         "lroot": lroot,
    //         "holder": holder,
    //         "depth": depth,
    //       }
    //     })
    //         .to_string()
    //         .as_str(),
    // );
}

pub fn log_on_uncompose(token_id: U64, holder: &str, child_key: String) {
    let log = NftOnUncomposeLog {
        token_id,
        holder: holder.to_string(),
        child_key,
    };
    let event = NearJsonEvent {
        standard: "nep171".to_string(),
        version: "1.0.0".to_string(),
        event: "nft_on_uncompose".to_string(),
        data: serde_json::to_string(&log).unwrap(),
    };
    env::log_str(event.near_json_event().as_str());
    // env::log_str(
    //     json!({
    //       "type": "on_uncompose".to_string(),
    //       "params": {
    //         "token_id": token_id,
    //         "holder": holder,
    //         "child_key": child_key
    //       }
    //     })
    //         .to_string()
    //         .as_str(),
    // );
}

pub fn log_on_move(token_id: U64, origin_key: &str) {
    let log = NftOnMoveLog {
        token_id,
        origin_key: origin_key.to_string(),
    };
    let event = NearJsonEvent {
        standard: "nep171".to_string(),
        version: "1.0.0".to_string(),
        event: "nft_on_move".to_string(),
        data: serde_json::to_string(&log).unwrap(),
    };
    env::log_str(event.near_json_event().as_str());
    // env::log_str(
    //     json!({
    //       "type": "nft_on_move".to_string(),
    //       "params": {
    //         "token_id": token_id,
    //         "origin_key": origin_key,
    //       }
    //     })
    //         .to_string()
    //         .as_str(),
    // );
}

pub fn log_nft_moved(token_id: U64, contract_id: String) {
    let log = NftMovedLog {
        token_id,
        contract_id,
    };
    let event = NearJsonEvent {
        standard: "nep171".to_string(),
        version: "1.0.0".to_string(),
        event: "nft_moved".to_string(),
        data: serde_json::to_string(&log).unwrap(),
    };
    env::log_str(event.near_json_event().as_str());
    // env::log_str(
    //     json!({
    //       "type": "4".to_string(),
    //       "params": {
    //         "token_id": token_id,
    //         "contract_id": contract_id,
    //       }
    //     })
    //     .to_string()
    //     .as_str(),
    // );
}

/// An alias for env::block_timestamp. Note that block_timestamp returns
/// the number of **nanoseconds since Jan 1 1970 UTC**. Note that each day
/// is 8.64*10^14 nanoseconds.
pub fn now() -> NearTime {
    NearTime(env::block_timestamp())
}

pub fn log_listing_created(
    list_id: &str,
    price: &U128,
    token_key: &str,
    owner_id: &AccountId,
    autotransfer: bool,
) {
    let mut iter = token_key.split(':');
    let mut iter2 = list_id.split(':');
    let token_id = iter.next();
    let store_id = iter.next();
    iter2.next();
    let approval_id = iter2.next().unwrap();
    let log = vec![NftListLog {
        list_id: list_id.to_string(),
        price: price.0.to_string(),
        token_key: token_key.to_string(),
        owner_id: owner_id.to_string(),
        autotransfer,
        approval_id: approval_id.to_string(),
        token_id: token_id.unwrap().to_string(),
        store_id: store_id.unwrap().to_string(),
    }];
    let event = NearJsonEvent {
        standard: "nep171".to_string(),
        version: "1.0.0".to_string(),
        event: "nft_1_list".to_string(),
        data: serde_json::to_string(&log).unwrap(),
    };
    env::log_str(event.near_json_event().as_str());
    // env::log(
    //     json!({
    //   "type": "1_list".to_string(),
    //   "params": {
    //     "list_id": list_id,
    //     "price": price,
    //     "token_key": token_key,
    //     "owner_id": owner_id,
    //     "autotransfer": autotransfer,
    //   }
    // })
    //         .to_string()
    //         .as_bytes(),
    // );
}

pub fn log_batch_listing_created(
    approval_ids: &[U64],
    price: &U128,
    token_ids: &[U64],
    owner_id: &AccountId,
    store_id: &AccountId,
    autotransfer: bool,
) {
    let log = approval_ids
        .iter()
        .enumerate()
        .map(|(u, x)| {
            let list_id = format!("{}:{}:{}",token_ids[u].0,x.0,store_id);
            let token_key = format!("{}:{}",token_ids[u].0,store_id);
            NftListLog {
                list_id,
                price: price.0.to_string(),
                token_key,
                owner_id: owner_id.to_string(),
                autotransfer,
                approval_id: x.0.to_string(),
                token_id: token_ids[u].0.to_string(),
                store_id: store_id.to_string(),
            }
        })
        .collect::<Vec<_>>();
    let event = NearJsonEvent {
        standard: "nep171".to_string(),
        version: "1.0.0".to_string(),
        event: "nft_batch_list".to_string(),
        data: serde_json::to_string(&log).unwrap(),
    };
    env::log_str(event.near_json_event().as_str());
    // env::log(
    //     json!({
    //   "type": "batch_list".to_string(),
    //   "params": {
    //     "approval_ids": approval_ids,
    //     "price": price,
    //     "token_ids": token_ids,
    //     "owner_id": owner_id,
    //     "store_id": store_id,
    //     "autotransfer": autotransfer,
    //   }
    // })
    //         .to_string()
    //         .as_bytes(),
    // );
}

pub fn log_set_token_autotransfer(auto_transfer: bool, list_id: &str) {
    let log = vec![NftUpdateListLog {
        auto_transfer: Option::from(auto_transfer),
        price: None,
        list_id: Option::from(list_id.to_string()),
    }];
    let event = NearJsonEvent {
        standard: "nep171".to_string(),
        version: "1.0.0".to_string(),
        event: "nft_set_autotransfer".to_string(),
        data: serde_json::to_string(&log).unwrap(),
    };
    env::log_str(event.near_json_event().as_str());
    // env::log(
    //     json!({
    //   "type": "set_autotransfer".to_string(),
    //   "params": {
    //     "autotransfer": autotransfer,
    //     "list_id": list_id,
    //   }
    // })
    //         .to_string()
    //         .as_bytes(),
    // );
}

pub fn log_set_token_asking_price(price: &U128, list_id: &str) {
    let log = vec![NftUpdateListLog {
        auto_transfer: None,
        price: Option::from(price.0.to_string()),
        list_id: Option::from(list_id.to_string()),
    }];
    let event = NearJsonEvent {
        standard: "nep171".to_string(),
        version: "1.0.0".to_string(),
        event: "nft_set_price".to_string(),
        data: serde_json::to_string(&log).unwrap(),
    };
    env::log_str(event.near_json_event().as_str());
    // env::log(
    //     json!({
    //   "type": "set_price".to_string(),
    //   "params": {
    //     "price": price,
    //     "list_id": list_id,
    //   }
    // })
    //         .to_string()
    //         .as_bytes(),
    // );
}

pub fn log_make_offer(
    offer: Vec<&TokenOffer>,
    token_key: Vec<&String>,
    list_id: Vec<String>,
    offer_num: Vec<u64>,
) {
    let log = offer
        .iter()
        .enumerate()
        .map(|(u, &x)| NftOfferLog2 {
            offer: x.clone(),
            list_id: list_id[u].clone(),
            token_key: token_key[u].clone(),
            offer_num: offer_num[u],
        })
        .collect::<Vec<_>>();
    let event = NearJsonEvent {
        standard: "nep171".to_string(),
        version: "1.0.0".to_string(),
        event: "nft_make_offer".to_string(),
        data: serde_json::to_string(&log).unwrap(),
    };
    env::log_str(event.near_json_event().as_str());
    // env::log(
    //     json!({
    //   "type": "make_offer".to_string(),
    //   "params": {
    //     "price": offer.price.to_string(),
    //     "from": offer.from,
    //     "timeout": offer.timeout,
    //     "list_id": list_id,
    //     "token_key": token_key,
    //     "offer_num": offer_num,
    //   }
    // })
    //         .to_string()
    //         .as_bytes(),
    // );
}

pub fn log_withdraw_token_offer(list_id: &str, offer_num: u64) {
    let log = NftUpdateOfferLog {
        offer_num,
        list_id: list_id.to_string(),
    };
    let event = NearJsonEvent {
        standard: "nep171".to_string(),
        version: "1.0.0".to_string(),
        event: "nft_withdraw_offer".to_string(),
        data: serde_json::to_string(&log).unwrap(),
    };
    env::log_str(event.near_json_event().as_str());
    // env::log(
    //     json!({
    //   "type": "withdraw_offer".to_string(),
    //   "params": {
    //     "list_id": list_id,
    //     "offer_num": offer_num,
    //   }
    // })
    //         .to_string()
    //         .as_bytes(),
    // );
}

pub fn log_sale(list_id: &str, offer_num: u64, token_key: &str, payout: &HashMap<AccountId, U128>) {
    let log = NftSaleLog {
        list_id: list_id.to_string(),
        offer_num,
        token_key: token_key.to_string(),
        payout: payout.clone(),
    };
    let event = NearJsonEvent {
        standard: "nep171".to_string(),
        version: "1.0.0".to_string(),
        event: "nft_sold".to_string(),
        data: serde_json::to_string(&log).unwrap(),
    };
    env::log_str(event.near_json_event().as_str());
    let _e = r#"few"#;
    // env::log(
    //     json!({
    //   "type": "sold".to_string(),
    //   "params": {
    //     "list_id": list_id,
    //     "offer_num": offer_num,
    //     "payout": payout,
    //     "token_key": token_key
    //   }
    // })
    //         .to_string()
    //         .as_bytes(),
    // );
}

pub fn log_token_removed(list_id: &str) {
    let log = NftStringLog {
        data: list_id.to_string(),
    };
    let event = NearJsonEvent {
        standard: "nep171".to_string(),
        version: "1.0.0".to_string(),
        event: "nft_removed".to_string(),
        data: serde_json::to_string(&log).unwrap(),
    };
    env::log_str(event.near_json_event().as_str());
    // env::log(
    //     json!({
    //   "type": "removed".to_string(),
    //   "params": {
    //     "list_id": list_id,
    //   }
    // })
    //         .to_string()
    //         .as_bytes(),
    // );
}

//////////////////
// Market owner //
//////////////////
pub fn log_allowlist_update(account_id: &AccountId, state: bool) {
    let log = vec![NftMarketLog {
        account_id: account_id.to_string(),
        state,
    }];
    let event = NearJsonEvent {
        standard: "nep171".to_string(),
        version: "1.0.0".to_string(),
        event: "nft_allowlist".to_string(),
        data: serde_json::to_string(&log).unwrap(),
    };
    env::log_str(event.near_json_event().as_str());
    // env::log(
    //     json!({
    //   "type": "allowlist".to_string(),
    //   "params": {
    //     "account": account_id,
    //     "state": state,
    //   }
    // })
    //         .to_string()
    //         .as_bytes(),
    // );
}
pub fn log_banlist_update(account_id: &AccountId, state: bool) {
    let log = vec![NftMarketLog {
        account_id: account_id.to_string(),
        state,
    }];
    let event = NearJsonEvent {
        standard: "nep171".to_string(),
        version: "1.0.0".to_string(),
        event: "nft_banlist".to_string(),
        data: serde_json::to_string(&log).unwrap(),
    };
    env::log_str(event.near_json_event().as_str());
    // env::log(
    //     json!({
    //   "type": "banlist".to_string(),
    //   "params": {
    //     "account": account_id,
    //     "state": state,
    //   }
    // })
    //         .to_string()
    //         .as_bytes(),
    // );
}

pub fn to_near(n: u128) -> u128 {
    n * 10u128.pow(24)
}


