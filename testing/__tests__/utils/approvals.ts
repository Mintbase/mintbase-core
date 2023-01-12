import { NearAccount } from "near-workspaces";
import { ExecutionContext } from "ava";

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
