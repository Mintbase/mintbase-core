import avaTest from "ava";
import { failPromiseRejection } from "./utils/index.js";
import { setup } from "./setup.js";

const test = setup(avaTest);

test("payout::splits", async (test) => {
  const { alice, store } = test.context.accounts;

  const split_owners = (() => {
    const o: Record<string, number> = {};
    o["a.near"] = 6000;
    o["b.near"] = 4000;
    return o;
  })();

  await alice
    .call(
      store,
      "nft_batch_mint",
      {
        owner_id: alice.accountId,
        metadata: {},
        num_to_mint: 1,
        split_owners,
      },
      { attachedDeposit: "1" }
    )
    .catch(failPromiseRejection(test, "minting"));

  const payout = (() => {
    const p: Record<string, string> = {};
    p["a.near"] = "6000000000000000";
    p["b.near"] = "4000000000000000";
    return p;
  })();
  test.deepEqual(
    await store.view("nft_payout", {
      token_id: "0",
      balance: "10000000000000000",
    }),
    { payout }
  );
});

test("payout::royalties", async (test) => {
  const { alice, store } = test.context.accounts;

  const split_between = (() => {
    const o: Record<string, number> = {};
    o["a.near"] = 5000;
    o["b.near"] = 5000;
    return o;
  })();

  await alice
    .call(
      store,
      "nft_batch_mint",
      {
        owner_id: alice.accountId,
        metadata: {},
        num_to_mint: 1,
        royalty_args: { split_between, percentage: 4000 },
      },
      { attachedDeposit: "1" }
    )
    .catch(failPromiseRejection(test, "minting"));

  const payout = (() => {
    const p: Record<string, string> = {};
    p["a.near"] = "2000000000000000";
    p["b.near"] = "2000000000000000";
    p[alice.accountId] = "6000000000000000";
    return p;
  })();
  test.deepEqual(
    await store.view("nft_payout", {
      token_id: "0",
      balance: "10000000000000000",
    }),
    { payout }
  );
});

test("payout::royalties_splits", async (test) => {
  const { alice, store } = test.context.accounts;

  const split_between = (() => {
    const o: Record<string, number> = {};
    o["a.near"] = 7500;
    o["b.near"] = 2500;
    return o;
  })();

  const split_owners = (() => {
    const o: Record<string, number> = {};
    o["c.near"] = 7500;
    o["d.near"] = 2500;
    return o;
  })();

  await alice
    .call(
      store,
      "nft_batch_mint",
      {
        owner_id: alice.accountId,
        metadata: {},
        num_to_mint: 1,
        royalty_args: { split_between, percentage: 2000 },
        split_owners,
      },
      { attachedDeposit: "1" }
    )
    .catch(failPromiseRejection(test, "minting"));

  const payout = (() => {
    const p: Record<string, string> = {};
    p["a.near"] = "1500000000000000";
    p["b.near"] = "500000000000000";
    p["c.near"] = "6000000000000000";
    p["d.near"] = "2000000000000000";
    return p;
  })();
  test.deepEqual(
    await store.view("nft_payout", {
      token_id: "0",
      balance: "10000000000000000",
    }),
    { payout }
  );
});

test("payout::low_balance", async (test) => {
  const { alice, store } = test.context.accounts;

  const split_owners = (() => {
    const o: Record<string, number> = {};
    o["a.near"] = 6000;
    o["b.near"] = 4000;
    return o;
  })();

  await alice
    .call(
      store,
      "nft_batch_mint",
      {
        owner_id: alice.accountId,
        metadata: {},
        num_to_mint: 1,
        split_owners,
      },
      { attachedDeposit: "1" }
    )
    .catch(failPromiseRejection(test, "minting"));

  const payout = (() => {
    const p: Record<string, string> = {};
    p["a.near"] = "6000";
    p["b.near"] = "4000";
    return p;
  })();
  test.deepEqual(
    await store.view("nft_payout", {
      token_id: "0",
      balance: "10000",
    }),
    { payout }
  );
});

// FIXME: doesn't work
test("payout::max_len", async (test) => {
  const { alice, store } = test.context.accounts;

  const split_owners = (() => {
    const o: Record<string, number> = {};
    o["a.near"] = 1000;
    o["b.near"] = 950;
    o["c.near"] = 900;
    o["d.near"] = 850;
    o["e.near"] = 800;
    o["f.near"] = 750;
    o["g.near"] = 700;
    o["h.near"] = 650;
    o["i.near"] = 600;
    o["j.near"] = 550;
    o["k.near"] = 500;
    o["l.near"] = 450;
    o["m.near"] = 400;
    o["n.near"] = 350;
    o["o.near"] = 300;
    o["p.near"] = 250;
    return o;
  })();

  await alice
    .call(
      store,
      "nft_batch_mint",
      {
        owner_id: alice.accountId,
        metadata: {},
        num_to_mint: 1,
        split_owners,
      },
      { attachedDeposit: "1" }
    )
    .catch(failPromiseRejection(test, "minting"));

  // FIXME: should work with lower number
  const payout = (() => {
    const p: Record<string, string> = {};
    p["a.near"] = "1000000000000000";
    p["b.near"] = "950000000000000";
    p["c.near"] = "900000000000000";
    p["d.near"] = "850000000000000";
    p["e.near"] = "800000000000000";
    p["f.near"] = "750000000000000";
    p["g.near"] = "700000000000000";
    p["h.near"] = "650000000000000";
    p["i.near"] = "600000000000000";
    p["j.near"] = "550000000000000";
    return p;
  })();
  test.deepEqual(
    await store.view("nft_payout", {
      token_id: "0",
      balance: "10000000000000000",
      max_len_payout: 10,
    }),
    { payout }
  );
});
