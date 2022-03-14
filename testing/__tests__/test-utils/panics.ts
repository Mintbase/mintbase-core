import { ExecutionContext } from "ava";

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
    // // log full error message in case anything goes wrong
    // test.log(error.kind.ExecutionError);

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
