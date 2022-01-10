. scripts/.env.sh;

export NEAR_ENV=$network;

near_dir=~/.near/$network;

if [ "$network" = "testnet" ]; then
  node_url="https://rpc.testnet.near.org"; #testnet
  elif [ $network == "mainnet" ]; then
    node_url="https://rpc.mainnet.near.org"; #mainnet
  elif [ $network == "local" ]; then
    node_url="http://127.0.0.1:3030"; #local
  else
    echo "invalid network $network";
    exit 1
fi


if [ $network = "local" ]; then
      key_path="~/.near/local/validator_key.json";
    else
      key_path="~/.near-credentials/$network/$root_account.json"
fi

. scripts/.cmd.sh;

#near create-account $root_account --masterAccount $top_level_account;

# Create all accounts needed to test Mintbase
#echo creating accounts;
#for account in $minter_account $store_account $store_owner_account $seller1_account $buyer1_account $buyer2_account; do
#  echo "running: near create-account $account --masterAccount $root_account --initialBalance 10"
#  near create-account $account --masterAccount $root_account --initialBalance 10;
#done;



function run_local_indexer() {
    str='bin/indexer --home-dir _near_dir_ init --chain-id local;';
    str="${str//_near_dir_/$near_dir}";
    str="${str//_NEAR_ENV_/$NEAR_ENV}";
    echo $str;
    eval $str;
    sed -i 's/"tracked_shards": \[\],/"tracked_shards": [0],/g' ~/.near/local/config.json;

    str='MALLOC_CONF=prof_leak:true,lg_prof_sample:0,prof_final:true NETWORK=_network_ POSTGRES=postgres://_postgres_user_:_postgres_password_@localhost:5432/mintlivebase WATCH_ACCOUNTS=_root_ bin/indexer --home-dir _near_dir_ run;';
    str="${str//_near_dir_/$near_dir}";
    str="${str//_root_/$root}";
    str="${str//_network_/$network}";
    str="${str//_postgres_password_/$postgres_password}";
    str="${str//_postgres_user_/$postgres_user}";

    echo $str;
    eval $str;
}


function build_contracts(){
  cd mintbase-deps && cargo market-wasm && cargo store-wasm && cargo factory-wasm && cargo helper-wasm && cd ../;
}

function build_indexer(){
  cargo indexer;
}

function init_indexer() {
    str='bin/indexer --home-dir _near_dir_ init --chain-id _NEAR_ENV_;';
    str="${str//_near_dir_/$near_dir}";
    str="${str//_NEAR_ENV_/$NEAR_ENV}";
    echo $str;
    eval $str;
    sed -i 's/"tracked_shards": \[\],/"tracked_shards": [0],/g' ~/.near/$network/config.json;
}

function run_local_indexer() {
    str='rm -rf _near_dir_';
    str="${str//_near_dir_/$near_dir}";
    echo $str;
    eval $str;

    str='bin/indexer --home-dir _near_dir_ init --chain-id _NEAR_ENV_;';
    str="${str//_near_dir_/$near_dir}";
    str="${str//_NEAR_ENV_/$NEAR_ENV}";
    echo $str;
    eval $str;

    sed -i 's/"tracked_shards": \[\],/"tracked_shards": [0],/g' ~/.near/$network/config.json;

#    str='MALLOC_CONF=prof_leak:true,lg_prof_sample:0,prof_final:true NETWORK=_network_ POSTGRES=postgres://_postgres_user_:_postgres_password_@localhost:5432/mintlivebase WATCH_ACCOUNTS=_root_ ./indexer --home-dir _near_dir_ run;';
    str='RUST_LOG=info NETWORK=_network_ POSTGRES=postgres://_postgres_user_:_postgres_password_@_postgres_host_:5432/_postgres_database_ WATCH_ACCOUNTS=_root_ bin/indexer --home-dir _near_dir_ run > out.log 2> error.log;';
    str="${str//_near_dir_/$near_dir}";
    str="${str//_root_/$root}";
    str="${str//_network_/$network}";
    str="${str//_postgres_password_/$postgres_password}";
    str="${str//_postgres_user_/$postgres_user}";
    str="${str//_postgres_host_/$postgres_host}";
    str="${str//_postgres_database_/$postgres_database}";
    echo $str;
    eval $str;
}

function run_stateful_indexer() {
    str='RUST_LOG=info NETWORK=_network_ POSTGRES=postgres://_postgres_user_:_postgres_password_@_postgres_host_:5432/_postgres_database_ WATCH_ACCOUNTS=_watch_accounts_ bin/indexer --home-dir _near_dir_ run;';
    str="${str//_near_dir_/$near_dir}";
    str="${str//_watch_accounts_/$watch_accounts}";
    str="${str//_network_/$network}";
    str="${str//_postgres_password_/$postgres_password}";
    str="${str//_postgres_user_/$postgres_user}";
    str="${str//_postgres_host_/$postgres_host}";
    str="${str//_postgres_database_/$postgres_database}";
    echo $str;
    eval $str;
}

function run_indexer2() {
#    cargo indexer;
    str='NETWORK=_network_ POSTGRES=postgres://_postgres_user_:_postgres_password_@localhost:5432/mintlivebase WATCH_ACCOUNTS=_root_ ./indexer --home-dir _near_dir_ run;';
    str="${str//_near_dir_/$near_dir}";
    str="${str//_root_/$root}";
    str="${str//_network_/$network}";
    str="${str//_postgres_password_/$postgres_password}";
    str="${str//_postgres_user_/$postgres_user}";
    echo $str;
    eval $str;
}

function create_accounts() {
    N=6
    for i in $minter_account $market_account \
    $store_owner_account $seller1 \
    $buyer1_account $buyer2_account $royalty1_account \
    $royalty2_account $receiver_account $helper_account \
    $royalty3_account $royalty4_account $royalty5_account \
    $royalty6_account $royalty7_account $royalty8_account \
    $royalty9_account $royalty10_account; do
      ((z=z%N)); ((z++==0)) && wait
        str="near create-account $i --masterAccount $root_account --initialBalance 4 --nodeUrl $node_url --keyPath $key_path";
        echo running $str;
        eval $str &
    done
}

function deploy() {
  str='near deploy --wasmFile wasm/factory.wasm _root_account_ --initFunction new --initArgs null --masterAccount _root_account_ --nodeUrl _node_url_ --keyPath _key_path_';
  str="${str//_root_account_/$root_account}";
  str="${str//_node_url_/$node_url}";
  str="${str//_key_path_/$key_path}";
  echo running "$str";
  eval "$str";
  str='near deploy --wasmFile wasm/market.wasm _market_account_ --initFunction new --initArgs '\''{"init_allowlist": ["_root_account_"]}'\'' --masterAccount _root_account_ --nodeUrl _node_url_ --keyPath _key_path_';
  str="${str//_root_account_/$root_account}";
  str="${str//_market_account_/$market_account}";
  str="${str//_node_url_/$node_url}";
  str="${str//_key_path_/$key_path}";
  echo running "$str";
  eval "$str";
  str='near deploy --wasmFile wasm/helper.wasm _root_account_ --initFunction new --initArgs null --masterAccount _root_account_ --nodeUrl _node_url_ --keyPath _key_path_';
  str="${str//_root_account_/$root_account}";
  str="${str//_node_url_/$node_url}";
  str="${str//_key_path_/$key_path}";
  echo running "$str";
  eval "$str";
}

function redeploy() {
  str='near deploy --wasmFile wasm/factory.wasm _root_account_ --masterAccount _root_account_ --nodeUrl _node_url_ --keyPath _key_path_';
  str="${str//_root_account_/$root_account}";
  str="${str//_node_url_/$node_url}";
  str="${str//_key_path_/$key_path}";
  echo running "$str";
  eval "$str";

  str='near deploy --wasmFile wasm/market.wasm _market_account_ --masterAccount _root_account_ --nodeUrl _node_url_ --keyPath _key_path_';
  str="${str//_root_account_/$root_account}";
  str="${str//_market_account_/$market_account}";
  str="${str//_node_url_/$node_url}";
  str="${str//_key_path_/$key_path}";
  echo running "$str";
  eval "$str";

  str='near deploy --wasmFile wasm/helper.wasm _helper_account_ --masterAccount _root_account_ --nodeUrl _node_url_ --keyPath _key_path_';
  str="${str//_root_account_/$root_account}";
  str="${str//_helper_account_/$helper_account}";
  str="${str//_node_url_/$node_url}";
  str="${str//_key_path_/$key_path}";
  echo running "$str";
  eval "$str";
}

function redeploy_single_store() {
  cred=$(cat ~/.near-credentials/$network/$root_account.json);
  echo creating credentials "$cred";
  echo "$cred" > ~/.near-credentials/"$network"/"$1".json
  str='near deploy --wasmFile wasm/store.wasm _1_ --masterAccount _root_account_ --nodeUrl _node_url_ --keyPath _key_path_';
  str="${str//_root_account_/$root_account}";
  str="${str//_1_/$1}";
  str="${str//_node_url_/$node_url}";
  str="${str//_key_path_/$key_path}";
  echo running "$str";
  eval "$str";
}


function create_store() {
  store="${store_account/.$root_account/}";
  str='near call _root_account create_store '\''{"metadata":{"spec":"nft-1.0.0","name":"_store_account_","symbol":"A","icon":null,"base_uri":null,"reference":null,"reference_hash":null},"owner_id":"_root_account"}'\'' --accountId _root_account --deposit 7 --gas 200000000000000 --nodeUrl _node_url_ --keyPath _key_path_';
  str="${str//_root_account/$root_account}";
  str="${str//_store_account_/$store}"
  str="${str//_node_url_/$node_url}";
  str="${str//_key_path_/$key_path}";
  echo running "$str";
  eval "$str";
}

function grant_minter() {
  str='near call _store_account_ grant_minter '\''{"account_id":"_minter_"}'\'' --accountId _root_account_ --deposit 0.000000000000000000000001 --nodeUrl _node_url_ --keyPath _key_path_';
  str="${str//_minter_/$minter_account}";
  str="${str//_root_account_/$root_account}";
  str="${str//_store_account_/$store_account}";
  str="${str//_node_url_/$node_url}";
  str="${str//_key_path_/$key_path}";
  echo running "$str";
  eval "$str";
}


function mint_tokens_nr() {
  str='near call _store_account_ nft_batch_mint '\''{"owner_id":"_minter_account_", "metadata":{"spec":"","name":"","symbol":"","icon":null,"base_uri":null,"reference":null,"reference_hash":null},"royalty_args":null,"num_to_mint":10,"split_owners":null}'\'' --accountId _minter_account_ --deposit 0.000000000000000000000001 --nodeUrl _node_url_ --keyPath _key_path_';
  str="${str//_root_account_/$root_account}";
  str="${str//_minter_account_/$minter_account}";
  str="${str//_store_account_/$store_account}";
  str="${str//_node_url_/$node_url}";
  str="${str//_key_path_/$key_path}";
  echo running "$str";
  eval "$str";
}

function mint_tokens() {
  str='near call _store_account_ nft_batch_mint '\''{"owner_id":"_minter_account_", "metadata":{"spec":"","name":"","symbol":"","icon":null,"base_uri":null,"reference":null,"reference_hash":null},"royalty_args":{"split_between": {"_royalty1_account_": 8000,"_royalty2_account_": 2000}, "percentage": 1000},"num_to_mint":10,"split_owners":{"_minter_account_": 8000,"_store_owner_account_": 2000}}'\'' --accountId _minter_account_ --deposit 0.000000000000000000000001';
  str="${str//_minter_account_/$minter_account}";
  str="${str//_store_owner_account_/$store_owner_account}";
  str="${str//_store_account_/$store_account}";
  str="${str//_royalty1_account_/$royalty1_account}";
  str="${str//_royalty2_account_/$royalty2_account}";
  echo running "$str";
  eval "$str";
}

function mint_tokens2() {
  str='near call _store_account_ nft_batch_mint '\''{"owner_id":"_minter_account_", "metadata":{"spec":"","name":"","symbol":"","icon":null,"base_uri":null,"reference":null,"reference_hash":null},"royalty_args":{"split_between": {"_royalty1_account_": 1000,"_royalty2_account_": 1000, "_royalty3_account_": 1000,"_royalty4_account_": 1000,"_royalty5_account_": 1000,"_royalty6_account_": 1000,"_royalty7_account_": 1000,"_royalty8_account_": 1000,"_royalty9_account_": 1000,"_royalty10_account_": 1000}, "percentage": 1000},"num_to_mint":10,"split_owners":{"_minter_account_": 8000,"_store_owner_account_": 2000}}'\'' --accountId _minter_account_ --deposit 0.000000000000000000000001';
  str="${str//_minter_account_/$minter_account}";
  str="${str//_store_owner_account_/$store_owner_account}";
  str="${str//_store_account_/$store_account}";
  str="${str//_royalty1_account_/$royalty1_account}";
  str="${str//_royalty2_account_/$royalty2_account}";
  str="${str//_royalty3_account_/$royalty3_account}";
  str="${str//_royalty4_account_/$royalty4_account}";
  str="${str//_royalty5_account_/$royalty5_account}";
  str="${str//_royalty6_account_/$royalty6_account}";
  str="${str//_royalty7_account_/$royalty7_account}";
  str="${str//_royalty8_account_/$royalty8_account}";
  str="${str//_royalty9_account_/$royalty9_account}";
  str="${str//_royalty10_account_/$royalty10_account}";
  echo running "$str";
  eval "$str";
}

function nft_approve_autotransfer() {
  str='near call _store_account_ nft_approve '\''{"token_id":"_1_", "account_id":"_market_account_", "msg":"{\"price\":\"1000000000000000000000000\",\"autotransfer\":true}" }'\'' --accountId _minter_account_ --deposit 1 --gas 200000000000000 --nodeUrl _node_url_ --keyPath _key_path_';
  str="${str//_1_/$1}";
  str="${str//_market_account_/$market_account}";
  str="${str//_store_account_/$store_account}";
  str="${str//_minter_account_/$minter_account}";
  str="${str//_node_url_/$node_url}";
  str="${str//_key_path_/$key_path}";
  echo running "$str";
  eval "$str";
}

function nft_approve_manual_transfer() {
  str='near call _store_account_ nft_approve '\''{"token_id":"_1_", "account_id":"_market_account_", "msg":"{\"price\":\"1000000000000000000000000\",\"autotransfer\":false}" }'\'' --accountId _minter_account_ --deposit 1 --gas 200000000000000';
  str="${str//_1_/$1}";
  str="${str//_market_account_/$market_account}";
  str="${str//_store_account_/$store_account}";
  str="${str//_minter_account_/$minter_account}";
  echo running "$str";
  eval "$str";
}

function make_offer() {
  str='near call _market_account_ make_offer '\''{"token_key":["_1_:_store_account_"], "price":["1000000000000000000000000"], "timeout":[{"Hours":24}] }'\'' --accountId _buyer1_account_ --deposit 1 --gas 200000000000000 --nodeUrl _node_url_ --keyPath _key_path_';
  str="${str//_1_/$1}";
  str="${str//_market_account_/$market_account}";
  str="${str//_store_account_/$store_account}";
  str="${str//_buyer1_account_/$buyer1_account}";
  str="${str//_node_url_/$node_url}";
  str="${str//_key_path_/$key_path}";
  echo running "$str";
  eval "$str";

  str='near view _store_account_ nft_holder '\''{"token_id": "_1_"}'\''  --nodeUrl _node_url_ --keyPath _key_path_';
  str="${str//_1_/$1}";
  str="${str//_store_account_/$store_account}";
  str="${str//_node_url_/$node_url}";
  str="${str//_key_path_/$key_path}";
  echo running "$str";
  eval "$str";
}



function accept_offer_and_transfer() {
  str='near call _market_account_ accept_and_transfer '\''{"token_key":"_1_:_store_account_"}'\'' --accountId _minter_account_ --deposit 0.000000000000000000000001 --gas 200000000000000';
  str="${str//_1_/$1}";
  str="${str//_market_account_/$market_account}";
  str="${str//_store_account_/$store_account}";
  str="${str//_minter_account_/$minter_account}";
  echo running "$str";
  eval "$str";

  str='near view _store_account_ nft_holder '\''{"token_id": "_1_"}'\'' '
  str="${str//_1_/$1}";
  str="${str//_store_account_/$store_account}";
  echo running "$str";
  eval "$str";
}

function revoke_minter() {
  str='near call _store_account_ revoke_minter '\''{"account_id":"_minter_account_"}'\'' --accountId _root_account_ --deposit 0.000000000000000000000001 --gas 200000000000000';
  str="${str//_root_account_/$root_account}";
  str="${str//_store_account_/$store_account}";
  str="${str//_minter_account_/$minter_account}";
  echo running "$str";
  eval "$str";
}

#  pub fn nft_batch_transfer(&mut self, token_ids: Vec<(U64, AccountId)>) {
# [[112, "_receiver_account_]]

function nft_batch_transfer() {
  str='near call _store_account_ nft_transfer '\''{"token_ids":[["_1_", "_receiver_account_"]]}'\'' --accountId _buyer1_account_ --deposit 0.000000000000000000000001 --gas 200000000000000';
  str="${str//_receiver_account_/$receiver_account}";
  str="${str//_store_account_/$store_account}";
  str="${str//_minter_account_/$minter_account}";
  str="${str//_buyer1_account_/$buyer1_account}";
  str="${str//_1_/$1}";
  echo running "$str";
  eval "$str";

  str='near view _store_account_ nft_holder '\''{"token_id": "_1_"}'\'' '
  str="${str//_1_/$1}";
  str="${str//_store_account_/$store_account}";
  echo running "$str";
  eval "$str";
}

function batch_upgrade_stores() {
  N=5
  for row in $(jq -r '.[]'); do
    ((i=i%N)); ((i++==0)) && wait
      redeploy_single_store "$row" &
  done < scripts/testnet.json
}

function revoke_all_approvals() {
  str='near call _store_account_ nft_revoke_all '\''{"token_id":""}'\'' --accountId _buyer1_account_ --deposit 0.000000000000000000000001 --gas 200000000000000';
  str="${str//_receiver_account_/$receiver_account}";
  str="${str//_store_account_/$store_account}";
  str="${str//_minter_account_/$minter_account}";
  str="${str//_buyer1_account_/$buyer1_account}";
  str="${str//_1_/$1}";
  echo running "$str";
  eval "$str";

  str='near view _store_account_ nft_holder '\''{"token_id": "_1_"}'\'' '
  str="${str//_1_/$1}";
  str="${str//_store_account_/$store_account}";
  echo running "$str";
  eval "$str";
}

function update_list(){
    str='near call _market_account_ update_allowlist '\''{"account_id":"_1_", "state":true}'\'' --accountId _market_account_ --deposit 0.000000000000000000000001 --gas 200000000000000';
    str="${str//_1_/$1}";
    str="${str//_market_account_/$market_account}";
    str="${str//_root_account_/$root_account}";
    echo running "$str";
    eval "$str";
}

function get_allow_list(){
    str='near view _market_account_ get_allowlist';
    str="${str//_market_account_/$market_account}";
    echo running "$str";
    eval "$str";
}

function update_ban_list(){
    str='near call _market_account_ update_banlist '\''{"account_id":"_1_", "state":false}'\'' --accountId _market_account_ --deposit 0.000000000000000000000001 --gas 200000000000000';
    str="${str//_1_/$1}";
    str="${str//_market_account_/$market_account}";
    str="${str//_root_account_/$root_account}";
    echo running "$str";
    eval "$str";
}

function get_ban_list(){
    str='near view _market_account_ get_banlist';
    str="${str//_market_account_/$market_account}";
    echo running "$str";
    eval "$str";
}

function nft_transfer_call(){
    str='near call _store_account_ nft_transfer_call '\''{"receiver_id":"_helper_account_", "token_id":"_1_", "msg":"true"}'\'' --accountId _minter_account_ --deposit 0.000000000000000000000001 --gas 200000000000000 --keyPath _key_path_ --nodeUrl _node_url_';
    str="${str//_1_/$1}";
    str="${str//_store_account_/$store_account}";
    str="${str//_helper_account_/$helper_account}";
    str="${str//_minter_account_/$minter_account}";
    str="${str//_key_path_/$key_path}";
    str="${str//_node_url_/$node_url}";
    echo running "$str";
    eval "$str";
}

if [ -n "$1" ]; then
  echo $1;
  programa $1;
  else
  programa
fi




#echo "Hello, who am I talking to?"
#read varname
#echo "It\'s nice to meet you $varname"