import { TransactionResult } from "near-workspaces-ava";
import {
  assertContractPanics,
  assertContractTokenOwner,
  assertEventLogs,
  assertMakeOfferEvent,
  failPromiseRejection,
  MARKET_WORKSPACE,
  mNEAR,
  NEAR,
  Tgas,
  hours,
  getBalance,
  assertBalanceChange,
  createPayout,
  prepareTokenListing,
} from "./test-utils";

MARKET_WORKSPACE.test(
  "market::auction",
  async (test, { root, factory, store, market, alice, bob, carol }) => {
    await prepareTokenListing(test, { root, alice, store, market, factory });

    await alice
      .call(
        store,
        "nft_approve",
        {
          token_id: "0",
          account_id: market.accountId,
          msg: JSON.stringify({ price: NEAR(1), autotransfer: false }),
        },
        { attachedDeposit: mNEAR(0.81), gas: Tgas(200) }
      )
      .catch(failPromiseRejection(test, "listing token"));
    const tokenKey = `0:${store.accountId}`;

    // -------------------------- create first offer ---------------------------
    const bobBalance0 = await getBalance(bob);
    const marketBalance0 = await getBalance(market);
    const makeOfferCall0 = await bob
      .call_raw(
        market,
        "make_offer",
        {
          token_key: [tokenKey],
          price: [NEAR(1)],
          timeout: [{ Hours: 24 }],
        },
        { attachedDeposit: NEAR(1), gas: Tgas(200) }
      )
      .catch(failPromiseRejection(test, "making first auction offer"));
    // check event logs
    assertMakeOfferEvent(
      { test, eventLog: (makeOfferCall0 as TransactionResult).logs[0] },
      {
        id: 1,
        store: store,
        maker: bob,
        specs: [
          {
            token_id: "0",
            approval_id: 0,
            price: NEAR(1).toString(),
            timeout: hours(24),
          },
        ],
      },
      "Making first auction offer"
    );
    test.is(
      (makeOfferCall0 as TransactionResult).logs.length,
      1,
      "Emitted too many events on making first auction offer"
    );

    // check chain state: token owner hasn't changed
    await assertContractTokenOwner(
      { test, store },
      { token_id: "0", owner_id: alice.accountId },
      "Token auto-transferred on making auction offer"
    );
    // check chain state: highest offer is 1N
    test.like(
      // FIXME::market::low: price should be a string
      await market.view("get_current_offer", { token_key: tokenKey }),
      { id: 1, price: parseInt(NEAR(1).toString()) },
      "Highest offer not set correctly"
    );
    // check chain state: bob has 1N less
    await Promise.all([
      assertBalanceChange(
        test,
        { account: bob, ref: bobBalance0, diff: NEAR(-1) },
        "Making first auction offer"
      ),
      // check chain state: market has 1N more
      assertBalanceChange(
        test, // FIXME::market::medium: where do the 10 mNEAR come from?
        { account: market, ref: marketBalance0, diff: NEAR(1.01) },
        "Making first auction offer"
      ),
    ]);
    // test.fail();
    // ---------------------- withdraw offer and recreate ----------------------
    // TODO::testing::medium: withdraw offer -> not feasible until `min_offer_hours`
    //  can be set to e.g. minutes or seconds
    //  when this is implemented, we need at least three offers:
    //  offer -> withdraw -> offer again -> offer (replaces current highest offer) -> accept
    // TODO: check event logs
    // TODO: check chain state -> token owner hasn't changed, no highest offer
    // TODO: try accepting withdrawn offer -> should fail

    // -------------------------- create second offer --------------------------
    const carolBalance1 = await getBalance(carol);
    const marketBalance1 = await getBalance(market);
    const bobBalance1 = await getBalance(bob);

    const makeOfferCall1 = await carol
      .call_raw(
        market,
        "make_offer",
        {
          token_key: [tokenKey],
          price: [NEAR(2)],
          timeout: [{ Hours: 24 }],
        },
        { attachedDeposit: NEAR(2), gas: Tgas(200) }
      )
      .catch(failPromiseRejection(test, "making second auction offer"));

    // check event logs
    assertMakeOfferEvent(
      { test, eventLog: (makeOfferCall1 as TransactionResult).logs[0] },
      {
        id: 2,
        store: store,
        maker: carol,
        specs: [
          {
            token_id: "0",
            approval_id: 0,
            price: NEAR(2).toString(),
            timeout: hours(24),
          },
        ],
      },
      "Making second auction offer"
    );
    test.is(
      (makeOfferCall0 as TransactionResult).logs.length,
      1,
      "Emitted too many events on making second auction offer"
    );

    // check chain state: token owner still hasn't changed
    await assertContractTokenOwner(
      { test, store },
      { token_id: "0", owner_id: alice.accountId },
      "Token auto-transferred on making auction offer"
    );
    // check chain state: highest offer is 2N
    test.like(
      await market.view("get_current_offer", { token_key: tokenKey }),
      { id: 2, price: parseInt(NEAR(2).toString()) },
      "Highest offer not replaced"
    );
    await Promise.all([
      // check chain state: carol has 2N less now
      assertBalanceChange(
        test,
        { account: carol, ref: carolBalance1, diff: NEAR(-2) },
        "outbidding on auction"
      ),
      // check chain state: market has 1N more now
      assertBalanceChange(
        test, // FIXME::market::medium: where do the 10 mNEAR come from?
        { account: market, ref: marketBalance1, diff: NEAR(1.01) },
        "outbidding on auction"
      ),
      // check chain state: bob got his 1N back
      assertBalanceChange(
        test,
        { account: bob, ref: bobBalance1, diff: NEAR(1) },
        "outbidding on auction"
      ),
    ]);
    // ----------------------------- accept offer ------------------------------
    const aliceBalance2 = await getBalance(alice);
    const marketBalance2 = await getBalance(market);
    await assertContractPanics(test, [
      // try accepting offer as non-owner
      [
        async () => {
          await bob.call(
            market,
            "accept_and_transfer",
            { token_key: tokenKey },
            { attachedDeposit: "1", gas: Tgas(200) }
          );
        },
        "panicked at 'assertion failed:",
        "Bob tried to accept an offer for Alice's token",
      ],
      // try accepting offer without yoctoNEAR deposit
      [
        async () => {
          await alice.call(
            market,
            "accept_and_transfer",
            { token_key: tokenKey },
            { gas: Tgas(200) }
          );
        },
        "Requires attached deposit of exactly 1 yoctoNEAR",
        "Alice tried to accept an offer without yoctoNEAR deposit",
      ],
    ]);

    const acceptOfferCall = await alice
      .call_raw(
        market,
        "accept_and_transfer",
        { token_key: tokenKey },
        { attachedDeposit: "1", gas: Tgas(200) }
      )
      .catch(failPromiseRejection(test, "accepting auction offer"));

    // check event logs
    assertEventLogs(
      test,
      (acceptOfferCall as TransactionResult).logs,
      [
        {
          standard: "nep171",
          version: "1.0.0",
          event: "nft_transfer",
          data: [
            {
              authorized_id: null, // FIXME::store::low,
              old_owner_id: alice.accountId,
              new_owner_id: carol.accountId,
              token_ids: ["0"],
              memo: null,
            },
          ],
        },
        {
          standard: "nep171",
          version: "1.0.0",
          event: "nft_sold",
          data: JSON.stringify({
            list_id: `0:0:${store.accountId}`,
            offer_num: 2,
            token_key: `0:${store.accountId}`,
            payout: createPayout([[alice, NEAR(1.95).toString()]]),
          }),
        },
      ],
      "accepting auction offer"
    );

    // check chain state: token is owned by carol now
    assertContractTokenOwner(
      { test, store },
      { token_id: "0", owner_id: carol.accountId },
      "accepting auction offer"
    );

    await Promise.all([
      // check chain state: alice has received her share
      assertBalanceChange(
        test,
        { account: alice, ref: aliceBalance2, diff: NEAR(1.95) },
        "accepting auction offer"
      ),
      // check chain state: market has transferred some funds but kept its fee
      assertBalanceChange(
        test,
        // FIXME::market::medium: why does the market retain more than it should?
        { account: market, ref: marketBalance2, diff: NEAR(-1.94) },
        "accepting auction offer"
      ),
    ]);
  }
);
