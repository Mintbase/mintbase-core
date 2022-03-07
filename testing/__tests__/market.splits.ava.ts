import { TransactionResult } from "near-workspaces-ava";
import {
  batchMint,
  failPromiseRejection,
  MARKET_WORKSPACE,
  mNEAR,
  NEAR,
  Tgas,
  getBalance,
  assertBalanceChange,
  createPayoutPercentage,
  createPayoutNumerators,
  assertEventLogs,
  assertContractPanics,
} from "./test-utils";

MARKET_WORKSPACE.test(
  "market::splits",
  async (test, { root, factory, store, market, alice, bob, carol }) => {
    // cannot use `prepareTokenListing`, because royalties need to be set
    // during minting
    await root
      .call(
        market,
        "update_allowlist",
        { account_id: factory.accountId, state: true },
        { attachedDeposit: "1" }
      )
      .catch(failPromiseRejection(test, "allowing store on market"));

    const dave = await root.createAccount("dave", { initialBalance: NEAR(20) });

    await batchMint({ owner: alice, store, num_to_mint: 1 }).catch(
      failPromiseRejection(test, "minting")
    );

    // ---------------------------- setting splits -----------------------------
    await assertContractPanics(test, [
      // only token owner can set
      [
        async () => {
          await bob.call(
            store,
            "set_split_owners",
            {
              token_ids: ["0"],
              split_between: createPayoutPercentage([
                [alice, 6000],
                [bob, 4000],
              ]),
            },
            { attachedDeposit: mNEAR(1.6) }
          );
        },
        "panicked at 'assertion failed: token.is_pred_owner()',",
        "Bob tried setting splits on Alice's token",
      ],
      [
        // requires storage deposit
        async () => {
          await alice.call(
            store,
            "set_split_owners",
            {
              token_ids: ["0"],
              split_between: createPayoutPercentage([
                [alice, 6000],
                [bob, 4000],
              ]),
            },
            { attachedDeposit: mNEAR(1.59) }
          );
        },
        // TODO::store::low: better error messages
        `panicked at 'insuf. deposit. Need: ${mNEAR(1.6)}',`,
        "Alice tried setting splits with insufficient storage deposit",
      ],
    ]);
    const setSplitsCall = await alice
      .call_raw(
        store,
        "set_split_owners",
        {
          token_ids: ["0"],
          split_between: createPayoutPercentage([
            [alice, 6000],
            [bob, 4000],
          ]),
        },
        { attachedDeposit: mNEAR(1.6) }
      )
      .catch(failPromiseRejection(test, "setting splits"));

    const storeFormattedSplits = {
      // TODO::store::low: why the nesting?
      split_between: createPayoutNumerators([
        [alice, 6000],
        [bob, 4000],
      ]),
    };

    // check event logs
    assertEventLogs(
      test,
      (setSplitsCall as TransactionResult).logs,
      [
        {
          standard: "nep171",
          version: "1.0.0",
          event: "nft_set_split_owners",
          // TODO::store::low: unstringify
          data: JSON.stringify({
            split_owners: storeFormattedSplits,
            token_ids: ["0"],
          }),
        },
      ],
      "setting splits"
    );

    // check chain state: splits in `nft_token`
    test.deepEqual(
      ((await store.view("nft_token", { token_id: "0" })) as any).split_owners,
      storeFormattedSplits,
      "Bad onchain splits (querying `nft_token`)"
    );
    // so far, I cannot find a direct method to query the split owners

    // ------------------- executing transfer with royalties -------------------
    await alice
      .call(
        store,
        "nft_approve",
        {
          token_id: "0",
          account_id: market.accountId,
          msg: JSON.stringify({ price: NEAR(1), autotransfer: true }),
        },
        { attachedDeposit: mNEAR(0.81), gas: Tgas(200) }
      )
      .catch(failPromiseRejection(test, "listing token"));
    // events have been checked previously -> no need here
    const tokenKey = `0:${store.accountId}`;

    const aliceBalance0 = await getBalance(alice);
    const bobBalance0 = await getBalance(bob);
    await carol
      .call(
        market,
        "make_offer",
        {
          token_key: [tokenKey],
          price: [NEAR(1)],
          timeout: [{ Hours: 24 }],
        },
        { attachedDeposit: NEAR(1), gas: Tgas(200) }
      )
      .catch(failPromiseRejection(test, "making offer"));
    // events have been checked previously -> no need here

    // check chain state: alice received 0.75 * 0.975 NEAR
    await assertBalanceChange(
      test,
      { account: alice, ref: aliceBalance0, diff: mNEAR(0.6 * 975) },
      "Checking royalties (without splits)"
    );
    // check chain state: bob received 0.25 * 0.975 NEAR
    await assertBalanceChange(
      test,
      { account: bob, ref: bobBalance0, diff: mNEAR(0.4 * 975) },
      "Checking royalties (without splits)"
    );

    // --------------------- redo, splits should be reset ----------------------
    test.deepEqual(
      ((await store.view("nft_token", { token_id: "0" })) as any).split_owners,
      null,
      "Bad onchain splits after transfer (querying `nft_token`)"
    );

    await carol
      .call(
        store,
        "nft_approve",
        {
          token_id: "0",
          account_id: market.accountId,
          msg: JSON.stringify({ price: NEAR(1), autotransfer: true }),
        },
        { attachedDeposit: mNEAR(0.81), gas: Tgas(200) }
      )
      .catch(failPromiseRejection(test, "listing token again"));

    const aliceBalance1 = await getBalance(alice);
    const bobBalance1 = await getBalance(bob);
    const carolBalance1 = await getBalance(carol);

    await dave
      .call(
        market,
        "make_offer",
        {
          token_key: [tokenKey],
          price: [NEAR(1)],
          timeout: [{ Hours: 24 }],
        },
        { attachedDeposit: NEAR(1), gas: Tgas(200) }
      )
      .catch(failPromiseRejection(test, "making offer again"));

    // check chain state: alice received nothing
    await assertBalanceChange(
      test,
      { account: alice, ref: aliceBalance1, diff: "0" },
      "Checking royalties (without splits)"
    );
    // check chain state: bob received nothing
    await assertBalanceChange(
      test,
      { account: bob, ref: bobBalance1, diff: "0" },
      "Checking royalties (without splits)"
    );
    // check chain state: carol received 0.975 NEAR
    await assertBalanceChange(
      test,
      { account: carol, ref: carolBalance1, diff: mNEAR(975) },
      "Checking royalties (without splits)"
    );
  }
);
