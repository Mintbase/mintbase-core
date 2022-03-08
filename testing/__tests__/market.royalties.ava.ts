import { TransactionResult } from "near-workspaces-ava";
import {
  assertEventLogs,
  failPromiseRejection,
  MARKET_WORKSPACE,
  createPayout,
  createPayoutNumerators,
  createPayoutPercentage,
  NEAR,
  mNEAR,
  Tgas,
  getBalance,
  assertBalanceChange,
} from "./test-utils";

MARKET_WORKSPACE.test(
  "market::royalties",
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

    // --------------------------- setting royalties ---------------------------
    const mintCall = await alice
      .call_raw(
        store,
        "nft_batch_mint",
        {
          owner_id: alice.accountId,
          num_to_mint: 1,
          metadata: {},
          royalty_args: {
            split_between: createPayoutPercentage([
              [alice, 5000],
              [bob, 5000],
            ]),
            percentage: 5000, // this is 50 %
          },
        },
        { attachedDeposit: "1" }
      )
      .catch(failPromiseRejection(test, "minting with royalties"));

    // check event logs
    // TODO::store::low: format seems clunky
    const storeFormattedRoyalties = {
      split_between: createPayoutNumerators([
        [alice, 5000],
        [bob, 5000],
      ]),
      percentage: { numerator: 5000 },
    };

    assertEventLogs(
      test,
      (mintCall as TransactionResult).logs,
      [
        {
          standard: "nep171",
          version: "1.0.0",
          event: "nft_mint",
          data: [
            {
              owner_id: "alice.test.near",
              token_ids: ["0"],
              // memo should be a string, as it's standardized like that!
              memo: JSON.stringify({
                royalty: storeFormattedRoyalties,
                split_owners: null,
                meta_id: null,
                meta_extra: null,
                minter: alice.accountId,
              }),
            },
          ],
        },
      ],
      "minting"
    );

    // check chain state: royalties in token info
    test.deepEqual(
      ((await store.view("nft_token", { token_id: "0" })) as any).royalty,
      storeFormattedRoyalties,
      "Bad onchain royalties (querying `nft_token`)"
    );
    test.deepEqual(
      await store.view("get_token_royalty", { token_id: "0" }),
      storeFormattedRoyalties,
      "Bad onchain royalties (querying `nft_token_royalty`)"
    );
    test.log("royalties as known by store:", storeFormattedRoyalties);
    // // check chain state: royalties in payout info
    // // FIXME::store::medium: these shouldn't be zero
    // test.deepEqual(
    //   (
    //     (await store.view("nft_payout", {
    //       token_id: "0",
    //       balance: "1000",
    //       max_len_payout: 5,
    //     })) as any
    //   ).payout,
    //   createPayout([
    //     [alice, "750"],
    //     [bob, "250"],
    //   ]),
    //   "Bad onchain royalties (querying `nft_payout`)"
    // );

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
      { account: alice, ref: aliceBalance0, diff: mNEAR(0.75 * 975) },
      "Checking first royalties payout"
    );
    // check chain state: bob received 0.25 * 0.975 NEAR
    await assertBalanceChange(
      test,
      { account: bob, ref: bobBalance0, diff: mNEAR(0.25 * 975) },
      "Checking first royalties payout"
    );
    // -------------- executing again -> royalties are perpetual ---------------
    // // check chain state: royalties in payout info
    // // FIXME::store::medium: these shouldn't be zero
    // test.deepEqual(
    //   (
    //     (await store.view("nft_payout", {
    //       token_id: "0",
    //       balance: "1000",
    //       max_len_payout: 5,
    //     })) as any
    //   ).payout,
    //   createPayout([
    //     [alice, "250"],
    //     [bob, "250"],
    //     [carol, "500"],
    //   ]),
    //   "Bad onchain royalties (querying `nft_payout`)"
    // );
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
      .catch(failPromiseRejection(test, "listing token"));
    // events have been checked previously -> no need here

    const aliceBalance1 = await getBalance(alice);
    const bobBalance1 = await getBalance(bob);
    const carolBalance1 = await getBalance(carol);

    const dave = await root.createAccount("dave", {
      initialBalance: NEAR(20).toString(),
    });
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
      .catch(failPromiseRejection(test, "making offer"));
    // events have been checked previously -> no need here

    // check chain state: alice received 0.75 * 0.975 NEAR
    await assertBalanceChange(
      test,
      { account: alice, ref: aliceBalance1, diff: mNEAR(0.25 * 975) },
      "Checking second royalties payout"
    );
    // check chain state: bob received 0.25 * 0.975 NEAR
    await assertBalanceChange(
      test,
      { account: bob, ref: bobBalance1, diff: mNEAR(0.25 * 975) },
      "Checking second royalties payout"
    );
    // check chain state: bob received 0.50 * 0.975 NEAR
    await assertBalanceChange(
      test,
      { account: carol, ref: carolBalance1, diff: mNEAR(0.5 * 975) },
      "Checking second royalties payout"
    );
  }
);
