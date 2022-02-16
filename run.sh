#!/bin/bash

# SETUP ENV #
# =====
# =====
# =====

mkdir -p bin
mkdir -p wasm
touch -a .postgres.sh

if [[ -z "${NETWORK}" ]]; then
  echo specify NETWORK variable - mainnet,testnet,local
  exit 1
else
  export NEAR_ENV=$NETWORK

fi

if [[ -z "${NEAR_DIR}" ]]; then
  eval NEAR_DIR="~/.near/$NETWORK"
else
  NEAR_DIR="${NEAR_DIR}"
fi
. .postgres.sh

if [[ -z "${POSTGRES}" ]]; then
  export POSTGRES="postgres://$postgres_user:$postgres_password@$postgres_host:5432/$postgres_database"
else
  export POSTGRES="$POSTGRES"
fi
# diesel cli
export DATABASE_URL=$POSTGRES

# rust log
if [[ -z "${RUST_LOG}" ]]; then
  export RUST_LOG="indexer=info,genesis=info,chain=info,client=info,stats=info,mintbase_near_indexer=info,near=error,mintbase_near_indexer=error"
else
  export RUST_LOG="${RUST_LOG}"
fi

# watch accounts
if [[ -z "${WATCH_ACCOUNTS}" ]]; then
  WATCH_ACCOUNTS="$root,tenk,nmkmint"
else
  WATCH_ACCOUNTS="${WATCH_ACCOUNTS}"
fi

# node_url, root, top_level_account
if [ "$NETWORK" = "testnet" ]; then
  node_url="https://rpc.testnet.near.org" #testnet
  top_level_account="testnet"
  root="mintspace2"
elif [ "$NETWORK" == "mainnet" ]; then
  node_url="https://rpc.mainnet.near.org" #mainnet
  top_level_account="near"
  root="mintbase1"
elif [ "$NETWORK" == "local" ]; then
  node_url="http://127.0.0.1:3030" #local
  top_level_account="near"
  root="test"
else
  echo "invalid network $NETWORK"
  exit 1
fi


# SETUP ENV END #
# =====
# =====
# =====

# ==================== #

# SETUP APPLICATION DATA #
# =====
# =====
# =====

root_account="$root.$top_level_account" # MUST ALREADY EXIST WITH LOCAL CRED
minter_account="minter02.$root_account"
market_account="market.$root_account"
helper_account="helper.$root_account"
store_account="store906.$root_account" #don't create store manually
store_owner_account="store-owner01.$root_account"
seller1_account="seller01.$root_account"
buyer1_account="buyer100.$root_account"
buyer2_account="buyer200.$root_account"
royalty1_account="royalty01.$root_account"
royalty2_account="royalty02.$root_account"
royalty3_account="royalty03.$root_account"
royalty4_account="royalty04.$root_account"
royalty5_account="royalty05.$root_account"
royalty6_account="royalty06.$root_account"
royalty7_account="royalty07.$root_account"
royalty8_account="royalty08.$root_account"
royalty9_account="royalty09.$root_account"
royalty10_account="royalty10.$root_account"
receiver_account="receiver01.$root_account"
receiver300_account="receiver01.$root_account"
receiver301_account="receiver01.$root_account"

# key path
if [ "$NETWORK" = "local" ]; then
  key_path="~/.near/local/validator_key.json"
else
  key_path="~/.near-credentials/$NETWORK/$root_account.json"
fi



# SETUP APPLICATION DATA END #
# =====
# =====
# =====

# ==================== #

# FUNCTIONS #
# =====
# =====
# =====

function tail_indexer_error_logs() {
  while read line; do
    case "$line" in
    *)
      gcloud logging write indexer-error-log "$line"
      ;;
    esac
  done < <(tail -f mintbase-core.error.log)
}

function build_contracts() {
  cd mintbase-deps && cargo market-wasm && cargo store-wasm && cargo factory-wasm && cargo helper-wasm && cd ../
}

function build_indexer() {
  cargo indexer
}

function run_indexer() {
  if [[ ! -d "$NEAR_DIR/data" ]]; then
    str='rm -rf _near_dir_'
    str="${str//_near_dir_/$NEAR_DIR}"
    echo $str
    eval $str

    if [ "$NETWORK" = "testnet" ]; then
      str='bin/indexer --home-dir _near_dir_ init --chain-id _NEAR_ENV_ --download-genesis;'
    elif [ "$NETWORK" = "mainnet" ]; then
      echo 22
    else
      str='bin/indexer --home-dir _near_dir_ init --chain-id _NEAR_ENV_;'
    fi
    str="${str//_near_dir_/$NEAR_DIR}"
    str="${str//_NEAR_ENV_/$NEAR_ENV}"
    echo $str
    eval $str

    sed -i 's/"tracked_shards": \[\],/"tracked_shards": [0],/g' $NEAR_DIR/config.json
  fi
  str='NETWORK=_network_ WATCH_ACCOUNTS=_WATCH_ACCOUNTS_ bin/indexer --home-dir _near_dir_ run'
  str="${str//_rust_log_/$RUST_LOG}"
  str="${str//_near_dir_/$NEAR_DIR}"
  str="${str//_WATCH_ACCOUNTS_/$WATCH_ACCOUNTS}"
  str="${str//_network_/$NETWORK}"
  str="${str//_postgres_/$POSTGRES}"
  if [ "$1" = "log" ]; then
    str="$str  >> out.log 2>> error.log"
  fi
  echo $str
  eval $str
}

function create_accounts() {
  N=3
  for i in $minter_account $market_account \
    $store_owner_account $seller1 \
    $buyer1_account $buyer2_account $royalty1_account \
    $royalty2_account $receiver_account $helper_account \
    $royalty3_account $royalty4_account $royalty5_account \
    $royalty6_account $royalty7_account $royalty8_account \
    $royalty9_account $royalty10_account; do
    #      ((z=z%N)); ((z++==0)) && wait
    str="near create-account $i --masterAccount $root_account --initialBalance 4 --nodeUrl $node_url --keyPath $key_path"
    echo running $str
    eval $str
  done
  #    wait
}

function deploy() {
  str='near deploy --wasmFile wasm/factory.wasm _root_account_ --initFunction new --initArgs null --masterAccount _root_account_ --nodeUrl _node_url_ --keyPath _key_path_'
  str="${str//_root_account_/$root_account}"
  str="${str//_node_url_/$node_url}"
  str="${str//_key_path_/$key_path}"
  echo running "$str"
  eval "$str"
  str='near deploy --wasmFile wasm/market.wasm _market_account_ --initFunction new --initArgs '\''{"init_allowlist": ["_root_account_"]}'\'' --masterAccount _root_account_ --nodeUrl _node_url_ --keyPath _key_path_'
  str="${str//_root_account_/$root_account}"
  str="${str//_market_account_/$market_account}"
  str="${str//_node_url_/$node_url}"
  str="${str//_key_path_/$key_path}"
  echo running "$str"
  eval "$str"
  str='near deploy --wasmFile wasm/helper.wasm _helper_account_ --initFunction new --initArgs null --masterAccount _root_account_ --nodeUrl _node_url_ --keyPath _key_path_'
  str="${str//_helper_account_/$helper_account}"
  str="${str//_node_url_/$node_url}"
  str="${str//_key_path_/$key_path}"
  echo running "$str"
  eval "$str"
}

function redeploy() {
  str='near deploy --wasmFile wasm/factory.wasm _root_account_ --masterAccount _root_account_ --nodeUrl _node_url_ --keyPath _key_path_'
  str="${str//_root_account_/$root_account}"
  str="${str//_node_url_/$node_url}"
  str="${str//_key_path_/$key_path}"
  echo running "$str"
  eval "$str"

  str='near deploy --wasmFile wasm/market.wasm _market_account_ --masterAccount _root_account_ --nodeUrl _node_url_ --keyPath _key_path_'
  str="${str//_root_account_/$root_account}"
  str="${str//_market_account_/$market_account}"
  str="${str//_node_url_/$node_url}"
  str="${str//_key_path_/$key_path}"
  echo running "$str"
  eval "$str"

  str='near deploy --wasmFile wasm/helper.wasm _helper_account_ --masterAccount _root_account_ --nodeUrl _node_url_ --keyPath _key_path_'
  str="${str//_root_account_/$root_account}"
  str="${str//_helper_account_/$helper_account}"
  str="${str//_node_url_/$node_url}"
  str="${str//_key_path_/$key_path}"
  echo running "$str"
  eval "$str"
}

function redeploy_single_store() {
  cred=$(cat ~/.near-credentials/$NETWORK/$root_account.json)
  echo "$cred" >~/.near-credentials/"$NETWORK"/"$1".json
  str='near deploy --wasmFile wasm/store.wasm _1_ --masterAccount _root_account_ --nodeUrl _node_url_ --keyPath _key_path_'
  str="${str//_root_account_/$root_account}"
  str="${str//_1_/$1}"
  str="${str//_node_url_/$node_url}"
  str="${str//_key_path_/$key_path}"
  echo running "$str"
  eval "$str"
}

function create_store() {
  store="${store_account/.$root_account/}"
  str='near call _root_account_ create_store '\''{"metadata":{"spec":"nft-1.0.0","name":"_store_account_","symbol":"A","icon":null,"base_uri":null,"reference":null,"reference_hash":null},"owner_id":"_root_account_"}'\'' --accountId _root_account_ --deposit 7 --gas 200000000000000 --nodeUrl _node_url_ --keyPath _key_path_'
  str="${str//_root_account_/$root_account}"
  str="${str//_store_account_/$store}"
  str="${str//_node_url_/$node_url}"
  str="${str//_key_path_/$key_path}"
  echo running "$str"
  eval "$str"
}

function grant_minter() {
  str='near call _store_account_ grant_minter '\''{"account_id":"_minter_"}'\'' --accountId _root_account_ --deposit 0.000000000000000000000001 --nodeUrl _node_url_ --keyPath _key_path_'
  str="${str//_minter_/$minter_account}"
  str="${str//_root_account_/$root_account}"
  str="${str//_store_account_/$store_account}"
  str="${str//_node_url_/$node_url}"
  str="${str//_key_path_/$key_path}"
  echo running "$str"
  eval "$str"
}

function send_to_store() {
  str='near send _root_account_ _store_account_ 10 --nodeUrl _node_url_ --keyPath _key_path_'
  str="${str//_root_account_/$root_account}"
  str="${str//_store_account_/$store_account}"
  str="${str//_node_url_/$node_url}"
  str="${str//_key_path_/$key_path}"
  echo running "$str"
  eval "$str"
}

function mint_tokens_nr() {
  str='near call _store_account_ nft_batch_mint '\''{"owner_id":"_minter_account_", "metadata":{"spec":"","name":"","symbol":"","icon":null,"base_uri":null,"reference":null,"reference_hash":null},"royalty_args":null,"num_to_mint":10,"split_owners":null}'\'' --accountId _minter_account_ --deposit 0.000000000000000000000001 --nodeUrl _node_url_ --keyPath _key_path_'
  str="${str//_root_account_/$root_account}"
  str="${str//_minter_account_/$minter_account}"
  str="${str//_store_account_/$store_account}"
  str="${str//_node_url_/$node_url}"
  str="${str//_key_path_/$key_path}"
  echo running "$str"
  eval "$str"
}

function mint_tokens_custom() {
  str='near call _store_account_ nft_batch_mint '\''_1_'\'' --accountId _minter_account_ --deposit 0.000000000000000000000001 --nodeUrl _node_url_ --keyPath _key_path_ --gas 300000000000000'
  str="${str//_1_/$1}"
  str="${str//_node_url_/$node_url}"
  str="${str//_key_path_/$key_path}"
  str="${str//_minter_account_/$minter_account}"
  str="${str//_store_owner_account_/$store_owner_account}"
  str="${str//_store_account_/$store_account}"
  str="${str//_royalty1_account_/$royalty1_account}"
  str="${str//_royalty2_account_/$royalty2_account}"
  echo running "$str"
  eval "$str"
}

function mint_tokens() {
  str='near call _store_account_ nft_batch_mint '\''{"owner_id":"_minter_account_", "metadata":{"spec":"","name":"","symbol":"","icon":null,"base_uri":null,"reference":null,"reference_hash":null},"royalty_args":{"split_between": {"_royalty1_account_": 8000,"_royalty2_account_": 2000}, "percentage": 1000},"num_to_mint":10,"split_owners":{"_minter_account_": 8000,"_store_owner_account_": 2000}}'\'' --accountId _minter_account_ --deposit 0.000000000000000000000001'
  str="${str//_minter_account_/$minter_account}"
  str="${str//_store_owner_account_/$store_owner_account}"
  str="${str//_store_account_/$store_account}"
  str="${str//_royalty1_account_/$royalty1_account}"
  str="${str//_royalty2_account_/$royalty2_account}"
  echo running "$str"
  eval "$str"
}

function mint_tokens2() {
  str='near call _store_account_ nft_batch_mint '\''{"owner_id":"_minter_account_", "metadata":{"spec":"","name":"","symbol":"","icon":null,"base_uri":null,"reference":null,"reference_hash":null},"royalty_args":{"split_between": {"_royalty1_account_": 1000,"_royalty2_account_": 1000, "_royalty3_account_": 1000,"_royalty4_account_": 1000,"_royalty5_account_": 1000,"_royalty6_account_": 1000,"_royalty7_account_": 1000,"_royalty8_account_": 1000,"_royalty9_account_": 1000,"_royalty10_account_": 1000}, "percentage": 1000},"num_to_mint":10,"split_owners":{"_minter_account_": 8000,"_store_owner_account_": 2000}}'\'' --accountId _minter_account_ --deposit 0.000000000000000000000001'
  str="${str//_minter_account_/$minter_account}"
  str="${str//_store_owner_account_/$store_owner_account}"
  str="${str//_store_account_/$store_account}"
  str="${str//_royalty1_account_/$royalty1_account}"
  str="${str//_royalty2_account_/$royalty2_account}"
  str="${str//_royalty3_account_/$royalty3_account}"
  str="${str//_royalty4_account_/$royalty4_account}"
  str="${str//_royalty5_account_/$royalty5_account}"
  str="${str//_royalty6_account_/$royalty6_account}"
  str="${str//_royalty7_account_/$royalty7_account}"
  str="${str//_royalty8_account_/$royalty8_account}"
  str="${str//_royalty9_account_/$royalty9_account}"
  str="${str//_royalty10_account_/$royalty10_account}"
  echo running "$str"
  eval "$str"
}

function nft_approve_autotransfer() {
  str='near call _store_account_ nft_approve '\''{"token_id":"_1_", "account_id":"_market_account_", "msg":"{\"price\":\"1000000000000000000000000\",\"autotransfer\":true}" }'\'' --accountId _minter_account_ --deposit 1 --gas 200000000000000 --nodeUrl _node_url_ --keyPath _key_path_'
  str="${str//_1_/$1}"
  str="${str//_market_account_/$market_account}"
  str="${str//_store_account_/$store_account}"
  str="${str//_minter_account_/$minter_account}"
  str="${str//_node_url_/$node_url}"
  str="${str//_key_path_/$key_path}"
  echo running "$str"
  eval "$str"
}

function nft_approve_manual_transfer() {
  str='near call _store_account_ nft_approve '\''{"token_id":"_1_", "account_id":"_market_account_", "msg":"{\"price\":\"1000000000000000000000000\",\"autotransfer\":false}" }'\'' --accountId _minter_account_ --deposit 1 --gas 200000000000000'
  str="${str//_1_/$1}"
  str="${str//_market_account_/$market_account}"
  str="${str//_store_account_/$store_account}"
  str="${str//_minter_account_/$minter_account}"
  echo running "$str"
  eval "$str"
}

function make_offer() {
  str='near call _market_account_ make_offer '\''{"token_key":["_1_:_store_account_"], "price":["1000000000000000000000000"], "timeout":[{"Hours":24}] }'\'' --accountId _buyer1_account_ --deposit 1 --gas 200000000000000 --nodeUrl _node_url_ --keyPath _key_path_'
  str="${str//_1_/$1}"
  str="${str//_market_account_/$market_account}"
  str="${str//_store_account_/$store_account}"
  str="${str//_buyer1_account_/$buyer1_account}"
  str="${str//_node_url_/$node_url}"
  str="${str//_key_path_/$key_path}"
  echo running "$str"
  eval "$str"

  str='near view _store_account_ nft_holder '\''{"token_id": "_1_"}'\''  --nodeUrl _node_url_ --keyPath _key_path_'
  str="${str//_1_/$1}"
  str="${str//_store_account_/$store_account}"
  str="${str//_node_url_/$node_url}"
  str="${str//_key_path_/$key_path}"
  echo running "$str"
  eval "$str"
}

function accept_offer_and_transfer() {
  str='near call _market_account_ accept_and_transfer '\''{"token_key":"_1_:_store_account_"}'\'' --accountId _minter_account_ --deposit 0.000000000000000000000001 --gas 200000000000000'
  str="${str//_1_/$1}"
  str="${str//_market_account_/$market_account}"
  str="${str//_store_account_/$store_account}"
  str="${str//_minter_account_/$minter_account}"
  echo running "$str"
  eval "$str"

  str='near view _store_account_ nft_holder '\''{"token_id": "_1_"}'\'' '
  str="${str//_1_/$1}"
  str="${str//_store_account_/$store_account}"
  echo running "$str"
  eval "$str"
}

function revoke_minter() {
  str='near call _store_account_ revoke_minter '\''{"account_id":"_minter_account_"}'\'' --accountId _root_account_ --deposit 0.000000000000000000000001 --gas 200000000000000'
  str="${str//_root_account_/$root_account}"
  str="${str//_store_account_/$store_account}"
  str="${str//_minter_account_/$minter_account}"
  echo running "$str"
  eval "$str"
}

function nft_batch_transfer() {
  str='near call _store_account_ nft_transfer '\''{"token_ids":[["_1_", "_receiver_account_"]]}'\'' --accountId _buyer1_account_ --deposit 0.000000000000000000000001 --gas 200000000000000'
  str="${str//_receiver_account_/$receiver_account}"
  str="${str//_store_account_/$store_account}"
  str="${str//_minter_account_/$minter_account}"
  str="${str//_buyer1_account_/$buyer1_account}"
  str="${str//_1_/$1}"
  echo running "$str"
  eval "$str"

  str='near view _store_account_ nft_holder '\''{"token_id": "_1_"}'\'' '
  str="${str//_1_/$1}"
  str="${str//_store_account_/$store_account}"
  echo running "$str"
  eval "$str"
}

function batch_upgrade_stores() {
  N=5
  for row in $(jq -r '.[]'); do
    ((i = i % N))
    ((i++ == 0)) && wait
    redeploy_single_store "$row" &
  done <scripts/testnet.json
}

function revoke_all_approvals() {
  str='near call _store_account_ nft_revoke_all '\''{"token_id":""}'\'' --accountId _buyer1_account_ --deposit 0.000000000000000000000001 --gas 200000000000000'
  str="${str//_receiver_account_/$receiver_account}"
  str="${str//_store_account_/$store_account}"
  str="${str//_minter_account_/$minter_account}"
  str="${str//_buyer1_account_/$buyer1_account}"
  str="${str//_1_/$1}"
  echo running "$str"
  eval "$str"

  str='near view _store_account_ nft_holder '\''{"token_id": "_1_"}'\'' '
  str="${str//_1_/$1}"
  str="${str//_store_account_/$store_account}"
  echo running "$str"
  eval "$str"
}

function update_list() {
  str='near call _market_account_ update_allowlist '\''{"account_id":"_1_", "state":true}'\'' --accountId _market_account_ --deposit 0.000000000000000000000001 --gas 200000000000000 --nodeUrl _node_url_ --keyPath _key_path_'
  str="${str//_1_/$1}"
  str="${str//_market_account_/$market_account}"
  str="${str//_root_account_/$root_account}"
  str="${str//_node_url_/$node_url}"
  str="${str//_key_path_/$key_path}"
  echo running "$str"
  eval "$str"
}

function get_allow_list() {
  str='near view _market_account_ get_allowlist'
  str="${str//_market_account_/$market_account}"
  echo running "$str"
  eval "$str"
}

function update_ban_list() {
  str='near call _market_account_ update_banlist '\''{"account_id":"_1_", "state":true}'\'' --accountId _market_account_ --deposit 0.000000000000000000000001 --gas 200000000000000 --nodeUrl _node_url_ --keyPath _key_path_'
  str="${str//_1_/$1}"
  str="${str//_market_account_/$market_account}"
  str="${str//_root_account_/$root_account}"
  str="${str//_node_url_/$node_url}"
  str="${str//_key_path_/$key_path}"
  echo running "$str"
  eval "$str"
}

function get_ban_list() {
  str='near view _market_account_ get_banlist'
  str="${str//_market_account_/$market_account}"
  echo running "$str"
  eval "$str"
}

function nft_transfer_call() {
  str='near call _store_account_ nft_transfer_call '\''{"receiver_id":"_helper_account_", "token_id":"_1_", "msg":"true"}'\'' --accountId _minter_account_ --deposit 0.000000000000000000000001 --gas 200000000000000 --keyPath _key_path_ --nodeUrl _node_url_'
  str="${str//_1_/$1}"
  str="${str//_store_account_/$store_account}"
  str="${str//_helper_account_/$helper_account}"
  str="${str//_minter_account_/$minter_account}"
  str="${str//_key_path_/$key_path}"
  str="${str//_node_url_/$node_url}"
  echo running "$str"
  eval "$str"
}

function nft_token() {
  str='near view _store_account_ nft_token '\''{"token_id":"_1_"}'\'' --keyPath _key_path_ --nodeUrl _node_url_'
  str="${str//_1_/$1}"
  str="${str//_store_account_/$store_account}"
  str="${str//_key_path_/$key_path}"
  str="${str//_node_url_/$node_url}"
  echo running "$str"
  eval "$str"
}

function top_stores() {
  str='./bin/stats --stat top-stores'
  echo running "$str"
  eval "$str"
}

. switch-cmd.sh

if [ -n "$1" ]; then
  programa2 $1 $2
else
  programa
fi
