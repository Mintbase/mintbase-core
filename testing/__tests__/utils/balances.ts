import { Gas, BN, NearAccount } from "near-workspaces";
import * as nearWs from "near-workspaces";
import { ExecutionContext } from "ava";

// TODO: move from this format to `ava.NEAR.parse`

/**
 * Interprets a float as NEAR and builds the corresponding string.
 * Rounded to closest milliNEAR.
 */
export function NEAR(x: number): nearWs.NEAR {
  return mNEAR(x).mul(new nearWs.NEAR(1e3));
}

/**
 * Interprets a float as milliNEAR and builds the corresponding string.
 * Rounded to closest microNEAR.
 */
export function mNEAR(x: number): nearWs.NEAR {
  return uNEAR(x).mul(new nearWs.NEAR(1e3));
}

/**
 * Interprets a float as microNEAR and builds the corresponding string.
 * Rounded to closest nanoNEAR.
 */
export function uNEAR(x: number): nearWs.NEAR {
  return nNEAR(x).mul(new nearWs.NEAR(1e3));
}

/**
 * Interprets a float as nanoNEAR and builds the corresponding string.
 * Rounded to closest picoNEAR.
 */
export function nNEAR(x: number): nearWs.NEAR {
  return new nearWs.NEAR((x * 1e3).toString() + "0".repeat(12));
}

/**
 * Interprets a float as Teragas and builds the corresponding string.
 * Rounded to closest Gigagas.
 */
export function Tgas(x: number): nearWs.Gas {
  return new nearWs.Gas((x * 1e3).toString() + "0".repeat(9));
}

/**
 * Interprets a float as NEAR and builds the corresponding `BN`.
 * Rounded to closest milliNEAR.
 */
export function NEARbn(x: number): BN {
  return new BN(NEAR(x));
}

/**
 * Interprets a float as milliNEAR and builds the corresponding `BN`.
 * Rounded to closest microNEAR.
 */
export function mNEARbn(x: number): BN {
  return new BN(mNEAR(x));
}

/**
 * Interprets a float as microNEAR and builds the corresponding `BN`.
 * Rounded to closest nanoNEAR.
 */
export function uNEARbn(x: number): BN {
  return new BN(uNEAR(x));
}

/**
 * Interprets a float as nanoNEAR and builds the corresponding `BN`.
 * Rounded to closest picoNEAR.
 */
export function nNEARbn(x: number): BN {
  return new BN(nNEAR(x));
}

/**
 * Interprets a float as Teragas and builds the corresponding `BN`.
 * Rounded to closest Gigagas.
 */
export function Tgasbn(x: number): BN {
  return new BN(Tgas(x));
}

/** Maximum possible gas (will be serialized to a u64) */
export const MAX_U64 = new BN("ffffffffffffffff", 16);
/** Gas cost for deploying a store (taken from mintbase-js) */
export const DEPLOY_STORE_GAS = Tgas(200);
/** Storage rent for deploying a store (taken from mintbase-js) */
export const DEPLOY_STORE_RENT = NEAR(7);

/**
 * Mostly a wrapper for getting total balance of an account, might change to
 * available balance in the future.
 */
export async function getBalance(account: NearAccount): Promise<nearWs.NEAR> {
  return (await account.balance()).total;
}

// TODO::testing::low: use this function consistently
/** Asserts balance changes for multiple accounts in parallel */
export async function assertBalanceChanges(
  test: ExecutionContext,
  specs: {
    account: NearAccount;
    ref: nearWs.NEAR;
    diff: nearWs.NEAR;
  }[],
  msg: string
) {
  await Promise.all(specs.map((spec) => assertBalanceChange(test, spec, msg)));
}

/**
 * Asserts the change of an account balance w.r.t. an earlier reference amount.
 * The balance is allowed to be 0.05 NEAR below `ref - diff`, which accounts for
 * gas costs that might have been expended.
 */
export async function assertBalanceChange(
  test: ExecutionContext,
  params: {
    account: NearAccount;
    ref: nearWs.NEAR;
    diff: nearWs.NEAR;
    gas?: Gas;
  },
  msg: string
) {
  const now = await getBalance(params.account);
  if (params.gas) {
    const { gas } = params;
    assertBalanceDiffExact(test, { ...params, now, gas }, msg);
  } else {
    const maxGas = NEAR(0.05).toString(); // allow 40 mNEAR of gas costs
    assertBalanceDiffRange(test, { ...params, now, maxGas }, msg);
  }
}

function assertBalanceDiffExact(
  test: ExecutionContext,
  {
    account,
    now,
    ref,
    diff,
    gas,
  }: {
    account: NearAccount;
    now: nearWs.NEAR;
    ref: nearWs.NEAR;
    diff: nearWs.NEAR;
    gas: Gas;
  },
  msg: string
) {
  const nearGas = new nearWs.NEAR(gas.mul(new BN(100e6)).toString());
  const expected = ref.add(diff).sub(nearGas);
  // test.log({
  //   account: account.accountId,
  //   expected: expected.toString(),
  //   now: now.toString(),
  //   ref: ref.toString(),
  //   diff: diff.toString(),
  //   nearGas: nearGas.toString(),
  // });

  test.true(
    now.eq(expected),
    [
      `${msg}: wrong balance for ${account.accountId}`,
      `\texpected: ${expected.toHuman()}`,
      `\tactual:   ${now.toHuman()}`,
    ].join("\n")
  );

  test.fail(
    [
      `${msg}: balance for ${account.accountId}`,
      `\texpected: ${expected.toHuman()}`,
      `\tactual:   ${now.toHuman()}`,
    ].join("\n")
  );
}

// TODO::testing::low: deprecate this (blocked until gas stuff becomes more sound)
function assertBalanceDiffRange(
  test: ExecutionContext,
  {
    account,
    now,
    ref,
    diff,
    maxGas,
  }: {
    account: NearAccount;
    now: nearWs.NEAR;
    ref: nearWs.NEAR;
    diff: nearWs.NEAR;
    maxGas: string;
  },
  msg: string
) {
  // test.log("entering assertBalanceDiffRange");
  const max = ref.add(new BN(diff));
  const min = max.sub(new BN(maxGas));
  // test.log({
  //   account: account.accountId,
  //   now: now.toString(),
  //   ref: ref.toString(),
  //   diff: diff.toString(), // cannot use toHuman on negative diff!
  //   min: min.toString(),
  //   max: max.toString(),
  // });
  test.true(now.lte(max), `${msg}: balance too high for ${account}`);
  test.true(now.gte(min), `${msg}: balance too low for ${account}`);
}
