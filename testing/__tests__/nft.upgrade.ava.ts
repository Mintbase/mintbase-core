import { Workspace, NearAccount } from "near-workspaces-ava";
import {
  batchMint,
  createAccounts,
  downloadContracts,
  failPromiseRejection,
  mNEAR,
  NEAR,
  Tgas,
} from "./test-utils";

Workspace.init().test("upgrade::mainnet", async (test, { root }) => {
  // download current contracts from blockchain
  await downloadContracts();

  // create accounts
  const [alice] = await createAccounts(root);

  // deploy old factory + store + market
  const factory = await root.createAndDeploy(
    "factory",
    "./downloads/mainnet-factory.wasm",
    { method: "new", args: {} }
  );
  const store = await root.createAndDeploy(
    "store",
    "./downloads/mainnet-store.wasm",
    {
      method: "new",
      args: {
        owner_id: alice.accountId,
        metadata: {
          spec: "nft-1.0.0",
          name: "store",
          symbol: "ALICE",
        },
      },
    }
  );
  const market = await root.createAndDeploy(
    "market",
    "./downloads/mainnet-market.wasm",
    { method: "new", args: { init_allowlist: [] } }
  );

  const accounts = {
    root,
    alice,
    store,
    market,
    factory,
  };

  // get pre-update state
  const referenceState = (await createState(accounts).catch(
    failPromiseRejection(test, "creating state")
  )) as StateSnapshot;

  // upgrade contracts
  await updateContract(store, "store");
  test.log("updated store");
  await updateContract(factory, "factory");
  test.log("updated factory");
  await updateContract(market, "market");
  test.log("updated market");

  // compare pre- and post-upgrade states
  const currentState = await queryState(accounts);

  test.is(
    currentState.aliceDeployed,
    referenceState.aliceDeployed,
    "Bad deployment status for alice"
  );
  test.is(
    currentState.bobDeployed,
    referenceState.bobDeployed,
    "Bad deployment status for bob"
  );
  test.deepEqual(currentState.marketAllowlist, referenceState.marketAllowlist);
  test.deepEqual(currentState.tokenListing, referenceState.tokenListing);

  // The token format did in fact change
  test.deepEqual(
    currentState.tokenData.metadata,
    referenceState.tokenData.metadata
  );
  test.is(
    currentState.tokenData.owner_id,
    referenceState.tokenData.owner_id.Account
  );
  test.is(
    currentState.tokenData.token_id,
    referenceState.tokenData.id.toString()
  );
  test.deepEqual(
    currentState.tokenData.approved_account_ids,
    referenceState.tokenData.approvals
  );
});

interface StateSnapshot {
  aliceDeployed: boolean;
  bobDeployed: boolean;
  tokenData: any;
  marketAllowlist: string[];
  tokenListing: any;
}

interface Accounts {
  root: NearAccount;
  alice: NearAccount;
  store: NearAccount;
  market: NearAccount;
  factory: NearAccount;
}

async function createState(accounts: Accounts): Promise<StateSnapshot> {
  const { root, alice, store, market, factory } = accounts;

  // mint some tokens
  await batchMint({ owner: alice, store, num_to_mint: 2 });

  // set allowlist on market
  await root.call(
    market,
    "update_allowlist",
    { account_id: root.accountId, state: true },
    { attachedDeposit: "1" }
  );

  // list the token
  await alice.call(
    store,
    "nft_approve",
    {
      token_id: "0",
      account_id: market.accountId,
      msg: JSON.stringify({ price: NEAR(1).toString(), autotransfer: true }),
    },
    { attachedDeposit: mNEAR(0.81), gas: Tgas(200) }
  );

  return queryState(accounts);
}

async function queryState(accounts: Accounts): Promise<StateSnapshot> {
  const { store, market, factory } = accounts;

  // query deployed stores
  // (cannot give list because the data structure is a LookupSet)
  const aliceDeployed: boolean = await factory.view("check_contains_store", {
    store_id: store.accountId,
  });
  const bobDeployed: boolean = await factory.view("check_contains_store", {
    store_id: "bob.factory.test.near",
  });

  // query token data
  const tokenData = await store.view("nft_token", {
    token_id: "0",
  });

  // query market allowlist
  const marketAllowlist: string[] = await market.view("get_allowlist");

  // query market listing
  const tokenListing = await market.view("get_token", {
    token_key: `0:${store.accountId}`,
  });

  return {
    aliceDeployed,
    bobDeployed,
    tokenData,
    marketAllowlist,
    tokenListing,
  };
}

async function updateContract(contract: NearAccount, what: string) {
  const tx = await contract
    .createTransaction(contract)
    .deployContractFile(`../wasm/${what}.wasm`);
  await tx.signAndSend();
}
