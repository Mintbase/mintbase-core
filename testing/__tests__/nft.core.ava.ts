import { TransactionResult } from "near-workspaces-ava";
import {
  assertContractPanics,
  FACTORY_WORKSPACE,
  DEPLOY_STORE_RENT,
  DEPLOY_STORE_GAS,
  assertContractTokenOwners,
  assertEventLogs,
  failPromiseRejection,
} from "./test-utils";

FACTORY_WORKSPACE.test(
  "core",
  async (test, { root, factory, alice, bob, carol }) => {
    // const failPromiseRejection = (msg: string) => (e: any) => {
    //   test.log(`Promise rejected while ${msg}:`);
    //   test.log(e);
    //   test.fail();
    // };

    // store creation
    await alice
      .call(
        factory,
        "create_store",
        {
          owner_id: alice.accountId,
          metadata: {
            spec: "nft-1.0.0",
            name: "alice",
            symbol: "ALICE",
          },
        },
        { attachedDeposit: DEPLOY_STORE_RENT, gas: DEPLOY_STORE_GAS }
      )
      .catch(failPromiseRejection(test, "creating store"));
    const store = root.getFullAccount(`alice.${factory.accountId}`);
    // TODO::testing::medium: check event logs

    // TODO::testing::medium trying deployment with forbidden names
    //  - reserved names: "market", "loan"
    //  - taken names, in this case "alice"
    // TODO::testing::medium deployment with more metadata
    // TODO::testing::medium check this contracts metadata

    // minting
    const mintCall = await alice
      .call_raw(
        store,
        "nft_batch_mint",
        { owner_id: alice.accountId, metadata: {}, num_to_mint: 6 },
        { attachedDeposit: "1" }
      )
      .catch(failPromiseRejection(test, "minting"));

    // check minting logs
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
              token_ids: ["0", "1", "2", "3", "4", "5"],
              // memo should be a string, as it's standardized like that!
              memo: JSON.stringify({
                royalty: null,
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

    // inspecting minted tokens (implicitly tests `nft_token`)
    await assertContractTokenOwners(
      { test, store },
      [
        { id: "0", owner_id: alice.accountId },
        { id: "1", owner_id: alice.accountId },
        { id: "2", owner_id: alice.accountId },
        { id: "3", owner_id: alice.accountId },
        { id: "4", owner_id: alice.accountId },
        { id: "5", owner_id: alice.accountId },
      ],
      "After minting"
    ).catch(failPromiseRejection(test, "checking token format"));

    await assertContractPanics(test, [
      // try to mint while not being minter
      [
        async () => {
          await bob.call(
            store,
            "nft_batch_mint",
            { owner_id: bob.accountId, metadata: {}, num_to_mint: 1 },
            { attachedDeposit: "1" }
          );
        },
        `panicked at '${bob.accountId} not a minter'`,
        "Bob tried minting without minter permission",
      ],
      // try minting without yoctoNEAR deposit
      [
        async () => {
          await alice.call(store, "nft_batch_mint", {
            owner_id: alice.accountId,
            metadata: {},
            num_to_mint: 1,
          });
        },
        "panicked at 'assertion failed: env::attached_deposit() >= 1'",
        "Alice tried minting without yoctoNEAR deposit",
      ],
    ]);

    // transfering a single token
    const transferCall = await alice
      .call_raw(
        store,
        "nft_transfer",
        { receiver_id: bob.accountId, token_id: "0" },
        { attachedDeposit: "1" }
      )
      .catch(failPromiseRejection(test, "transferring"));

    // check transfer logs
    // TODO::contracts::low: should empty fields be serialized as null or
    //  simply omitted? -> null might make sense for the indexer
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
              authorized_id: null,
              old_owner_id: "alice.test.near",
              new_owner_id: "bob.test.near",
              token_ids: ["0"],
              memo: null,
            },
          ],
        },
      ],
      "transferring"
    );

    await assertContractPanics(test, [
      // try to transfer unowned token (random bob)
      [
        async () => {
          await bob.call(
            store,
            "nft_transfer",
            { receiver_id: bob.accountId, token_id: "1" },
            { attachedDeposit: "1" }
          );
        },
        "panicked at 'approval_id required'",
        "Bob tried to transfer an unowned token",
      ],
      // try to transfer unowned token (store owner)
      [
        async () => {
          await alice.call(
            store,
            "nft_transfer",
            { receiver_id: alice.accountId, token_id: "0" },
            { attachedDeposit: "1" }
          );
        },
        "panicked at 'approval_id required'",
        "Alice tried to transfer an unowned token",
      ],
    ]);

    // batch transfering tokens
    const batchTransferCall = await alice
      .call_raw(
        store,
        "nft_batch_transfer",
        // TODO::contracts::low: undescriptive param name
        // TODO::contracts::low: why is this a tuple whereas `nft_transfer` is
        //  a record?
        // TODO::contracts::low: missing memo parameter?
        {
          token_ids: [
            ["1", bob.accountId],
            ["2", carol.accountId],
          ],
        },
        { attachedDeposit: "1" }
      )
      .catch(failPromiseRejection(test, "batch transferring"));

    // check transfer logs
    // TODO::contracts::low: should empty fields be serialized as null or
    //  simply omitted? -> null might make sense for the indexer
    // TODO::testing::low: assert event when batch transferring two to the same
    // address
    assertEventLogs(
      test,
      (batchTransferCall as TransactionResult).logs,
      [
        {
          standard: "nep171",
          version: "1.0.0",
          event: "nft_transfer",
          data: [
            {
              authorized_id: null,
              old_owner_id: "alice.test.near",
              new_owner_id: "bob.test.near",
              token_ids: ["1"],
              memo: null,
            },
            {
              authorized_id: null,
              old_owner_id: "alice.test.near",
              new_owner_id: "carol.test.near",
              token_ids: ["2"],
              memo: null,
            },
          ],
        },
      ],
      "batch transferring"
    );

    await assertContractPanics(test, [
      // try to batch transfer unowned tokens (random bob)
      [
        async () => {
          await bob.call(
            store,
            "nft_batch_transfer",
            {
              token_ids: [
                ["1", carol.accountId],
                ["2", bob.accountId],
              ],
            },
            { attachedDeposit: "1" }
          );
        },
        "panicked at 'assertion failed: token.is_pred_owner()'",
        "Bob tried to batch transfer unowned tokens",
      ],
      // try to batch transfer unowned tokens (store owner)
      [
        async () => {
          await alice.call(
            store,
            "nft_batch_transfer",
            {
              token_ids: [
                ["0", alice.accountId],
                ["1", alice.accountId],
              ],
            },
            { attachedDeposit: "1" }
          );
        },
        "panicked at 'assertion failed: token.is_pred_owner()'",
        "Alice tried to batch transfer unowned tokens",
      ],
      // try to batch transfer without yoctoNEAR deposit
      [
        async () => {
          await alice.call(store, "nft_batch_transfer", {
            token_ids: [
              ["0", alice.accountId],
              ["1", alice.accountId],
            ],
          });
        },
        "Requires attached deposit of exactly 1 yoctoNEAR",
        "Alice tried to batch transfer tokens without yoctoNEAR deposit",
      ],
    ]);

    // checking token ownership
    await assertContractTokenOwners(
      { test, store },
      [
        { id: "0", owner_id: bob.accountId },
        { id: "1", owner_id: bob.accountId },
        { id: "2", owner_id: carol.accountId },
        { id: "3", owner_id: alice.accountId },
        { id: "4", owner_id: alice.accountId },
        { id: "5", owner_id: alice.accountId },
      ],
      "After transfers"
    ).catch(failPromiseRejection(test, "checking token ownership"));

    // burning tokens
    const burnCall = await alice
      .call_raw(
        store,
        "nft_batch_burn",
        { token_ids: ["4", "5"] },
        { attachedDeposit: "1" }
      )
      .catch(failPromiseRejection(test, "burning"));

    // check burn logs
    assertEventLogs(
      test,
      (burnCall as TransactionResult).logs,
      [
        {
          standard: "nep171",
          version: "1.0.0",
          event: "nft_burn",
          data: [
            {
              owner_id: "alice.test.near",
              authorized_id: null,
              token_ids: ["4", "5"],
              memo: null,
            },
          ],
        },
      ],
      "burning"
    );

    await assertContractPanics(test, [
      // try to burn unowned tokens (random bob)
      [
        async () => {
          await bob.call(
            store,
            "nft_batch_burn",
            { token_ids: ["1", "2"] },
            { attachedDeposit: "1" }
          );
        },
        // TODO::contracts::low: msg should be more descriptive
        "panicked at 'assertion failed: `(left == right)`",
        "Bob tried to burn unowned tokens",
      ],
      // try to burn unowned tokens (store owner)
      [
        async () => {
          await alice.call(
            store,
            "nft_batch_burn",
            { token_ids: ["0"] },
            { attachedDeposit: "1" }
          );
        },
        // TODO::contracts::low: msg should be more descriptive
        "panicked at 'assertion failed: `(left == right)`",
        "Alice tried to burn unowned tokens",
      ],
      // try to burn tokens without deposit
      [
        async () => {
          await alice.call(store, "nft_batch_burn", {
            token_ids: ["3"],
          });
        },
        "Requires attached deposit of exactly 1 yoctoNEAR",
        "Alice tried to burn tokens without yoctoNEAR deposit",
      ],
      // TODO: figure out if alice is still token owner
      // TODO::testing::medium: can no longer transfer burned token
      // TODO::testing::medium: cannot burn token twice
    ]);

    // TODO::testing::low: transfer store ownership
    // TODO::testing::low: try to transfer store ownership (random bob)
    // TODO::testing::low: try to transfer store ownership without yN deposit

    // TODO::testing::low: try to undeploy contract (random bob)
    // TODO::testing::low: undeploy contract (store owner)
  }
);
