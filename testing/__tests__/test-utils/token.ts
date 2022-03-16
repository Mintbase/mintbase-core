import { NearAccount } from "near-workspaces-ava";
import { ExecutionContext } from "ava";

/** The current interface of a token on the store contract */
export interface MintbaseToken {
  // FIXME::store::high: this should  be a string, not number
  id: number;
  // owner_id: { Account: string };
  owner_id: string;
}

/** Typescript narrowing function for `MintbaseToken` */
export function isMintbaseToken(x: any): x is MintbaseToken {
  return typeof x.id === "number" && typeof x.owner_id === "string";
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
  test.is(`${token.owner_id}`, owner_id, `${msg}: Wrong owner_id`);
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
