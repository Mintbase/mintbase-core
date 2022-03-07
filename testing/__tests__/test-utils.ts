import { BN, NearAccount, Workspace } from "near-workspaces-ava";
import * as ava from "near-workspaces-ava";
import { ExecutionContext } from "ava";

// TODO::testing::low: commenting all my test utils

// ------------------- gas costs, storage rent, balances -------------------- //
/**
 * Interprets a float as NEAR and builds the corresponding string.
 * Rounded to closest milliNEAR.
 */
export function NEAR(x: number): string {
  return mNEAR(x) + "000";
}

/**
 * Interprets a float as milliNEAR and builds the corresponding string.
 * Rounded to closest microNEAR.
 */
export function mNEAR(x: number): string {
  return uNEAR(x) + "000";
}

/**
 * Interprets a float as microNEAR and builds the corresponding string.
 * Rounded to closest nanoNEAR.
 */
export function uNEAR(x: number): string {
  return nNEAR(x) + "000";
}

/**
 * Interprets a float as nanoNEAR and builds the corresponding string.
 * Rounded to closest picoNEAR.
 */
export function nNEAR(x: number): string {
  return (x * 1e3).toString() + "0".repeat(12);
}

/**
 * Interprets a float as Teragas and builds the corresponding string.
 * Rounded to closest Gigagas.
 */
export function Tgas(x: number): string {
  return (x * 1e3).toString() + "0".repeat(9);
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

// ---------------------------- contract panics ----------------------------- //

/** Asserts multiple panics in parallel to speed up tests */
export async function assertContractPanics(
  test: ExecutionContext,
  params: [() => Promise<void>, string, string][]
) {
  await Promise.all(params.map((p) => assertContractPanic(test, ...p)));
}

/** Asserts that a contract call panics with a given message */
export async function assertContractPanic(
  test: ExecutionContext,
  thrower: () => Promise<void>,
  panicMsg: string,
  assertMsg: string
) {
  // TODO::testing::medium ensure that no logging took place?
  await test
    .throwsAsync(thrower, undefined, `${assertMsg}: succeeded`)
    .then(assertContractPanicMsg(test, panicMsg, assertMsg));
}

/**
 * Asserts that an error returned from a contract call contains a given message
 */
export function assertContractPanicMsg(
  test: ExecutionContext,
  panicMsg: string,
  assertMsg?: string
): (error: any) => void {
  return (error: any) => {
    // The slicing assures we don't assert against source location, the comma at
    // the message end assures that we capture everything but source location
    const expectedPanicMsg = `Smart contract panicked: ${panicMsg}`;
    const actualPanicMsg = error.kind.ExecutionError.slice(
      0,
      expectedPanicMsg.length
    );
    // log full error message in case anything goes wrong
    test.log(error.kind.ExecutionError);

    test.is(
      error.type,
      "FunctionCallError",
      `Wrong error/panic type when ${assertMsg}`
    );
    test.is(
      actualPanicMsg,
      expectedPanicMsg,
      `Wrong error/panic type when ${assertMsg}`
    );
  };
}

// ------------------------------- workspaces ------------------------------- //
async function createAccounts(root: NearAccount): Promise<NearAccount[]> {
  // const alice = await root.createAccount("alice", { initialBalance: NEAR(20) });
  // const bob = await root.createAccount("bob", { initialBalance: NEAR(20) });
  // const carol = await root.createAccount("carol", { initialBalance: NEAR(20) });
  // return [alice, bob, carol];
  return Promise.all([
    root.createAccount("alice", { initialBalance: NEAR(20) }),
    root.createAccount("bob", { initialBalance: NEAR(20) }),
    root.createAccount("carol", { initialBalance: NEAR(20) }),
  ]);
}

/** deploys the factory to a subaccount `factory` of `root` */
export async function deployFactory(root: NearAccount): Promise<NearAccount> {
  return root.createAndDeploy(
    "factory", // subaccount name
    "../wasm/factory.wasm", // path to wasm
    { method: "new", args: {} }
  );
}

/** deploys the market to a subaccount `market` of `root` */
export async function deployMarket(root: NearAccount): Promise<NearAccount> {
  return root.createAndDeploy(
    "market", // subaccount name
    "../wasm/market.wasm", // path to wasm
    { method: "new", args: { init_allowlist: [] } }
  );
}

/**
 * deploys the store to a subaccount `name` of `factory`, setting the store
 * owner to `owner`
 */
export async function deployStore({
  factory,
  owner,
  name,
}: {
  factory: NearAccount;
  owner: NearAccount;
  name: string;
}): Promise<NearAccount> {
  await owner.call(
    factory,
    "create_store",
    {
      owner_id: owner.accountId,
      metadata: {
        spec: "nft-1.0.0",
        name,
        symbol: "ALICE",
      },
    },
    { attachedDeposit: DEPLOY_STORE_RENT, gas: DEPLOY_STORE_GAS }
  );
  return factory.getFullAccount(`alice.${factory.accountId}`);
}

/** A workspace with the factory deployed by root, no store deployed */
export const FACTORY_WORKSPACE = Workspace.init(async ({ root }) => {
  const [alice, bob, carol] = await createAccounts(root);

  const factory = await deployFactory(root);

  return { alice, bob, carol, factory };
});

/** A workspace with the factory deployed by root, store deployed by Alice */
export const STORE_WORKSPACE = Workspace.init(async ({ root }) => {
  const [alice, bob, carol] = await createAccounts(root);

  const factory = await deployFactory(root);
  const store = await deployStore({ factory, owner: alice, name: "alice" });

  return { alice, bob, carol, factory, store };
});

/**
 * A workspace with the factory and market deployed by root,
 * store deployed by Alice
 */
export const MARKET_WORKSPACE = Workspace.init(async ({ root }) => {
  const [alice, bob, carol] = await createAccounts(root);

  const factory = await deployFactory(root);
  const store = await deployStore({ factory, owner: alice, name: "alice" });
  const market = await deployMarket(root);

  return { alice, bob, carol, factory, store, market };
});

// ---------------------------- core token data ----------------------------- //

/** The current interface of a token on the store contract */
export interface MintbaseToken {
  // FIXME::store::high: this should  be a string, not number
  id: number;
  // FIXME::store::high: this should be a string, not object
  owner_id: { Account: string };
}

/** Typescript narrowing function for `MintbaseToken` */
export function isMintbaseToken(x: any): x is MintbaseToken {
  return (
    typeof x.id === "number" &&
    x.owner_id instanceof Object &&
    typeof x.owner_id.Account === "string"
  );
}

// FIXME::store::high make token compliant with this
/** The current interface of a token as described in NEP171 */
export interface Nep171Token {
  id: string;
  owner_id: string;
}

// export function isNep171Token(x: any): x is Nep171Token {
//   return typeof x.id === "string" && typeof x.owner_id === "string";
// }

// FIXME::store::high should be based on NEP171, not the Mintbase token
/**
 * Asserts that a token matches an expected token_id/owner_id pairing.
 */
export function assertTokenIs(
  test: ExecutionContext,
  token: MintbaseToken,
  { id, owner_id }: Nep171Token,
  msg: string
) {
  test.is(`${token.id}`, id, `${msg}: Wrong token_id`);
  test.is(`${token.owner_id.Account}`, owner_id, `${msg}: Wrong owner_id`);
}

/**
 * Asserts that a token matches an expected token_id/owner_id pairing.
 */
export function assertTokensAre(
  test: ExecutionContext,
  actual: MintbaseToken[],
  expected: Nep171Token[],
  msg: string
) {
  test.log("Actual token list:", actual);
  test.log("Expected token list:", expected);
  test.is(
    actual.length,
    expected.length,
    `${msg}: token lists mismatched in length`
  );
  expected.forEach((token, i) => {
    assertTokenIs(test, actual[i], token, msg);
  });
}

// FIXME::store::high should be based on NEP171, not the Mintbase token
/**
 * Asserts the contract state matches an expected token_id/owner_id pairing.
 */
export async function assertContractTokenOwner(
  { test, store }: { test: ExecutionContext; store: NearAccount },
  { id, owner_id }: Nep171Token,
  msg: string
) {
  const token: MintbaseToken = await store.view("nft_token", {
    token_id: id,
  });
  test.true(
    isMintbaseToken(token),
    `${msg}: Not a MintbaseToken (token_id: ${id})`
  );
  assertTokenIs(test, token, { id, owner_id }, msg);
}

/**
 * Asserts that a list of token_id/owner_id pairs match the contract state.
 * Queries all tokens on their own, and thus doesn't suffer from the limitations
 * of the `nft_tokens` method.
 */
export async function assertContractTokenOwners(
  { test, store }: { test: ExecutionContext; store: NearAccount },
  tokens: { id: string; owner_id: string }[],
  msg: string
) {
  await Promise.all(
    tokens.map(async (token) => {
      await assertContractTokenOwner({ test, store }, token, msg);
    })
  );
}

// ------------------------------- approvals -------------------------------- //
interface ApprovalSpec {
  token_id: string;
  approved_account_id: string;
  approval_id?: number;
}

export async function assertApprovals(
  { test, store }: { test: ExecutionContext; store: NearAccount },
  specs: ApprovalSpec[],
  msg: string
) {
  await Promise.all(
    specs.map((spec) => {
      assertApproval({ test, store }, spec, msg);
    })
  );
}

export async function assertApproval(
  { test, store }: { test: ExecutionContext; store: NearAccount },
  spec: ApprovalSpec,
  msg: string
) {
  test.true(await getApproval(store, spec), msg);
}

export async function assertNoApprovals(
  { test, store }: { test: ExecutionContext; store: NearAccount },
  specs: ApprovalSpec[],
  msg: string
) {
  await Promise.all(
    specs.map((spec) => {
      assertNoApproval({ test, store }, spec, msg);
    })
  );
}

export async function assertNoApproval(
  { test, store }: { test: ExecutionContext; store: NearAccount },
  spec: ApprovalSpec,
  msg: string
) {
  // FIXME::testing::medium: remove the early return. (currently blocked)
  // this is currently blocked by the contract requiring us to specify an
  // approval_id
  if (!spec.approval_id) return Promise.resolve(undefined);

  test.false(await getApproval(store, spec), msg);
}

async function getApproval(
  store: NearAccount,
  { token_id, approved_account_id, approval_id }: ApprovalSpec
): Promise<boolean> {
  return store.view("nft_is_approved", {
    token_id,
    approved_account_id,
    approval_id,
  });
}

// TODO::testing::low: use this function consistently
export async function assertMinters(
  { test, store }: { test: ExecutionContext; store: NearAccount },
  specs: [NearAccount, boolean][],
  msg: string
) {
  await Promise.all(
    specs.map((spec) => {
      assertMinter({ test, store }, spec, msg);
    })
  );
}

export async function assertMinter(
  { test, store }: { test: ExecutionContext; store: NearAccount },
  [account, expected]: [NearAccount, boolean],
  msg: string
) {
  test.is(
    await store.view("check_is_minter", { account_id: account.accountId }),
    expected,
    msg
  );
}

// --------------------------------- events --------------------------------- //
export function assertEventLogs(
  test: ExecutionContext,
  actual: string[],
  expected: any[],
  msg: string
) {
  test.is(actual.length, expected.length, `${msg}: mismatched number of logs`);
  actual.forEach((log, i) => {
    assertEventLog(test, log, expected[i], msg);
  });
}

// TODO::testing::low: Use this function consistently
export function assertEventLog(
  test: ExecutionContext,
  actual: string,
  expected: any,
  msg: string
) {
  const baseMsg = `Bad event log for ${msg}`;
  const event = parseEvent(test, actual, baseMsg);
  test.log("Expected:", expected);
  test.deepEqual(event, expected, baseMsg);
}

function parseEvent(test: ExecutionContext, log: string, msg: string) {
  // FIXME::contracts::medium: standard has no space between colon and JSON
  test.is(log.slice(0, 12), "EVENT_JSON: ", `${msg}: Not an event log`);
  test.log("Sliced:", log.slice(12));
  const event = JSON.parse(log.slice(12));
  test.log("Parsed:", event);
  return event;
}

export function assertMakeOfferEvent(
  { test, eventLog }: { test: ExecutionContext; eventLog: string },
  {
    id,
    maker,
    store,
    specs,
  }: {
    id: number;
    maker: NearAccount;
    store: NearAccount;
    specs: {
      token_id: string;
      approval_id: number;
      price: string;
      timeout?: number;
    }[];
  },
  msg: string
) {
  const event: any = parseEvent(test, eventLog, msg);
  test.true(event instanceof Object, `${msg}: Event is not even an object`);
  test.like(
    event,
    {
      standard: "nep171",
      version: "1.0.0",
      event: "nft_make_offer",
    },
    `${msg}: bad event metadata`
  );

  test.is(typeof event.data, "string", `${msg}: event.data is not a string`);
  const data: any[] = JSON.parse(event.data);
  test.is(
    data.length,
    specs.length,
    `${msg}: length of parsed event.data doesn't match expectation`
  );

  data.map((chunk, i) => {
    // TODO::testing::low: use the timeout
    const { token_id, approval_id, price, timeout } = specs[i];
    const list_id = `${token_id}:${approval_id}:${store.accountId}`;
    const token_key = `${token_id}:${store.accountId}`;
    test.like(
      chunk,
      {
        // TODO::testing::medium: additional fields (except timestamp)
        // FIXME::contracts::medium: price (u128) should always be stringified!
        offer: { id, from: maker.accountId, price: JSON.parse(price) },
        offer_num: id,
        list_id,
        token_key,
      },
      `${msg}: data chunk ${i} doesn't match expectation`
    );
    const chunkTimestamp = chunk.offer.timestamp;
    const chunkTimeout = chunk.offer.timeout;
    test.is(
      chunkTimeout - chunkTimestamp,
      timeout,
      `${msg}: data chunk ${i} has bad timeout`
    );
  });
}
// -------------------------------- balances -------------------------------- //
/**
 * Mostly a wrapper for getting total balance of an account, might change to
 * available balance in the future.
 */
export async function getBalance(account: NearAccount): Promise<ava.NEAR> {
  return (await account.balance()).total;
}

// TODO::testing::low: use this function consistently
/** Asserts balance changes for multiple accounts in parallel */
export async function assertBalanceChanges(
  test: ExecutionContext,
  specs: { account: NearAccount; ref: ava.NEAR; diff: string }[],
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
  { account, ref, diff }: { account: NearAccount; ref: ava.NEAR; diff: string },
  msg: string
) {
  const now = await getBalance(account);
  await assertBalanceDiff(
    test,
    { account: account.accountId, now, old: ref, diff },
    msg
  );
}

// TODO::testing::low: optional `maxGas` param
/**
 * Asserts that the difference between two balances is in a certain range.
 * The assertion is not for equality to account for gas costs. The range is
 * currently hardcoded as `(old + diff - 50 mNEAR) <= now <= (old + diff)`
 */
export async function assertBalanceDiff(
  test: ExecutionContext,
  {
    account,
    now,
    old,
    diff,
  }: { account: string; now: ava.NEAR; old: ava.NEAR; diff: string },
  msg: string
) {
  const maxGas = NEAR(0.04); // allow 50 mNEAR of gas costs

  const max = old.add(new BN(diff));
  const min = max.sub(new BN(maxGas));
  test.log({
    account,
    now: now.toHuman(),
    ref: old.toHuman(),
    diff: diff,
    min: min.toHuman(),
    max: max.toHuman(),
  });
  test.true(now.lte(max), `${msg}: balance too high for ${account}`);
  test.true(now.gte(min), `${msg}: balance too low for ${account}`);
}

// -------------------------------- payouts --------------------------------- //
export function createPayout(spec: [NearAccount, string][]) {
  const payout = {};
  spec.forEach(([account, amount]) => {
    payout[account.accountId] = amount;
  });
  return payout;
}

export function createPayoutPercentage(spec: [NearAccount, number][]) {
  const payout = {};
  spec.forEach(([account, amount]) => {
    payout[account.accountId] = amount;
  });
  return payout;
}

export function createPayoutNumerators(spec: [NearAccount, number][]) {
  const payout = {};
  spec.forEach(([account, amount]) => {
    payout[account.accountId] = { numerator: amount };
  });
  return payout;
}

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
