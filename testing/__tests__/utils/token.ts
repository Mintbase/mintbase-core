import { NearAccount } from "near-workspaces";
import { ExecutionContext } from "ava";

/** The current interface of a token as described in NEP171 */
export interface Nep171Token {
  token_id: string;
  owner_id: string;
}

export function isNep171Token(x: any): x is Nep171Token {
  return typeof x.token_id === "string" && typeof x.owner_id === "string";
}

/**
 * Asserts that a token matches an expected token_id/owner_id pairing.
 */
export function assertTokenIs(
  test: ExecutionContext,
  token: Nep171Token,
  { token_id, owner_id }: Nep171Token,
  msg: string
) {
  test.is(`${token.token_id}`, token_id, `${msg}: Wrong token_id`);
  test.is(`${token.owner_id}`, owner_id, `${msg}: Wrong owner_id`);
}

/**
 * Asserts that a token matches an expected token_id/owner_id pairing.
 */
export function assertTokensAre(
  test: ExecutionContext,
  actual: Nep171Token[],
  expected: Nep171Token[],
  msg: string
) {
  // test.log("Actual token list:", actual);
  // test.log("Expected token list:", expected);
  test.is(
    actual.length,
    expected.length,
    `${msg}: token lists mismatched in length`
  );
  expected.forEach((token, i) => {
    assertTokenIs(test, actual[i], token, msg);
  });
}

/**
 * Asserts the contract state matches an expected token_id/owner_id pairing.
 */
export async function assertContractTokenOwner(
  { test, store }: { test: ExecutionContext; store: NearAccount },
  { token_id, owner_id }: Nep171Token,
  msg: string
) {
  const token: Nep171Token = await store.view("nft_token", {
    token_id,
  });
  test.true(
    isNep171Token(token),
    `${msg}: Not a MintbaseToken (token_id: ${token_id})`
  );
  assertTokenIs(test, token, { token_id, owner_id }, msg);
}

/**
 * Asserts that a list of token_id/owner_id pairs match the contract state.
 * Queries all tokens on their own, and thus doesn't suffer from the limitations
 * of the `nft_tokens` method.
 */
export async function assertContractTokenOwners(
  { test, store }: { test: ExecutionContext; store: NearAccount },
  tokens: { token_id: string; owner_id: string }[],
  msg: string
) {
  await Promise.all(
    tokens.map(async (token) => {
      await assertContractTokenOwner({ test, store }, token, msg);
    })
  );
}
