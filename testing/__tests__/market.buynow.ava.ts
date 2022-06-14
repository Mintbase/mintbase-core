import { TransactionResult } from "near-workspaces-ava";
import {
  assertContractPanic,
  assertContractTokenOwner,
  assertEventLogs,
  assertMakeOfferEvent,
  assertBalanceChanges,
  failPromiseRejection,
  getBalance,
  MARKET_WORKSPACE,
  mNEAR,
  NEAR,
  Tgas,
  hours,
  prepareTokenListing,
  createPayout,
} from "./test-utils";
MARKET_WORKSPACE.test(
  "market::buynow",
  async (test, { root, factory, store, market, alice, bob }) => {
    await prepareTokenListing(test, { root, alice, store, market, factory });

    // TODO::testing::low: test this function
    await root.call(
      market,
      "set_min_offer_hours",
      { min_offer_hours: 0 },
      { attachedDeposit: "1" }
    );

    await alice
      .call(
        store,
        "nft_batch_approve",
        {
          token_ids: ["0", "1"],
          account_id: market.accountId,
          msg: JSON.stringify({ price: NEAR(1), autotransfer: true }),
        },
        { attachedDeposit: mNEAR(8.8), gas: Tgas(200) }
      )
      .catch(failPromiseRejection(test, "listing token"));

    const token0Key = `0:${store.accountId}`;
    const token1Key = `1:${store.accountId}`;

    // need to assert panics in series, so we don't get into race conditions
    //  regarding locked tokens
    // try to attach less than claimed
    await assertContractPanic(
      test,
      async () => {
        await bob.call(
          market,
          "make_offer",
          {
            token_key: [token0Key],
            price: [NEAR(1.1)],
            timeout: [{ Hours: 1 }],
          },
          { attachedDeposit: NEAR(1), gas: Tgas(200) }
        );
      },
      `Summed prices must match the attached deposit`,
      "Bob tried attaching less than claimed"
    );
    // try to set price below ask
    await assertContractPanic(
      test,
      //  ownership
      async () => {
        await bob.call(
          market,
          "make_offer",
          {
            token_key: [token0Key, token1Key],
            price: [NEAR(0.95), NEAR(1.05)],
            timeout: [{ Hours: 1 }, { Hours: 1 }],
          },
          { attachedDeposit: NEAR(2) }
        );
      },
      "Cannot set offer below ask",
      "Bob tried setting the price below the asking price"
    );
    // try to set instant expiry
    await assertContractPanic(
      test,
      async () => {
        await bob.call(
          market,
          "make_offer",
          {
            token_key: [token0Key],
            price: [NEAR(1)],
            timeout: [{ Hours: 0 }],
          },
          { attachedDeposit: NEAR(1), gas: Tgas(200) }
        );
      },
      "Cannot set times into the past",
      "Bob tried to set instant expiry"
    );
    // fuzzing: to few arguments
    await assertContractPanic(
      test,
      async () => {
        await bob.call(
          market,
          "make_offer",
          {
            token_key: [token0Key, token1Key],
            price: [NEAR(1)],
            timeout: [{ Hours: 1 }],
          },
          { attachedDeposit: NEAR(1), gas: Tgas(200) }
        );
      },
      "Price list doesn't match up with token list",
      "Bob tried fuzzing by omitting arguments"
    );
    // fuzzing: to many arguments
    await assertContractPanic(
      test,
      async () => {
        await bob.call(
          market,
          "make_offer",
          {
            token_key: [token0Key, token1Key],
            price: [NEAR(1), NEAR(1.5), NEAR(0.5)],
            timeout: [{ Hours: 1 }, { Hours: 1 }, { Hours: 1 }],
          },
          { attachedDeposit: NEAR(3), gas: Tgas(200) }
        );
      },
      "Price list doesn't match up with token list",
      "Bob tried fuzzing by adding arguments"
    );

    const aliceBalance0 = await getBalance(alice);
    const bobBalance0 = await getBalance(bob);
    const marketBalance0 = await getBalance(market);

    // TODO::market::low: improve this interface
    const makeOfferCall = await bob
      .call_raw(
        market,
        "make_offer",
        {
          token_key: [token0Key, token1Key],
          price: [NEAR(1), NEAR(1.5)],
          timeout: [{ Hours: 1 }, { Hours: 1 }],
        },
        { attachedDeposit: NEAR(2.5), gas: Tgas(200) }
      )
      .catch(failPromiseRejection(test, 'making "buy now" offer'));

    // check event logs
    // this needs to be `test.like` because of the contained timestamps
    assertMakeOfferEvent(
      { test, eventLog: (makeOfferCall as TransactionResult).logs[0] },
      {
        id: 1, // TODO::market::low: why do we start counting at 1?
        store: store,
        maker: bob,
        specs: [
          {
            token_id: "0",
            approval_id: 0,
            price: NEAR(1).toString(),
            timeout: hours(1),
          },
          {
            token_id: "1",
            approval_id: 1,
            price: NEAR(1.5).toString(),
            timeout: hours(1),
          },
        ],
      },
      'Making "buy now" offer'
    );
    assertEventLogs(
      test,
      (makeOfferCall as TransactionResult).logs.slice(1),
      [
        {
          standard: "nep171",
          version: "1.0.0",
          event: "nft_transfer",
          data: [
            {
              authorized_id: null, // FIXME::store::low
              memo: null,
              new_owner_id: bob.accountId,
              old_owner_id: alice.accountId,
              token_ids: ["0"],
            },
          ],
        },
        {
          standard: "mb_market",
          version: "0.1.0",
          event: "nft_sold",
          data: {
            list_id: `0:0:${store.accountId}`,
            offer_num: 1,
            token_key: `0:${store.accountId}`,
            payout: createPayout([[alice, NEAR(0.975).toString()]]),
          },
        },
        {
          standard: "nep171",
          version: "1.0.0",
          event: "nft_transfer",
          data: [
            {
              authorized_id: null, // FIXME::store::low
              memo: null,
              new_owner_id: bob.accountId,
              old_owner_id: alice.accountId,
              token_ids: ["1"],
            },
          ],
        },
        {
          standard: "mb_market",
          version: "0.1.0",
          event: "nft_sold",
          data: {
            list_id: `1:1:${store.accountId}`,
            offer_num: 1,
            token_key: `1:${store.accountId}`,
            payout: createPayout([[alice, mNEAR(1462.5).toString()]]),
          },
        },
      ],
      'making "buy now" offer'
    );

    await assertContractTokenOwner(
      { test, store },
      { token_id: "0", owner_id: bob.accountId },
      "After transfers"
    ).catch(failPromiseRejection(test, "checking token ownership"));
    await assertContractTokenOwner(
      { test, store },
      { token_id: "1", owner_id: bob.accountId },
      "After transfers"
    ).catch(failPromiseRejection(test, "checking token ownership"));

    // check market state (tokens unlisted)
    await test.throwsAsync(async () => {
      await market.view("get_token", { token_key: `0:${store.accountId}` });
    });
    await test.throwsAsync(async () => {
      await market.view("get_token", { token_key: `1:${store.accountId}` });
    });

    // chain state: account balances
    await assertBalanceChanges(
      test,
      [
        // 30 mNEAR extra gas costs for bob
        { account: bob, ref: bobBalance0, diff: NEAR(-2.53) },
        { account: alice, ref: aliceBalance0, diff: mNEAR(975 * 2.5) },
        // FIXME::market::low: where do the 15 mNEAR come from?
        { account: market, ref: marketBalance0, diff: mNEAR(25 * 2.5 + 15) },
      ],
      "After accepting 'buy now' offer"
    );

    // TODO::testing::low what happens in the case where one offer is valid and the other is not?

    // TODO::testing::medium: Users don't need to pay for replacing an offer
    // TODO::testing::medium: Users don't need to pay for replacing multiple offers
  }
);
