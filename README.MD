**[Click here for Docs](https://mintbase-core-docs.netlify.app/mintbase_deps/)**
****

A work in progress


**./run.sh**

`./run.sh` is a interactive script to help 
you run, deploy and test mintbase smart
contracts. 

![](mintbase-core.gif)

**Setup**

Create `scripts/.postgres.sh` with the
following credentials

```shell
postgres_user=postgres;
postgres_password=postgres;
postgres_host=127.0.0.1;
postgres_database=mintlivebase;
```

Specify the environment

`NETWORK=local ./run.sh`

`NETWORK=testnet ./run.sh`

`NETWORK=mainnet ./run.sh`

You can export the env variable for 
convenience

**Recipes**

`./run.sh run-indexer`
    - runs local indexer


`./run.sh build-contracts` 
    - build smart contracts

`./run.sh build-contracts`
    - build smart contracts

`./run.sh create-accounts`
    - creates required accounts

`./run.sh deploy`
    - deploys smart contracts

`./run.sh create-store`
    - creates mintbase store

