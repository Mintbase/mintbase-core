. scripts/env.sh;

export NEAR_ENV=$network;

near_dir=~/.near/$network;

if [ "$network" = "testnet" ]; then
  node_url="https://rpc.testnet.near.org"; #testnet
  elif [ $network == "mainnet" ]; then
    node_url="https://rpc.near.org"; #mainnet
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


#near create-account $root_account --masterAccount $top_level_account;

# Create all accounts needed to test Mintbase
#echo creating accounts;
#for account in $minter_account $store_account $store_owner_account $seller1_account $buyer1_account $buyer2_account; do
#  echo "running: near create-account $account --masterAccount $root_account --initialBalance 10"
#  near create-account $account --masterAccount $root_account --initialBalance 10;
#done;

question=$(cat <<EOF
Type number
(-2.1) init and run indexer
(-2) run indexer
(-1) build contracts
(0) create required accounts
(1) redeploy contracts.
(2) deploy contracts
(3) create store
(4) grant minter permission
(5) mint 10 tokens with no royalty
(6) mint 10 tokens with royalty
(7) approve nft to be market listed with auto-transfer
(8) approve nft to be market listed without auto-transfer
(9) make offer to buy nft
(10) accept offer and transfer nft
(11) revoke minter permissions
(12) Batch transfer nft tokens
(13) Batch upgrade stores
(14) Revoke all approvals
EOF
)


function build_contracts(){
  cd mintbase-deps && cargo market-wasm && cargo store-wasm && cargo factory-wasm && cd ../;
#echo $root_account;
#echo 333;
}

function init_and_run_indexer() {
#    cargo indexer;
    str='rm -rf _near_dir_';
    str="${str//_near_dir_/$near_dir}";
    echo $str;
    eval $str;

    str='./indexer --home-dir _near_dir_ init --chain-id _NEAR_ENV_;';
    str="${str//_near_dir_/$near_dir}";
    str="${str//_NEAR_ENV_/$NEAR_ENV}";
    echo $str;
    eval $str;

    sed -i 's/"tracked_shards": \[\],/"tracked_shards": [0],/g' ~/.near/$network/config.json;

#    str='MALLOC_CONF=prof_leak:true,lg_prof_sample:0,prof_final:true NETWORK=_network_ POSTGRES=postgres://_postgres_user_:_postgres_password_@localhost:5432/mintlivebase WATCH_ACCOUNTS=_root_ ./indexer --home-dir _near_dir_ run;';
    str='NETWORK=_network_ POSTGRES=postgres://_postgres_user_:_postgres_password_@localhost:5432/mintlivebase WATCH_ACCOUNTS=_root_ ./indexer --home-dir _near_dir_ run;';
    str="${str//_near_dir_/$near_dir}";
    str="${str//_root_/$root}";
    str="${str//_network_/$network}";
    str="${str//_postgres_password_/$postgres_password}";
    str="${str//_postgres_user_/$postgres_user}";

    echo $str;
    eval $str;
}

function run_indexer() {
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
    $royalty2_account $receiver_account; do
      ((z=z%N)); ((z++==0)) && wait
        str="near create-account $i --masterAccount $root_account --initialBalance 3 --nodeUrl $node_url --keyPath $key_path";
        echo running $str;
        eval $str &
    done
}

function deploy() {
  str='near deploy --wasmFile factory.wasm _root_account_ --initFunction new --initArgs null --masterAccount _root_account_ --nodeUrl _node_url_ --keyPath _key_path_';
  str="${str//_root_account_/$root_account}";
  str="${str//_node_url_/$node_url}";
  str="${str//_key_path_/$key_path}";
  echo running "$str";
  eval "$str";
  str='near deploy --wasmFile market.wasm _market_account_ --initFunction new --initArgs '\''{"init_allowlist": ["_root_account_"]}'\'' --masterAccount _root_account_ --nodeUrl _node_url_ --keyPath _key_path_';
  str="${str//_root_account_/$root_account}";
  str="${str//_market_account_/$market_account}";
  str="${str//_node_url_/$node_url}";
  str="${str//_key_path_/$key_path}";
  echo running "$str";
  eval "$str";
}

function redeploy() {
  str='near deploy --wasmFile factory.wasm _root_account_ --masterAccount _root_account_ --nodeUrl _node_url_';
  str="${str//_root_account_/$root_account}";
  str="${str//_node_url_/$node_url}";
  str="${str//_key_path_/$key_path}";
  echo running "$str";
  eval "$str";

  str='near deploy --wasmFile market.wasm _market_account_ --masterAccount _root_account_ --nodeUrl _node_url_';
  str="${str//_root_account_/$root_account}";
  str="${str//_market_account_/$market_account}";
  str="${str//_node_url_/$node_url}";
  str="${str//_key_path_/$key_path}";
  echo running "$str";
  eval "$str";
}

function redeploy_single_store() {
  cred=$(cat ~/.near-credentials/$top_level_account/$root_account.json);
  echo creating credentials "$cred";
  echo "$cred" > ~/.near-credentials/"$network"/"$1".json
  str='near deploy --wasmFile store.wasm _1_ --masterAccount _root_account_';
  str="${str//_root_account_/$root_account}";
  str="${str//_1_/$1}";
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
  str='near call _market_account_ make_offer '\''{"token_key":"_1_:_store_account_", "price":"1000000000000000000000000", "timeout":{"Hours":24} }'\'' --accountId _buyer1_account_ --deposit 1 --gas 200000000000000 --nodeUrl _node_url_ --keyPath _key_path_';
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
  done < ./stores.json
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

function programa() {
  echo "$question";
  read -r response;
  echo "you chose $response";

  case $response in
  -2.1)
    if [ $network = 'mainnet' ]; then
      echo 'we stopped you from doing something dangerous';
    elif [ $network = 'testnet' ]; then
      echo 'we stopped you from doing something dangerous';
   else
       init_and_run_indexer;
    fi
#    init_and_run_indexer & (sleep 2 && create_accounts && deploy);
    programa;
    ;;
  -2)
    run_indexer &
    programa;
    ;;
  -1)
    build_contracts;
    programa;
    ;;
  0)
    create_accounts;
    programa;
    ;;
  1)
    redeploy;
    programa;
    ;;
  2)
    deploy;
    programa;
    ;;
  3)
    create_store;
    programa;
    ;;
  4)
    grant_minter;
    programa;
    ;;
  5)
    mint_tokens_nr;
    echo "remember token_id to list in marketplace";
    programa;
    ;;
  6)
    mint_tokens;
    echo "remember token_id to list in marketplace";
    programa;
    ;;
  7)
    echo "enter token_id:";
    read -r token_id;
    nft_approve_autotransfer "$token_id";
    programa;
    ;;
  8)
    echo "enter token_id:";
    read -r token_id;
    nft_approve_manual_transfer "$token_id";
    programa;
    ;;
  9)
    echo "token_id:";
    read -r token_id;
    make_offer "$token_id";
    programa;
    ;;
  10)
    echo "token_id:";
    read -r token_id;
    accept_offer_and_transfer "$token_id";
    programa;
    ;;
  11)
    revoke_minter;
    programa;
    ;;
  12)
    echo "token_id:";
    read -r token_id;
    nft_batch_transfer "$token_id";
    programa;
    ;;
  13)
    batch_upgrade_stores;
    programa;
    ;;
  *)
    echo not a command;
    programa
    ;;
  esac
}

if [ -n "$1" ]; then
  echo $1;
  else
  programa
fi




#echo "Hello, who am I talking to?"
#read varname
#echo "It\'s nice to meet you $varname"