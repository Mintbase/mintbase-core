import { NearAccount, TestnetRpc } from "near-workspaces-ava";
import { ExecutionContext } from "ava";

// TODO::testing::low: commenting all my test utils

export * from "./balances";
export * from "./workspaces";
export * from "./panics";
export * from "./token";
export * from "./approvals";
export * from "./events";
export * from "./payouts";
export * from "./download-contracts";

// ---------------------------------- misc ---------------------------------- //
export async function batchMint({
  owner,
  store,
  owner_id,
  num_to_mint,
}: {
  owner: NearAccount;
  store: NearAccount;
  num_to_mint: number;
  owner_id?: string;
}) {
  if (!owner_id) owner_id = owner.accountId;
  await owner.call(
    store,
    "nft_batch_mint",
    {
      owner_id,
      num_to_mint,
      metadata: {},
    },
    { attachedDeposit: "1" }
  );
}

export async function prepareTokenListing(
  test: ExecutionContext,
  { root, alice, store, market, factory }
) {
  await batchMint({ owner: alice, store, num_to_mint: 2 }).catch(
    failPromiseRejection(test, "minting")
  );

  await root
    .call(
      market,
      "update_allowlist",
      { account_id: factory.accountId, state: true },
      { attachedDeposit: "1" }
    )
    .catch(failPromiseRejection(test, "allowing store on market"));
}

// TODO::testing::low: use this function consistently
export function failPromiseRejection(
  test: ExecutionContext,
  msg: string
): (e: any) => void {
  return (e: any) => {
    test.log(`Promise rejected while ${msg}:`);
    test.log(e);
    test.fail();
  };
}

export function hours(x: number): number {
  return Math.round(x * 3600 * 1e9);
}

// ---- xxxx ---- //
