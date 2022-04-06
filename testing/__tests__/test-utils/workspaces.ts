import { NearAccount, Workspace } from "near-workspaces-ava";
import { NEAR, DEPLOY_STORE_RENT, DEPLOY_STORE_GAS } from "./balances";

export async function createAccounts(
  root: NearAccount
): Promise<NearAccount[]> {
  // const alice = await root.createAccount("alice", { initialBalance: NEAR(20) });
  // const bob = await root.createAccount("bob", { initialBalance: NEAR(20) });
  // const carol = await root.createAccount("carol", { initialBalance: NEAR(20) });
  // return [alice, bob, carol];
  return Promise.all([
    root.createAccount("alice", { initialBalance: NEAR(20).toString() }),
    root.createAccount("bob", { initialBalance: NEAR(20).toString() }),
    root.createAccount("carol", { initialBalance: NEAR(20).toString() }),
    root.createAccount("dave", { initialBalance: NEAR(20).toString() }),
  ]);
}

/** deploys the factory to a subaccount `factory` of `root` */
export async function deployFactory(root: NearAccount): Promise<NearAccount> {
  return root.createAndDeploy(
    "factory", // subaccount name
    "../wasm/factory.wasm", // path to wasm
    { method: "new", args: {} }
  );
}

/** deploys the market to a subaccount `market` of `root` */
export async function deployMarket(root: NearAccount): Promise<NearAccount> {
  return root.createAndDeploy(
    "market", // subaccount name
    "../wasm/market.wasm", // path to wasm
    { method: "new", args: { init_allowlist: [] } }
  );
}

/**
 * deploys the store to a subaccount `name` of `factory`, setting the store
 * owner to `owner`
 */
export async function deployStore({
  factory,
  owner,
  name,
}: {
  factory: NearAccount;
  owner: NearAccount;
  name: string;
}): Promise<NearAccount> {
  await owner.call(
    factory,
    "create_store",
    {
      owner_id: owner.accountId,
      metadata: {
        spec: "nft-1.0.0",
        name,
        symbol: "ALICE",
      },
    },
    { attachedDeposit: DEPLOY_STORE_RENT, gas: DEPLOY_STORE_GAS }
  );
  return factory.getFullAccount(`${name}.${factory.accountId}`);
}

/** A workspace with the factory deployed by root, no store deployed */
export const FACTORY_WORKSPACE = Workspace.init(async ({ root }) => {
  const [alice, bob, carol, dave] = await createAccounts(root);

  const factory = await deployFactory(root);

  return { alice, bob, carol, dave, factory };
});

/** A workspace with the factory deployed by root, store deployed by Alice */
export const STORE_WORKSPACE = Workspace.init(async ({ root }) => {
  const [alice, bob, carol, dave] = await createAccounts(root);

  const factory = await deployFactory(root);
  const store = await deployStore({ factory, owner: alice, name: "alice" });

  return { alice, bob, carol, dave, factory, store };
});

/**
 * A workspace with the factory and market deployed by root,
 * store deployed by Alice
 */
export const MARKET_WORKSPACE = Workspace.init(async ({ root }) => {
  const [alice, bob, carol, dave] = await createAccounts(root);

  const factory = await deployFactory(root);
  const store = await deployStore({ factory, owner: alice, name: "alice" });
  const market = await deployMarket(root);

  return { alice, bob, carol, dave, factory, store, market };
});
