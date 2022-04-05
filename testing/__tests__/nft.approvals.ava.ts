import { BN, TransactionResult } from "near-workspaces-ava";
import {
  assertApprovals,
  assertNoApprovals,
  assertContractPanics,
  assertEventLogs,
  assertBalanceChange,
  batchMint,
  STORE_WORKSPACE,
  mNEAR,
  getBalance,
  assertContractTokenOwners,
  assertNoApproval,
} from "./test-utils";
// import * as ava from "near-workspaces-ava";

STORE_WORKSPACE.test(
  "approvals::core",
  async (test, { alice, bob, carol, store }) => {
    const failPromiseRejection = (msg: string) => (e: any) => {
      test.log(`Promise rejected while ${msg}:`);
      test.log(e);
      test.fail();
    };

    await batchMint({ owner: alice, store, num_to_mint: 4 }).catch(
      failPromiseRejection("minting")
    );
    // // assert correctness of current owners
    // await assertContractTokenOwners(
    //   test,
    //   store,
    //   [
    //     { id: "0", owner_id: alice.accountId },
    //     { id: "1", owner_id: alice.accountId },
    //     { id: "2", owner_id: alice.accountId },
    //     { id: "3", owner_id: alice.accountId },
    //   ],
    //   "minting"
    // );

    // assert correctness of current approvals
    await assertNoApprovals(
      { test, store },
      [
        { token_id: "0", approved_account_id: bob.accountId },
        { token_id: "1", approved_account_id: bob.accountId },
        { token_id: "2", approved_account_id: bob.accountId },
        { token_id: "3", approved_account_id: bob.accountId },
      ],
      "minting"
    );

    // -------------------------------- approve --------------------------------
    const approveCall = await alice
      .call_raw(
        store,
        "nft_approve",
        { token_id: "0", account_id: bob.accountId },
        { attachedDeposit: mNEAR(0.81) } // no value for this in mintbase-js
      )
      .catch(failPromiseRejection("approving"));
    // check event logs
    assertEventLogs(
      test,
      (approveCall as TransactionResult).logs,
      [
        {
          event: "nft_approve",
          standard: "nep171",
          version: "1.0.0",
          // TODO::store::low: unstringify
          data: JSON.stringify([
            { token_id: 0, approval_id: 0, account_id: bob.accountId },
          ]),
        },
      ],
      "approving"
    );

    await assertContractPanics(test, [
      // try approving when not owning token
      [
        async () =>
          bob.call(
            store,
            "nft_approve",
            { token_id: "1", account_id: bob.accountId },
            { attachedDeposit: mNEAR(0.81) }
          ),
        "panicked at 'assertion failed: token.is_pred_owner()',",
        "Bob tried approving on unowned token",
      ],
      // require at least one yoctoNEAR to approve
      [
        async () =>
          alice.call(
            store,
            "nft_approve",
            { token_id: "1", account_id: bob.accountId },
            { attachedDeposit: mNEAR(0.8) } // deposit > 0.8
          ),
        //TODO::store::low: panic message format
        "panicked at 'assertion failed: env::attached_deposit() > self.storage_costs.common',",
        "Alice tried approving with insufficient deposit",
      ],
    ]);

    // assert correctness of current approvals
    await assertApprovals(
      { test, store },
      [{ token_id: "0", approved_account_id: bob.accountId, approval_id: 0 }],
      "approving"
    );
    await assertNoApprovals(
      { test, store },
      [
        { token_id: "1", approved_account_id: bob.accountId },
        { token_id: "2", approved_account_id: bob.accountId },
        { token_id: "3", approved_account_id: bob.accountId },
      ],
      "approving"
    );
    test.is(
      await store.view("nft_approval_id", {
        token_id: "0",
        account_id: bob.accountId,
      }),
      0
    );

    // ----------------------------- batch approve -----------------------------
    const batchApproveCall = await alice
      .call_raw(
        store,
        "nft_batch_approve",
        { token_ids: ["1", "2"], account_id: bob.accountId },
        { attachedDeposit: mNEAR(1.61) } // no value for this in mintbase-js
      )
      .catch(failPromiseRejection("batch approving"));
    // check event logs
    assertEventLogs(
      test,
      (batchApproveCall as TransactionResult).logs,
      [
        {
          standard: "nep171",
          version: "1.0.0",
          event: "nft_approve",
          // TODO::store::low: unstringify
          data: JSON.stringify([
            // FIXME::store::medium: token_id should be a string
            { token_id: 1, approval_id: 1, account_id: bob.accountId },
            // FIXME::store::medium: token_id should be a string
            { token_id: 2, approval_id: 2, account_id: bob.accountId },
          ]),
        },
      ],
      "batch approving"
    );

    await assertContractPanics(test, [
      // try batch approving when not owning token
      [
        async () =>
          bob.call(
            store,
            "nft_batch_approve",
            { token_ids: ["2", "3"], account_id: bob.accountId },
            { attachedDeposit: mNEAR(1.61) }
          ),
        "panicked at 'assertion failed: token.is_pred_owner()',",
        "Bob tried batch approving on unowned tokens",
      ],
      // require at sufficient deposit to cover storage rent
      [
        async () =>
          alice.call(
            store,
            "nft_batch_approve",
            { token_ids: ["3"], account_id: bob.accountId },
            { attachedDeposit: mNEAR(0.8) }
          ),
        //TODO::store::low: consistent error messages
        "panicked at 'deposit less than: 800000000000000000000',",
        "Alice tried batch approving with insufficient deposit",
      ],
    ]);

    // assert correctness of current approvals
    await assertApprovals(
      { test, store },
      [
        { token_id: "0", approved_account_id: bob.accountId, approval_id: 0 },
        { token_id: "1", approved_account_id: bob.accountId, approval_id: 1 },
        { token_id: "2", approved_account_id: bob.accountId, approval_id: 2 },
      ],
      "batch approving"
    );
    await assertNoApprovals(
      { test, store },
      [{ token_id: "3", approved_account_id: bob.accountId }],
      "batch approving"
    );

    // -------------------------------- revoke ---------------------------------
    // get bob's balance to check the refunding
    const aliceBalance1 = await getBalance(alice);
    const revokeCall = await alice
      .call_raw(
        store,
        "nft_revoke",
        {
          token_id: "2",
          account_id: bob.accountId,
        },
        { attachedDeposit: "1" }
      )
      .catch(failPromiseRejection("revoking"));
    // const aliceBalance2 = await getBalance(alice);
    // const balanceDiff = aliceBalance1.sub(aliceBalance2);
    // const gas = (revokeCall as TransactionResult).gas_burnt;
    // const nearGasBN = new BN(gas.toString()).mul(new BN(100e6)).toString();
    // const nearGas = new ava.NEAR(nearGasBN);
    // test.log(`Alice's balance before revoking: ${aliceBalance1.toHuman()}`);
    // test.log(`Alice's balance after revoking:  ${aliceBalance2.toHuman()}`);
    // test.log(`Difference:                      ${balanceDiff.toHuman()}`);
    // test.log(`Gas costs (1 Tgas = 0.3 mNEAR):  ${nearGas.toHuman()}`);
    // test.log(`Gas costs (gas units):           ${gas.toHuman()}`);
    // test.fail();

    // check event logs
    assertEventLogs(
      test,
      (revokeCall as TransactionResult).logs,
      [
        {
          standard: "nep171",
          version: "1.0.0",
          event: "nft_revoke",
          // FIXME::store::medium: token_id should be a string
          // TODO::store::low: for `nft_approve`, data is an array, here
          //  it's an object -> should have the same predictable structure
          data: JSON.stringify({ token_id: 2, account_id: bob.accountId }),
        },
      ],
      "revoking"
    );
    // check if revoking refunds the storage deposit
    // TODO::idk::medium: 6 mNEAR gone missing -> create issue on github
    // await assertBalanceChange(
    //   test,
    //   {
    //     account: alice,
    //     // subtract the yoctoNEAR deposit
    //     ref: aliceBalance1.sub(new BN("1")),
    //     diff: mNEAR(0.8),
    //     gas: (revokeCall as TransactionResult).gas_burnt,
    //   },
    //   "Revoking"
    // );

    await assertContractPanics(test, [
      // try revoking when not owning token
      [
        async () =>
          bob.call(
            store,
            "nft_revoke",
            {
              token_id: "1",
              account_id: bob.accountId,
            },
            { attachedDeposit: "1" }
          ),
        "panicked at 'assertion failed: token.is_pred_owner()',",
        "Bob tried revoking on unowned token",
      ],
      // require at least one yoctoNEAR to revoke
      [
        async () =>
          alice.call(store, "nft_revoke", {
            token_id: "0",
            account_id: bob.accountId,
          }),
        "Requires attached deposit of exactly 1 yoctoNEAR",
        "Alice tried revoking without yoctoNEAR deposit",
      ],
    ]);

    // assert correctness of current approvals
    await assertApprovals(
      { test, store },
      [
        { token_id: "0", approved_account_id: bob.accountId, approval_id: 0 },
        { token_id: "1", approved_account_id: bob.accountId, approval_id: 1 },
      ],
      "revoking"
    );
    await assertNoApprovals(
      { test, store },
      [
        { token_id: "2", approved_account_id: bob.accountId },
        { token_id: "3", approved_account_id: bob.accountId },
      ],
      "revoking"
    );

    // ------------------------------ revoke_all -------------------------------
    // prior to revoking all, we need a token with two approvals
    await alice.call(
      store,
      "nft_batch_approve",
      { token_ids: ["0", "1"], account_id: carol.accountId },
      { attachedDeposit: mNEAR(1.61) } // no value for this in mintbase-js
    );
    await assertApprovals(
      { test, store },
      [
        { token_id: "0", approved_account_id: carol.accountId, approval_id: 3 },
        { token_id: "1", approved_account_id: carol.accountId, approval_id: 4 },
      ],
      "preparing revoke_all"
    );

    // actual call
    // const aliceBalance2 = await getBalance(alice);
    const revokeAllCall = await alice
      .call_raw(
        store,
        "nft_revoke_all",
        { token_id: "1" },
        { attachedDeposit: "1" }
      )
      .catch(failPromiseRejection("revoking all"));
    // check event logs
    assertEventLogs(
      test,
      (revokeAllCall as TransactionResult).logs,
      [
        {
          standard: "nep171",
          version: "1.0.0",
          event: "nft_revoke_all",
          // TODO::store::medium: wtf is this format?
          data: JSON.stringify({ data: "1" }),
        },
      ],
      "revoking all"
    );
    // // check if revoking all refunds the required security deposit
    // // FIXME::testing::low: this cannot test properly because the cost is so low
    // // -> use TransactionResult::gas_burnt()
    // await assertBalanceChange(
    //   test,
    //   { account: alice, ref: aliceBalance2, diff: mNEAR(1.6) },
    //   "Revoking all"
    // );

    await assertContractPanics(test, [
      // try revoking all when not owning token
      [
        async () =>
          bob.call(
            store,
            "nft_revoke_all",
            { token_id: "0" },
            { attachedDeposit: "1" }
          ),
        "panicked at 'assertion failed: token.is_pred_owner()',",
        "Bob tried revoking all on unowned token",
      ],
      // require at least one yoctoNEAR to revoke all
      [
        async () => alice.call(store, "nft_revoke_all", { token_id: "0" }),
        "Requires attached deposit of exactly 1 yoctoNEAR",
        "Alice tried revoking all without yoctoNEAR deposit",
      ],
    ]);

    // // assert correctness of current approvals
    await assertApprovals(
      { test, store },
      [
        { token_id: "0", approved_account_id: bob.accountId, approval_id: 0 },
        { token_id: "0", approved_account_id: carol.accountId, approval_id: 3 },
      ],
      "revoking all"
    );
    await assertNoApprovals(
      { test, store },
      [
        { token_id: "1", approved_account_id: carol.accountId },
        { token_id: "1", approved_account_id: bob.accountId },
        { token_id: "2", approved_account_id: bob.accountId },
        { token_id: "3", approved_account_id: bob.accountId },
      ],
      "revoking all"
    );
  }
);

STORE_WORKSPACE.test(
  "approvals::minting",
  async (test, { alice, bob, carol, store }) => {
    const failPromiseRejection = (msg: string) => (e: any) => {
      test.log(`Promise rejected while ${msg}:`);
      test.log(e);
      test.fail();
    };

    // ---------------------------- authorized mint ----------------------------
    // TODO::store::low: this increases storage, shouldn't it then require
    //  a sufficient deposit? -> this is not third party-territory, only the
    //  owner can call this
    const grantMinterCall = await alice
      .call_raw(
        store,
        "grant_minter",
        { account_id: bob.accountId },
        { attachedDeposit: "1" }
      )
      .catch(failPromiseRejection("grant minting rights"));

    // check logs
    assertEventLogs(
      test,
      (grantMinterCall as TransactionResult).logs,
      [
        {
          standard: "nep171",
          version: "1.0.0",
          event: "nft_grant_minter",
          // TODO::store::medium: wtf is this format?
          data: JSON.stringify({ data: bob.accountId }),
        },
      ],
      "grant minting rights"
    );

    await assertContractPanics(test, [
      // only owner can grant minting rights
      [
        async () =>
          bob.call(
            store,
            "grant_minter",
            { account_id: bob.accountId },
            { attachedDeposit: "1" }
          ),
        "panicked at 'assertion failed: `(left == right)`",
        "Bob tried granting himself minting rights",
      ],
      //  require deposit
      [
        async () =>
          alice.call(store, "grant_minter", { account_id: bob.accountId }),
        "Requires attached deposit of exactly 1 yoctoNEAR",
        "Alice tried to grant minting rights without yoctoNEAR deposit",
      ],
    ]);

    // check contract state (implicitly tests `check_is_minter`)
    test.true(
      await store.view("check_is_minter", { account_id: bob.accountId }),
      "Failed to grant minting rights to Bob"
    );
    test.false(
      await store.view("check_is_minter", { account_id: carol.accountId }),
      "How on earth did Carol get minting rights?"
    );
    // checking the list_minters method
    test.deepEqual(
      await store.view("list_minters"),
      [alice.accountId, bob.accountId],
      "Bad minters list after granting minting rigths to Bob"
    );

    // actual minting
    // TODO::store::low: shouldn't third party minting require deposits to
    //  cover storage costs? -> otherwise third-party minters might exhaust a
    //  contracts storage
    const batchMintCall = await bob
      .call_raw(
        store,
        "nft_batch_mint",
        { owner_id: bob.accountId, num_to_mint: 2, metadata: {} },
        { attachedDeposit: "1" }
      )
      .catch(failPromiseRejection("approved minting"));

    // check logs
    assertEventLogs(
      test,
      (batchMintCall as TransactionResult).logs,
      [
        {
          standard: "nep171",
          version: "1.0.0",
          event: "nft_mint",
          data: [
            {
              owner_id: bob.accountId,
              token_ids: ["0", "1"],
              memo: JSON.stringify({
                royalty: null,
                split_owners: null,
                meta_id: null,
                meta_extra: null,
                minter: bob.accountId,
              }),
            },
          ],
        },
      ],
      "approved minting"
    );

    // check contract state
    assertContractTokenOwners(
      { test, store },
      [
        { token_id: "0", owner_id: bob.accountId },
        { token_id: "1", owner_id: bob.accountId },
      ],
      "approved minting"
    );

    // revoke minting rights
    const revokeMinterCall = await alice
      .call_raw(
        store,
        "revoke_minter",
        { account_id: bob.accountId },
        { attachedDeposit: "1" }
      )
      .catch(failPromiseRejection("revoke minting rights"));

    // check logs
    assertEventLogs(
      test,
      (revokeMinterCall as TransactionResult).logs,
      [
        {
          standard: "nep171",
          version: "1.0.0",
          event: "nft_revoke_minter",
          // TODO::store::medium: wtf is this format?
          data: JSON.stringify({ data: bob.accountId }),
        },
      ],
      "approved minting"
    );

    await assertContractPanics(test, [
      // only owner can revoke minting rights
      [
        async () =>
          bob.call(
            store,
            "revoke_minter",
            { account_id: bob.accountId },
            { attachedDeposit: "1" }
          ),
        "panicked at 'assertion failed: `(left == right)`",
        "Bob tried to revoke his minting rights",
      ],
      // requires yoctoNEAR deposit
      [
        async () =>
          alice.call(store, "revoke_minter", { account_id: bob.accountId }),
        "Requires attached deposit of exactly 1 yoctoNEAR",
        "Alice tried to revoke minting rights without yoctoNEAR deposit",
      ],
      // owner cannot revoke their own minting rights
      [
        async () =>
          alice.call(
            store,
            "revoke_minter",
            { account_id: alice.accountId },
            { attachedDeposit: "1" }
          ),
        // TODO::testing::low: look for the comment AFTER the failed
        //  assertion message (in this case: "can't revoke owner")
        // TODO::testing::low: look for similar test cases where I might
        //  have missed this
        "panicked at 'assertion failed: `(left != right)`",
        "Alice tried to revoke her own minting rights",
      ],
    ]);

    // check contract state
    test.false(
      await store.view("check_is_minter", { account_id: bob.accountId }),
      "Failed to revoke Bob's minting rights"
    );
    // checking the list_minters method
    test.deepEqual(
      await store.view("list_minters"),
      [alice.accountId],
      "Bad minters list after granting minting rigths to Bob"
    );
  }
);

STORE_WORKSPACE.test(
  "approvals::token-actions",
  async (test, { alice, bob, carol, store }) => {
    const failPromiseRejection = (msg: string) => (e: any) => {
      test.log(`Promise rejected while ${msg}:`);
      test.log(e);
      test.fail();
    };

    await batchMint({ owner: alice, store, num_to_mint: 5 }).catch(
      failPromiseRejection("minting")
    );

    await alice
      .call(
        store,
        "nft_batch_approve",
        {
          token_ids: ["0", "1", "2", "3"],
          account_id: bob.accountId,
        },
        { attachedDeposit: mNEAR(3.21) } // no value for this in mintbase-js
      )
      .catch(failPromiseRejection("approving"));

    // -------------------------- authorized transfer --------------------------
    const transferCall = await bob
      .call_raw(
        store,
        "nft_transfer",
        { receiver_id: carol.accountId, token_id: "0", approval_id: 0 },
        { attachedDeposit: "1" }
      )
      .catch(failPromiseRejection("transferring (approved)"));
    assertEventLogs(
      test,
      (transferCall as TransactionResult).logs,
      [
        {
          standard: "nep171",
          version: "1.0.0",
          event: "nft_transfer",
          data: [
            {
              authorized_id: null, // FIXME::store::medium: why null?
              old_owner_id: alice.accountId,
              new_owner_id: carol.accountId,
              token_ids: ["0"],
              memo: null,
            },
          ],
        },
      ],
      "transferring (approved)"
    );

    await assertContractPanics(test, [
      // try transferring without approval ID
      [
        async () => {
          await bob.call(
            store,
            "nft_transfer",
            { receiver_id: carol.accountId, token_id: "1" },
            { attachedDeposit: "1" }
          );
        },
        "panicked at 'approval_id required'",
        "Bob tried transferring (approved) without approval_id",
      ],
      // require at least one yoctoNEAR to transfer
      [
        async () => {
          await bob.call(store, "nft_transfer", {
            receiver_id: carol.accountId,
            token_id: "1",
            approval_id: 1,
          });
        },
        "Requires attached deposit of exactly 1 yoctoNEAR",
        "Bob tried transferring (approved) without yoctoNEAR deposit",
      ],
      // TODO::testing::medium workaround until fixed for not being able to
      //  check absence of approval
      [
        async () => {
          await bob.call(
            store,
            "nft_transfer",
            { receiver_id: carol.accountId, token_id: "0", approval_id: 0 },
            { attachedDeposit: "1" }
          );
        },
        // TODO::store::low: better error messages
        "panicked at 'assertion failed: self.nft_is_approved_internal(&token, env::predecessor_account_id(),",
        "Bob tried transferring (approved) without yoctoNEAR deposit",
      ],
    ]);

    // token must now belong to carol
    await assertContractTokenOwners(
      { test, store },
      [
        { token_id: "0", owner_id: carol.accountId },
        { token_id: "1", owner_id: alice.accountId },
        { token_id: "2", owner_id: alice.accountId },
        { token_id: "3", owner_id: alice.accountId },
      ],
      "Bad ownership state after approved transfer"
    );
    // approval must have cleared -> FIXME: cannot check properly, because API is broken
    assertNoApproval(
      { test, store },
      { token_id: "1", approved_account_id: bob.accountId },
      "Bob didn't loose approval after transfer"
    );

    // // ----------------------- authorized batch_transfer -----------------------
    // // currently, only the owner of tokens may batch-transfer them
    // const batchTransferCall = await bob
    //   .call_raw(
    //     store,
    //     "nft_batch_transfer",
    //     {
    //       token_ids: [
    //         // ["1", bob.accountId],
    //         ["2", carol.accountId],
    //       ],
    //     },
    //     { attachedDeposit: "1" }
    //   )
    //   // FIXME::testing::medium: tokens loaned?!
    //   .catch(failPromiseRejection("batch transferring (approved)"));

    // assertEventLogs(
    //   test,
    //   (batchTransferCall as TransactionResult).logs,
    //   [
    //     {
    //       standard: "nep171",
    //       version: "1.0.0",
    //       event: "nft_transfer",
    //       data: [
    //         {
    //           authorized_id: null,
    //           old_owner_id: alice.accountId,
    //           new_owner_id: bob.accountId,
    //           token_ids: ["1"],
    //           memo: null,
    //         },
    //         {
    //           authorized_id: null,
    //           old_owner_id: alice.accountId,
    //           new_owner_id: carol.accountId,
    //           token_ids: ["2"],
    //           memo: null,
    //         },
    //       ],
    //     },
    //   ],
    //   "batch transferring (approved)"
    // );

    // // await assertContractPanics(test, [
    // //   // TODO::testing::low: try batch transferring without approval IDs
    // //   [async () => {}, " ".repeat(180), ""],
    // //   // TODO::testing::low: require at least one yoctoNEAR to approve
    // //   [async () => {}, " ".repeat(180), ""],
    // // ]);

    // await assertContractTokenOwners(
    //   test,
    //   store,
    //   [
    //     { id: "0", owner_id: carol.accountId },
    //     { id: "1", owner_id: bob.accountId },
    //     { id: "2", owner_id: carol.accountId },
    //     { id: "3", owner_id: alice.accountId },
    //   ],
    //   "Bad ownership state after approved batch transfer"
    // );

    // // ---------------------------- authorized burn ----------------------------
    // // currently, only the owner of a token may burn it
    // const burnCall = await bob
    //   .call_raw(
    //     store,
    //     "nft_batch_burn",
    //     { token_ids: ["3"] },
    //     { attachedDeposit: "1" }
    //   )
    //   .catch(failPromiseRejection("burning (approved)"));
    // assertEventLogs(
    //   test,
    //   (burnCall as TransactionResult).logs,
    //   [
    //     {
    //       standard: "nep171",
    //       version: "1.0.0",
    //       event: "nft_burn",
    //       data: [
    //         {
    //           owner_id: "alice.test.near",
    //           authorized_id: null,
    //           token_ids: ["4", "5"],
    //           memo: null,
    //         },
    //       ],
    //     },
    //   ],
    //   "burning (approved)"
    // );

    // await assertContractPanics(test, [
    //   // TODO::testing::low: try approving when not owning token
    //   [async () => {}, " ".repeat(180), ""],
    //   // TODO::testing::low: require at least one yoctoNEAR to approve
    //   [async () => {}, " ".repeat(180), ""],
    // ]);
  }
);
