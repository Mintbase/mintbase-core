// import { UnencryptedFileSystemKeyStore } from "near-api-js/lib/key_stores";
import { UnencryptedFileSystemKeyStore } from "near-workspaces/node_modules/near-api-js/lib/key_stores";
import {
  Workspace,
  NearAccount,
  KeyPair,
  NearAccountManager,
} from "near-workspaces-ava";
import {
  batchMint,
  createAccounts,
  deployStore,
  downloadContracts,
  failPromiseRejection,
  mNEAR,
  NEAR,
  Tgas,
} from "./test-utils";
import { writeFile, readdir } from "fs/promises";
import * as path from "path";

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
  const store = await (async () => {
    let store = await deployStore({
      factory: factory,
      owner: alice,
      name: "alice",
    });
    // patch the key for the store
    // @ts-ignore (touching the private parts of root)
    await copyKeyInFs(root.manager.keyStore, factory, store);
    // TODO: scrutinize if we really need to reload the account
    // TODO: replace the signer
    // // @ts-ignore (touching the private parts of store)
    // store.manager as NearAccountManager;
    //store = factory.getFullAccount(store.accountId);
    return store;
  })();
  test.log("created store");

  const market = await root.createAndDeploy(
    "market",
    "./downloads/mainnet-market.wasm",
    { method: "new", args: { init_allowlist: [] } }
  );
  test.log("created market");

  // get pre-update state
  const referenceState = await createState({
    root,
    alice,
    store,
    market,
    factory,
  }).catch(failPromiseRejection(test, "creating state"));
  test.log("created state");

  // upgrade contracts
  // FIXME: cannot find the damn key
  await updateContract(store, "store");
  test.log("updated store");
  await updateContract(factory, "factory");
  test.log("updated factory");
  await updateContract(market, "market");
  test.log("updated market");

  // compare pre- and post-upgrade states
  const currentState = await queryState({
    root,
    alice,
    store,
    market,
    factory,
  });
  test.deepEqual(currentState, referenceState);
});

type StateSnapshot = any;

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
    { account_id: factory.accountId, state: true },
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
  const aliceDeployed = await factory.view("check_contains_store", {
    store_id: store.accountId,
  });
  const bobDeployed = await factory.view("check_contains_store", {
    store_id: "bob.factory.test.near",
  });

  // query token data
  const tokenData = await store.view("nft_token", { token_id: "0" });

  // query market allowlist
  const marketAllowlist = await market.view("get_allowlist");

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
  console.log("tx created");
  // const key = await contract.getKey();
  // console.log("got key:", key);
  await tx.signAndSend();
  console.log("tx sent");
}

async function copyKeyInFs(
  keystore: UnencryptedFileSystemKeyStore,
  from: NearAccount,
  to: NearAccount
) {
  const dir = path.join(keystore.keyDir, "sandbox");
  console.log("directory name: ", dir);
  console.log("before patching: ", await readdir(dir));
  const key: KeyPair = await keystore.getKey("sandbox", from.accountId);
  // const content = {
  //   account_id: to.accountId,
  //   public_key: key.getPublicKey().toString(),
  //   private_key: key.toString(),
  // };
  // await writeFile(
  //   `${keystore.keyDir}/sandbox/${to.accountId}.json`,
  //   JSON.stringify(content),
  //   { mode: 0o600 }
  // );
  await keystore.setKey("sandbox", to.accountId, key);
  console.log("after patching: ", await readdir(dir));

  // replace the damn key store (maybe stuff is being cached?)
  // FIXME: it looks like we need to replace the signer, not the store
  // @ts-ignore
  to.manager.config.keyStore = new UnencryptedFileSystemKeyStore(
    // @ts-ignore
    to.manager.keyStore.keyDir
  );

  console.log("got new key store");
}
