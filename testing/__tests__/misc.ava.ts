import { TransactionResult } from "near-workspaces";
import avaTest from "ava";
import {
  NEAR,
  mNEAR,
  uNEAR,
  nNEAR,
  assertEventLogs,
  failPromiseRejection,
  assertMinters,
  assertContractPanics,
} from "./utils/index.js";
import { setup } from "./setup.js";

const test = setup(avaTest);

// No need to fire up the chain for testing my utils
avaTest("util tests", (test) => {
  test.is(NEAR(1.5).toString(), "1500000000000000000000000");
  test.is(mNEAR(1.5).toString(), "1500000000000000000000");
  test.is(uNEAR(1.5).toString(), "1500000000000000000");
  test.is(nNEAR(1.5).toString(), "1500000000000000");
});

const changeSettingsData = (subset: Record<string, string>) => {
  const data: Record<string, string | null> = {
    granted_minter: null,
    revoked_minter: null,
    new_icon_base64: null,
    new_owner: null,
    new_base_uri: null,
  };

  Object.keys(subset).forEach((k) => {
    data[k] = subset[k];
  });

  return data;
};

// // As this tests deployment, we do it in a clean-state environment
// Workspace.init().test("deployment", async (test, { root }) => {
//   const failDeploymentError = (name: string) => (e: any) => {
//     test.log(`Failed to deploy ${name} contract`);
//     test.log(e);
//     test.fail();
//   };

//   await root
//     .createAndDeploy(
//       "factory", // subaccount name
//       "../wasm/factory.wasm", // path to wasm
//       { method: "new", args: {} }
//     )
//     .catch(failDeploymentError("factory"));

//   await root
//     .createAndDeploy("store", "../wasm/store.wasm", {
//       method: "new",
//       args: {
//         owner_id: root.accountId,
//         metadata: {
//           spec: "nft-1.0.0",
//           name: "store.root",
//           symbol: "ROOT",
//         },
//       },
//     })
//     .catch(failDeploymentError("store"));

//   await root
//     .createAndDeploy("helper", "../wasm/helper.wasm", {
//       method: "new",
//       args: {},
//     })
//     .catch(failDeploymentError("helper"));

//   await root
//     .createAndDeploy("market", "../wasm/market.wasm", {
//       method: "new",
//       args: { init_allowlist: [] },
//     })
//     .catch(failDeploymentError("market"));
// });

test("ownership::transfer-store", async (test) => {
  const { alice, bob, carol, store } = test.context.accounts;

  await alice
    .call(store, "grant_minter", { account_id: bob }, { attachedDeposit: "1" })
    .catch(failPromiseRejection(test, "granting minter rights"));

  // ---------------------------- remove minters -----------------------------
  const transferStoreClearMintersCall = await alice
    .callRaw(
      store,
      "transfer_store_ownership",
      { new_owner: carol.accountId, keep_old_minters: false },
      { attachedDeposit: "1" }
    )
    .catch(
      failPromiseRejection(
        test,
        "transferring store ownership (minters cleared)"
      )
    );

  // check logs
  assertEventLogs(
    test,
    (transferStoreClearMintersCall as TransactionResult).logs,
    [
      {
        standard: "mb_store",
        version: "0.1.0",
        event: "change_setting",
        data: changeSettingsData({
          revoked_minter: alice.accountId,
        }),
      },
      {
        standard: "mb_store",
        version: "0.1.0",
        event: "change_setting",
        data: changeSettingsData({
          revoked_minter: bob.accountId,
        }),
      },
      {
        standard: "mb_store",
        version: "0.1.0",
        event: "change_setting",
        data: changeSettingsData({
          granted_minter: carol.accountId,
        }),
      },
      {
        standard: "mb_store",
        version: "0.1.0",
        event: "change_setting",
        data: changeSettingsData({
          new_owner: carol.accountId,
        }),
      },
    ],
    "transferring store ownership (minters cleared)"
  );

  test.is(await store.view("get_owner_id"), carol.accountId);

  // query minters
  await assertMinters(
    { test, store },
    [
      [alice, false],
      [bob, false],
      [carol, true],
    ],
    "transferring store ownership (minters cleared)"
  );

  await assertContractPanics(test, [
    // require ownership
    [
      async () => {
        await alice.call(
          store,
          "transfer_store_ownership",
          { new_owner: alice.accountId, keep_old_minters: false },
          { attachedDeposit: "1" }
        );
      },
      "This method can only be called by the store owner",
      "Non-owner tried to transfer store ownership",
    ],
    // require yoctoNEAR deposit
    [
      async () => {
        await carol.call(store, "transfer_store_ownership", {
          new_owner: alice.accountId,
          keep_old_minters: false,
        });
      },
      "Requires attached deposit of exactly 1 yoctoNEAR",
      "Tried to transfer store ownership without yoctoNEAR deposit",
    ],
  ]);

  // // ----------------------------- keep minters ------------------------------
  const transferStoreKeepMintersCall = await carol
    .callRaw(
      store,
      "transfer_store_ownership",
      { new_owner: alice.accountId, keep_old_minters: true },
      { attachedDeposit: "1" }
    )
    .catch(
      failPromiseRejection(test, "transferring store ownership (keep minters)")
    );

  // check logs
  assertEventLogs(
    test,
    (transferStoreKeepMintersCall as TransactionResult).logs,
    [
      {
        standard: "mb_store",
        version: "0.1.0",
        event: "change_setting",
        data: changeSettingsData({
          granted_minter: alice.accountId,
        }),
      },
      {
        standard: "mb_store",
        version: "0.1.0",
        event: "change_setting",
        data: changeSettingsData({
          new_owner: alice.accountId,
        }),
      },
    ],
    "transferring store ownership (keep minters)"
  );

  test.is(await store.view("get_owner_id"), alice.accountId);
  // query minters
  await assertMinters(
    { test, store },
    [
      [alice, true],
      [bob, false],
      [carol, true],
    ],
    "transferring store ownership (keep minters)"
  );
});
