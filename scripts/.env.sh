## root
# ----
# usually factory account
# `test` if `network` is local

## top level account
# ----
# depends on network
# `near` if `network` is local or mainnet
# `testnet` `network` is testnet

. scripts/.postgres.sh;

# localnet
network="local";
top_level_account="near"
root="test"

# testnet
#network="testnet";
#top_level_account="testnet"
#root="mintspace2"

# mainnet
#network="mainnet";
#top_level_account="near"
#root="mintbase1"

misfits_root="tenk"
nmkmint="nmkmint"
watch_accounts="$root,$misfits_root,$nmkmint"
postgres_user=$_postgres_user;
postgres_password=$_postgres_password;
postgres_host=$_postgres_host;
postgres_database=$_postgres_database;
rust_log="tokio_reactor=info,near=info,near=error,stats=info,telemetry=info,indexer_for_wallet=info,mintbase_near_indexer=info,near_indexer=debug"

. scripts/.data.sh;