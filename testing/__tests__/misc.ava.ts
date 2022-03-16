import { TransactionResult, Workspace } from "near-workspaces-ava";
import avaTest from "ava";
import {
  NEAR,
  mNEAR,
  uNEAR,
  nNEAR,
  STORE_WORKSPACE,
  assertEventLogs,
  failPromiseRejection,
  assertMinters,
  assertContractPanics,
} from "./test-utils";

// No need to fire up the chain for testing my utils
avaTest("util tests", (test) => {
  test.is(NEAR(1.5).toString(), "1500000000000000000000000");
  test.is(mNEAR(1.5).toString(), "1500000000000000000000");
  test.is(uNEAR(1.5).toString(), "1500000000000000000");
  test.is(nNEAR(1.5).toString(), "1500000000000000");

  // TODO::testing::low: assertTokenIs?
  // TODO::testing::low: assertTokensAre?
  // TODO::testing::low: assertEventLog?
  // TODO::testing::low: assertEventLogs?
});

// As this tests deployment, we do it in a clean-state environment
Workspace.init().test("deployment", async (test, { root }) => {
  // TODO::testing::low: edge cases of deployment
  const failDeploymentError = (name: string) => (e: any) => {
    test.log(`Failed to deploy ${name} contract`);
    test.log(e);
    test.fail();
  };

  await root
    .createAndDeploy(
      "factory", // subaccount name
      "../wasm/factory.wasm", // path to wasm
      { method: "new", args: {} }
    )
    .catch(failDeploymentError("factory"));

  await root
    .createAndDeploy("store", "../wasm/store.wasm", {
      method: "new",
      args: {
        owner_id: root.accountId,
        metadata: {
          spec: "nft-1.0.0",
          name: "store.root",
          symbol: "ROOT",
        },
      },
    })
    .catch(failDeploymentError("store"));

  await root
    .createAndDeploy("helper", "../wasm/helper.wasm", {
      method: "new",
      args: {},
    })
    .catch(failDeploymentError("helper"));

  await root
    .createAndDeploy("market", "../wasm/market.wasm", {
      method: "new",
      args: { init_allowlist: [] },
    })
    .catch(failDeploymentError("market"));
});

STORE_WORKSPACE.test(
  "ownership::transfer-store",
  async (test, { alice, bob, carol, store }) => {
    await alice
      .call(
        store,
        "grant_minter",
        { account_id: bob },
        { attachedDeposit: "1" }
      )
      .catch(failPromiseRejection(test, "granting minter rights"));

    // ---------------------------- remove minters -----------------------------
    const transferStoreClearMintersCall = await alice
      .call_raw(
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
          standard: "nep171",
          version: "1.0.0",
          event: "nft_revoke_minter",
          // TODO::store::medium: wtf is this format?
          data: JSON.stringify({ data: alice.accountId }),
        },
        {
          standard: "nep171",
          version: "1.0.0",
          event: "nft_revoke_minter",
          data: JSON.stringify({ data: bob.accountId }),
        },
        {
          standard: "nep171",
          version: "1.0.0",
          event: "nft_grant_minter",
          data: JSON.stringify({ data: carol.accountId }),
        },
        {
          standard: "nep171",
          version: "1.0.0",
          event: "nft_transfer_store",
          data: JSON.stringify({ data: carol.accountId }),
        },
      ],
      "transferring store ownership (minters cleared)"
    );

    // TODO::store::medium query owner

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
        "panicked at 'assertion failed: `(left == right)",
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

    // ----------------------------- keep minters ------------------------------
    const transferStoreKeepMintersCall = await carol
      .call_raw(
        store,
        "transfer_store_ownership",
        { new_owner: alice.accountId, keep_old_minters: true },
        { attachedDeposit: "1" }
      )
      .catch(
        failPromiseRejection(
          test,
          "transferring store ownership (keep minters)"
        )
      );

    // check logs
    assertEventLogs(
      test,
      (transferStoreKeepMintersCall as TransactionResult).logs,
      [
        {
          standard: "nep171",
          version: "1.0.0",
          event: "nft_grant_minter",
          // TODO::store::medium: wtf is this format?
          data: JSON.stringify({ data: alice.accountId }),
        },
        {
          standard: "nep171",
          version: "1.0.0",
          event: "nft_transfer_store",
          data: JSON.stringify({ data: alice.accountId }),
        },
      ],
      "transferring store ownership (keep minters)"
    );

    // TODO::store::medium query owner
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
  }
);

// TODO::testing::low factory ownership transfer
// DEFAULT_WORKSPACE.test(
//   "ownership::transfer-factory",
//   async (test, { root, alice, bob, carol, store }) => {
//     await alice.call(store, "grant_minter", {});

//     const transferStoreCall = await alice.call_raw(
//       store,
//       "store_ownership",
//       {}
//     );

//     // TODO: check logs
//     // TODO: require ownership
//     // TODO: require yoctoNEAR deposit
//   }
// );

// TODO::testing::low market ownership transfer
// DEFAULT_WORKSPACE.test(
//   "ownership::transfer-market",
//   async (test, { root, alice, bob, carol, store }) => {
//     await alice.call(store, "grant_minter", {});

//     const transferStoreCall = await alice.call_raw(
//       store,
//       "store_ownership",
//       {}
//     );

//     // TODO: check logs
//     // TODO: require ownership
//     // TODO: require yoctoNEAR deposit
//   }
// );

// TODO:

// Things to address in tests
// - [x] Edge cases should be handled in a `Promise.all`
// - [x] break into modules and subfunctions
// - [x] Check logs method
// - [] subfunctions with the same signature as AvaTestingFn?
// - [] better test names + more granularity to allow descriptive selections?
// - [] wrapping the `test` object to do common things (high effort, not urgent)
//   - [] make the underlying methods available
//   - [] test.isNftOwner(token_id, owner_id)
//   - [] test.notIsNftOwner(contract, token_id, owner_id)
//   - [] test.isNftApproved(contract, token_id, owner_id, approval_id?)
//   - [] test.notIsNftApproved(contract, token_id, owner_id, approval_id?)
//   - [] test.isMinter(account_id)
//   - [] test.notIsMinter(account_id)
// - []
// - []
//

// Things to address in codebase
//
// high priority
// - [] Token doesn't comply with NEP171
// - [] Events don't comply with NEP297 -> Cannot use impls from
//      `near_contract_standards`, as we need a version bump for fixing that
// - [] `nft_tokens` and `nft_tokens_for_owner` are non-compliant
//   - [] ask frontend if changing them will break things
// - []
//
// low priority
// - [] decide whether empty fields should show up in the event logs
// - [] revise the signature of `batch_transfer`
// - []
//
