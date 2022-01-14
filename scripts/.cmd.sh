question=$(
  cat <<EOF
Type number
(-2.3) init indexer
(-2.2) run stateful indexer
(-2.1) init and run indexer
(-2) run indexer
(-1) build contracts
(-1.1) build indexer
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
(12.1) Nft transfer call
(13) Batch upgrade stores
(14) Revoke all approvals
(15) Update market allow list
(16) Get allow list
(17) Update ban list
(18) Get ban list
EOF
)

function programa() {
  echo "$question"
  read -r response
  echo "you chose $response"
  case $response in
  -2.3)
    init_indexer
    programa
    ;;
  -2.2)
    echo "are you sure? y/n"
    read -r answer
    if [ $answer = 'y' ]; then
      run_stateful_indexer
    fi
    programa
    ;;
  -2.1)
    if [ $network = 'mainnet' ]; then
      echo 'we stopped you from doing something dangerous'
    elif [ $network = 'testnet' ]; then
      echo 'we stopped you from doing something dangerous'
    else
      init_and_run_indexer
    fi
    #    init_and_run_indexer & (sleep 2 && create_accounts && deploy);
    programa
    ;;
  -2)
    run_local_indexer &
    programa
    ;;
  -1.1)
    build_indexer
    programa
    ;;
  -1)
    build_contracts
    programa
    ;;
  0)
    create_accounts
    programa
    ;;
  1)
    redeploy
    programa
    ;;
  2)
    #    deploy;
    programa
    ;;
  3)
    create_store
    programa
    ;;
  4)
    grant_minter
    programa
    ;;
  5)
    mint_tokens_nr
    echo "remember token_id to list in marketplace"
    programa
    ;;
  6)
    mint_tokens
    echo "remember token_id to list in marketplace"
    programa
    ;;
  7)
    echo "enter token_id:"
    read -r token_id
    nft_approve_autotransfer "$token_id"
    programa
    ;;
  8)
    echo "enter token_id:"
    read -r token_id
    nft_approve_manual_transfer "$token_id"
    programa
    ;;
  9)
    echo "token_id:"
    read -r token_id
    make_offer "$token_id"
    programa
    ;;
  10)
    echo "token_id:"
    read -r token_id
    accept_offer_and_transfer "$token_id"
    programa
    ;;
  11)
    revoke_minter
    programa
    ;;
  12)
    echo "token_id:"
    read -r token_id
    nft_batch_transfer "$token_id"
    programa
    ;;
  12.1)
    echo "token_id:"
    read -r token_id
    nft_transfer_call "$token_id"
    programa
    ;;
  13)
    batch_upgrade_stores
    programa
    ;;
  15)
    echo "account_id:"
    read -r account_id
    update_list $account_id
    programa
    ;;
  16)
    get_allow_list
    programa
    ;;
  17)
    echo "account_id:"
    read -r account_id
    update_ban_list $account_id
    programa
    ;;
  18)
    get_ban_list
    programa
    ;;
  *)
    echo not a command
    programa
    ;;
  esac
}

function programa2() {
  case $1 in
  "git-pull")
    git pull
    cd mintbase-near-indexer && git checkout localnet && git pull && cd ../
    cd simple-market-contract && git checkout localnet && git pull && cd ../
    ;;
  "e2e")
    build_indexer
    build_contracts
    run_local_indexer &
    create_accounts >>out.log 2>>error.log
    deploy >>out.log 2>>error.log
    create_store >>out.log 2>>error.log
    grant_minter
    mint_tokens_custom '{"owner_id":"_minter_account_", "metadata":{"spec":"","name":"","symbol":"","icon":null,"base_uri":null,"reference":null,"reference_hash":null},"royalty_args":{"split_between": {"_royalty1_account_": 8000,"_royalty2_account_": 2000}, "percentage": 1000},"num_to_mint":50,"split_owners":{"_minter_account_": 8000,"_store_owner_account_": 2000}}'
    ;;
  "e2e-loop")
    while :; do
      e2e $2
    done
    ;;
  "indexer")
    run_stateful_indexer $2
    ;;
  "run-indexer")
    run_local_indexer $2
    ;;
  "build-indexer")
    build_indexer
    ;;
  "build-contracts")
    build_contracts
    ;;
  "create-accounts")
    create_accounts
    ;;
  "redeploy")
    redeploy
    ;;
  "deploy")
    deploy
    ;;
  "create-store")
    create_store
    ;;
  "grant-minter")
    grant_minter
    ;;
  "send-store")
      send_to_store;
      ;;
  "mint-tokens")
    mint_tokens_custom '{"owner_id":"_minter_account_", "metadata":{"spec":"","name":"","symbol":"","icon":null,"base_uri":null,"reference":null,"reference_hash":null},"royalty_args":{"split_between": {"_royalty1_account_": 8000,"_royalty2_account_": 2000}, "percentage": 1000},"num_to_mint":100,"split_owners":{"_minter_account_": 8000,"_store_owner_account_": 2000}}'
    echo "remember token_id to list in marketplace"
    ;;
    #  6)
    #    mint_tokens;
    #    echo "remember token_id to list in marketplace";
    #    programa;
    #    ;;
    #  7)
    #    echo "enter token_id:";
    #    read -r token_id;
    #    nft_approve_autotransfer "$token_id";
    #    programa;
    #    ;;
    #  8)
    #    echo "enter token_id:";
    #    read -r token_id;
    #    nft_approve_manual_transfer "$token_id";
    #    programa;
    #    ;;
    #  9)
    #    echo "token_id:";
    #    read -r token_id;
    #    make_offer "$token_id";
    #    programa;
    #    ;;
    #  10)
    #    echo "token_id:";
    #    read -r token_id;
    #    accept_offer_and_transfer "$token_id";
    #    programa;
    #    ;;
    #  11)
    #    revoke_minter;
    #    programa;
    #    ;;
    #  12)
    #    echo "token_id:";
    #    read -r token_id;
    #    nft_batch_transfer "$token_id";
    #    programa;
    #    ;;
    #  12.1)
    #    echo "token_id:";
    #    read -r token_id;
    #    nft_transfer_call "$token_id";
    #    programa;
    #    ;;
    #  13)
    #    batch_upgrade_stores;
    #    programa;
    #    ;;
    #  15)
    #    echo "account_id:";
    #    read -r account_id;
    #    update_list $account_id;
    #    programa;
    #    ;;
    #  16)
    #    get_allow_list;
    #    programa;
    #    ;;
    #  17)
    #    echo "account_id:";
    #    read -r account_id;
    #    update_ban_list $account_id;
    #    programa;
    #    ;;
    #  18)
    #    get_ban_list;
    #    programa;
    #    ;;
  *)
    echo not a command
    ;;
  esac
}
