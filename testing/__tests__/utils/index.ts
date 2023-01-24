import { NearAccount } from "near-workspaces";
import { ExecutionContext } from "ava";

// TODO::testing::low: commenting all my test utils

export * from "./balances.js";
export * from "./panics.js";
export * from "./token.js";
export * from "./approvals.js";
export * from "./events.js";
export * from "./payouts.js";
export * from "./download-contracts.js";

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
  accounts: Record<string, NearAccount>
) {
  const { alice, store, market, factory } = accounts;
  await batchMint({ owner: alice, store, num_to_mint: 2 }).catch(
    failPromiseRejection(test, "minting")
  );

  await market
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
