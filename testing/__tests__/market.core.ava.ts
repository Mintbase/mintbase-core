import { TransactionResult } from "near-workspaces";
import avaTest from "ava";
import {
  assertEventLogs,
  batchMint,
  failPromiseRejection,
  mNEAR,
  NEAR,
  Tgas,
} from "./utils/index.js";
import { setup } from "./setup.js";

const test = setup(avaTest);

test("market::core", async (test) => {
  const { root, factory, store, market, alice } = test.context.accounts;
  await batchMint({ owner: alice, store, num_to_mint: 2 }).catch(
    failPromiseRejection(test, "minting")
  );

  // ----------- allow the store to list tokens to the marketplace -----------
  const updateAllowlistCall = await market
    .callRaw(
      market,
      "update_allowlist",
      { account_id: factory.accountId, state: true },
      { attachedDeposit: "1" }
    )
    .catch(failPromiseRejection(test, "updating allowlist"));

  // check event logs
  assertEventLogs(
    test,
    (updateAllowlistCall as TransactionResult).logs,
    [
      {
        standard: "mb_market",
        version: "0.1.0",
        event: "update_allowlist",
        data: { account_id: factory.accountId, state: true },
      },
    ],
    "updating allowlist"
  );

  // TODO: try allowing as non-owner
  // TODO: try allowing without yoctoNEAR deposit

  // check on-chain state
  test.deepEqual(await market.view("get_allowlist", {}), [factory.accountId]);

  // ---------------------------- list as auction ----------------------------
  const auctionApproveCall = await alice
    .callRaw(
      store,
      "nft_approve",
      {
        token_id: "0",
        account_id: market.accountId,
        // TODO::market::medium: doesn't make sense to list a price for an
        //  auction
        msg: JSON.stringify({
          price: NEAR(1).toString(),
          autotransfer: false,
        }),
      },
      { attachedDeposit: mNEAR(0.81), gas: Tgas(200) }
    )
    // TODO::market::low: Complained about `alice.factory.test.near` not
    //  being allowed, which was on the allowlist. The requirement however is
    //  for `factory.test.near` to be on the allowlist.
    //  => better error message
    .catch(failPromiseRejection(test, "auction listing"));

  // check event logs
  assertEventLogs(
    test,
    // we already tested the approval event on the store, so skip that
    (auctionApproveCall as TransactionResult).logs.slice(1),
    [
      {
        standard: "mb_market",
        version: "0.1.0",
        event: "nft_list",
        data: [
          {
            // TODO::market::low: why this duplication?
            list_id: `0:0:${store.accountId}`,
            price: NEAR(1).toString(),
            // TODO::market::low: why this duplication?
            token_key: `0:${store.accountId}`,
            owner_id: alice.accountId,
            autotransfer: false,
            approval_id: "0",
            token_id: "0",
            store_id: store.accountId,
            // meta_id: null,
          },
        ],
      },
    ],
    "auction listing"
  );
  // TODO::testing::medium: what happens when I approve the same token twice?

  // checking market state
  // TODO::market::low: more descriptive method name
  test.deepEqual(
    await market.view("get_token", { token_key: `0:${store.accountId}` }),
    {
      id: 0, // TODO::market::low: rename to token_id, use string type
      owner_id: alice.accountId,
      store_id: store.accountId,
      autotransfer: false,
      asking_price: NEAR(1).toString(),
      approval_id: 0,
      current_offer: null,
      num_offers: 0,
      locked: false,
    }
  );

  // ------------------------ revoke auction approval ------------------------
  const auctionRevokeCall = await alice
    .callRaw(
      store,
      "nft_revoke",
      { token_id: "0", account_id: market.accountId },
      { attachedDeposit: "1" }
    )
    .catch(failPromiseRejection(test, "revoke auction listing"));

  // check event logs
  assertEventLogs(
    test,
    // we already tested the revoke event on the store, so skip that
    (auctionRevokeCall as TransactionResult).logs.slice(1),
    [],
    "revoke auction listing"
  );

  // TODO: check market state -> do we have the functionality for that?
  // TODO: find out if the indexer picks up on this revoke
  //   if so, we have divergent state between indexer and marketplace
  // marketplace::check_approvals method is unfeasible because we have about
  // half a million tokens on offer on a normal day

  // --------------------------- list as "buy now" ---------------------------
  const buynowApproveCall = await alice
    .callRaw(
      store,
      "nft_approve",
      {
        token_id: "0",
        account_id: market.accountId,
        msg: JSON.stringify({
          price: NEAR(1).toString(),
          autotransfer: true,
        }),
      },
      { attachedDeposit: mNEAR(0.81).toString(), gas: Tgas(200) }
    )
    .catch(failPromiseRejection(test, "buy now listing"));

  // check event logs
  assertEventLogs(
    test,
    // we already tested the approval event, so skip that
    (buynowApproveCall as TransactionResult).logs.slice(1),
    [
      {
        standard: "mb_market",
        version: "0.1.0",
        event: "nft_unlist",
        data: [{ list_id: `0:0:${store.accountId}` }],
      },
      {
        standard: "mb_market",
        version: "0.1.0",
        event: "nft_list",
        data: [
          {
            // TODO::market::low: why this duplication?
            list_id: `0:1:${store.accountId}`,
            price: NEAR(1).toString(),
            // TODO::market::low: why this duplication?
            token_key: `0:${store.accountId}`,
            owner_id: alice.accountId,
            autotransfer: true,
            approval_id: "1",
            token_id: "0",
            store_id: store.accountId,
            // thing_id: null,
          },
        ],
      },
    ],
    "buy now listing"
  );

  // check market state
  test.deepEqual(
    await market.view("get_token", { token_key: `0:${store.accountId}` }),
    {
      id: 0, // FIXME::market::low: rename to token_id, use string type
      owner_id: alice.accountId,
      store_id: store.accountId,
      autotransfer: true,
      asking_price: NEAR(1).toString(),
      approval_id: 1,
      current_offer: null,
      num_offers: 0,
      locked: false,
    }
  );

  // ----------------------- revoke "buy now" approval -----------------------
  const buynowRevokeCall = await alice
    .callRaw(
      store,
      "nft_revoke",
      { token_id: "0", account_id: market.accountId },
      { attachedDeposit: "1" }
    )
    .catch(failPromiseRejection(test, "revoke auction listing"));

  // check event logs
  assertEventLogs(
    test,
    // we already tested the revoke event on the store, so skip that
    (buynowRevokeCall as TransactionResult).logs.slice(1),
    [],
    "revoke auction listing"
  );

  // ----------------------------- batch listing -----------------------------
  const batchApproveLogs = await alice.callRaw(
    store,
    "nft_batch_approve",
    {
      token_ids: ["0", "1"],
      account_id: market.accountId,
      msg: JSON.stringify({ price: NEAR(1).toString(), autotransfer: true }),
    },
    // TODO::store::medium: why does thir require more storage deposit than
    //  batch approving without tail call?
    //  -> we might and probably should require a deposit on the market for
    //     each token on offer
    { attachedDeposit: mNEAR(8.8).toString(), gas: Tgas(200) }
  );

  // check event logs
  assertEventLogs(
    test,
    (batchApproveLogs as TransactionResult).logs.slice(1),
    [
      {
        standard: "mb_market",
        version: "0.1.0",
        event: "nft_unlist",
        data: [{ list_id: `0:1:${store.accountId}` }],
      },
      {
        standard: "mb_market",
        version: "0.1.0",
        event: "nft_list",
        data: [
          {
            list_id: `0:2:${store.accountId}`,
            price: NEAR(1).toString(),
            token_key: `0:${store.accountId}`,
            owner_id: alice.accountId,
            autotransfer: true,
            approval_id: "2",
            token_id: "0",
            store_id: store.accountId,
          },
          {
            list_id: `1:3:${store.accountId}`,
            price: NEAR(1).toString(),
            token_key: `1:${store.accountId}`,
            owner_id: alice.accountId,
            autotransfer: true,
            approval_id: "3",
            token_id: "1",
            store_id: store.accountId,
          },
        ],
      },
    ],
    "batch approving"
  );

  // check market state
  test.like(
    await market.view("get_token", { token_key: `0:${store.accountId}` }),
    { autotransfer: true, asking_price: NEAR(1).toString() }
  );
  test.like(
    await market.view("get_token", { token_key: `1:${store.accountId}` }),
    { autotransfer: true, asking_price: NEAR(1).toString() }
  );

  // ---------------------------- batch revoking -----------------------------
  // doesn't make any sense at the moment
  // TODO::testing::medium: batch revoking of tokens
  // TODO: check event logs
  // TODO: check market state
});

// TODO: market::allowlist/banlist

// // --------------------- FIXME: users should be refunded! ----------------------
// With this bug, a user may call `make_offer` with a deposit matching the
// claimed price, but the claimed price being below the ask of the owner.
// The contract should panic and refund the attached deposit, while it does
// neither.
// MARKET_WORKSPACE.test(
//   "market::transfer-bug",
//   async (test, { root, factory, store, market, alice, bob }) => {
//     await batchMint({ owner: alice, store, num_to_mint: 1 }).catch(
//       failPromiseRejection(test, "minting")
//     );

//     await root
//       .call(
//         market,
//         "update_allowlist",
//         { account_id: factory.accountId, state: true },
//         { attachedDeposit: "1" }
//       )
//       .catch(failPromiseRejection(test, "allowing store on market"));

//     await alice
//       .call(
//         store,
//         "nft_approve",
//         {
//           token_id: "0",
//           account_id: market.accountId,
//           msg: JSON.stringify({ price: NEAR(100), autotransfer: true }),
//         },
//         { attachedDeposit: mNEAR(20), gas: Tgas(200) }
//       )
//       .catch(failPromiseRejection(test, "listing token"));

//     const token_key = `0:${store.accountId}`;
//     const log_owner = async (msg?: string) => {
//       const token = await store.view("nft_token", { token_id: "0" });
//       test.log(msg, (token as MintbaseToken).owner_id.Account);
//     };

//     await log_owner("After approving");
//     test.log(await market.view("get_token", { token_key }));
//     test.log("market balance: ", (await market.balance()).total.toHuman());
//     test.log("bob balance: ", (await bob.balance()).total.toHuman());

//     await bob.call(
//       market,
//       "make_offer",
//       {
//         token_key: [token_key],
//         price: [NEAR(10)],
//         timeout: [{ Hours: 48 }],
//       },
//       { attachedDeposit: NEAR(10), gas: Tgas(200) }
//     );

//     await log_owner("After bob making offer");
//     test.log(await market.view("get_token", { token_key }));
//     test.log("market balance: ", (await market.balance()).total.toHuman());
//     test.log("bob balance: ", (await bob.balance()).total.toHuman());
//   }
// );
