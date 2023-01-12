import { TestFn } from "ava";
import { Worker, NearAccount } from "near-workspaces";
import * as nearAPI from "near-api-js";
import { DEPLOY_STORE_RENT, DEPLOY_STORE_GAS } from "./utils/balances.js";

const createSubaccount = async (
  root: NearAccount,
  name: string,
  { initialBalanceNear }: { initialBalanceNear: string }
): Promise<NearAccount> =>
  root.createAccount(`${name}.${root.accountId}`, {
    initialBalance: nearAPI.utils.format.parseNearAmount(
      initialBalanceNear
    ) as string,
  });

export const createAndDeploy = async (
  root: NearAccount,
  name: string,
  args: {
    initialBalanceNear: string;
    codePath: string;
    initMethod: string;
    initArgs: any;
  }
): Promise<NearAccount> => {
  const { codePath, initMethod, initArgs } = args;
  const account = await createSubaccount(root, name, args);
  await account.deploy(codePath);
  await account.call(account, initMethod, initArgs);
  return account;
};

export const deployStore = async ({
  factory,
  owner,
  name,
}: {
  factory: NearAccount;
  owner: NearAccount;
  name: string;
}): Promise<NearAccount> => {
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
  return factory.getAccount(`${name}.${factory.accountId}`);
};

type TestContext = {
  worker: Worker;
  accounts: Record<string, NearAccount>;
};

export const setup = (test: TestFn): TestFn<TestContext> => {
  test.beforeEach(async (t) => {
    const worker = await Worker.init();
    const root = worker.rootAccount;
    const alice = await createSubaccount(root, "alice", {
      initialBalanceNear: "20",
    });
    const bob = await createSubaccount(root, "bob", {
      initialBalanceNear: "20",
    });
    const carol = await createSubaccount(root, "carol", {
      initialBalanceNear: "20",
    });
    const dave = await createSubaccount(root, "dave", {
      initialBalanceNear: "20",
    });

    const factory = await createAndDeploy(root, "factory", {
      initialBalanceNear: "10",
      codePath: "../wasm/factory.wasm",
      initMethod: "new",
      initArgs: {},
    });
    // const store = await createAndDeploy(root, "store", {
    //   initialBalanceNear: "10",
    //   codePath: "../wasm/store.wasm",
    //   initMethod: "new",
    //   initArgs: {
    //     owner_id: root,
    //     metadata: {
    //       spec: "nft-1.0.0",
    //       name: `store.${root}`,
    //       symbol: "STORE",
    //     },
    //   },
    // });
    const market = await createAndDeploy(root, "market", {
      initialBalanceNear: "10",
      codePath: "../wasm/market.wasm",
      initMethod: "new",
      initArgs: { init_allowlist: [] },
    });

    const store = await deployStore({ owner: alice, factory, name: "alice" });

    (t.context as TestContext).worker = worker;
    (t.context as TestContext).accounts = {
      root,
      alice,
      bob,
      carol,
      dave,
      factory,
      store,
      market,
    };
  });

  test.afterEach(async (t) => {
    await (t.context as TestContext).worker.tearDown().catch((e) => {
      console.log("Failed to tear down the worker:", e);
    });
  });

  return test as TestFn<TestContext>;
};
export default setup;
