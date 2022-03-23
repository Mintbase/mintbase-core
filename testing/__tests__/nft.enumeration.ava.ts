import { STORE_WORKSPACE, assertTokensAre, batchMint } from "./test-utils";

STORE_WORKSPACE.test("enumeration", async (test, { alice, bob, store }) => {
  const failPromiseRejection = (msg: string) => (e: any) => {
    test.log(`Promise rejected while ${msg}:`);
    test.log(e);
    test.fail();
  };

  // seeding: mint 4 tokens (2 for Alice, 2 for Bob)
  await batchMint({ owner: alice, store, num_to_mint: 2 }).catch(
    failPromiseRejection("minting")
  );
  await batchMint({
    owner: alice,
    store,
    num_to_mint: 2,
    owner_id: bob.accountId,
  }).catch(failPromiseRejection("minting"));

  // testing `nft_total_supply` and `nft_supply_for_owner`
  test.is(await store.view("nft_total_supply", {}), "4");
  test.is(
    await store.view("nft_supply_for_owner", { account_id: alice.accountId }),
    "2"
  );
  test.is(
    await store.view("nft_supply_for_owner", { account_id: bob.accountId }),
    "2"
  );

  // call `nft_tokens` without params
  assertTokensAre(
    test,
    await store.view("nft_tokens", {}),
    [
      { token_id: "0", owner_id: alice.accountId },
      { token_id: "1", owner_id: alice.accountId },
      { token_id: "2", owner_id: bob.accountId },
      { token_id: "3", owner_id: bob.accountId },
    ],
    "`nft_tokens({})` output is wrong"
  );

  // call `nft_tokens` with starting index
  assertTokensAre(
    test,
    await store.view("nft_tokens", { from_index: "2" }),
    [
      { token_id: "2", owner_id: bob.accountId },
      { token_id: "3", owner_id: bob.accountId },
    ],
    "`nft_tokens({ from_index })` output is wrong"
  );

  // call `nft_tokens` with starting index and limit
  // FIXME::contracts::medium: according to standard, `limit` is not the
  //  index of the last token, but the maximum number of tokens to return
  assertTokensAre(
    test,
    // FIXME::contracts::medium: limit should be 2
    await store.view("nft_tokens", { from_index: "1", limit: 3 }),
    [
      { token_id: "1", owner_id: alice.accountId },
      { token_id: "2", owner_id: bob.accountId },
    ],
    "`nft_tokens({ from_index, limit })` output is wrong"
  );

  // call `nft_tokens_for_owner` for Bob without params
  assertTokensAre(
    test,
    await store.view("nft_tokens_for_owner", { account_id: bob.accountId }),
    [
      { token_id: "2", owner_id: bob.accountId },
      { token_id: "3", owner_id: bob.accountId },
    ],
    "`nft_tokens_for_owner({})` output is wrong"
  );

  // call `nft_tokens_for_owner` for Bob with starting index
  assertTokensAre(
    test,
    await store.view("nft_tokens_for_owner", {
      account_id: bob.accountId,
      // TODO::contracts::medium: should this index refer to token_id, or the
      //  index of token for this token owner? -> if token_id, then use "3"
      from_index: "1",
    }),
    [{ token_id: "3", owner_id: bob.accountId }],
    "`nft_tokens_for_owner({ from_index })` output is wrong"
  );

  // call `nft_tokens_for_owner` for Bob with starting index and limit
  assertTokensAre(
    test,
    await store.view("nft_tokens_for_owner", {
      account_id: bob.accountId,
      // TODO::contracts::medium: should this index refer to token_id, or the
      //  index of token for this token owner? -> if token_id, then use "2"
      from_index: "0",
      // Unlike `nft_tokens`, here the limit behaves according to spec
      // (see above)
      limit: 1,
    }),
    [{ token_id: "2", owner_id: bob.accountId }],
    "`nft_tokens_for_owner({ from_index, limit })` output is wrong"
  );
});

// TODO:
// - [] test `nft_tokens_for_owner_set`, but only after syncing back wether it
//      is used e.g. in mintbase-js, otherwise make it private
