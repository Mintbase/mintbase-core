# Mintbase Core

This repo holds the components that make Mintbase run:

- `mintbase-deps` is the SDK that holds code shared between contracts and the
  indexer
- `mintbase-near-indexer` holds our indexer (currently closed-source)
- The smart contracts:
  - `store` is an automatically deployed store that anyone may create
  - `factory` facilitates the deployment of stores
  - `helper` was used in automated testing and is no longer needed, thus
    soon-to-be deprecated
  - `simple-market-contract` holds the Mintbase marketplace, where NFTs may be
    traded (currently closed-source)
- `mintbase-core-docs` holds the documentation, which is currently in migration
  and offline
