import { NearAccount } from "near-workspaces-ava";
import { ExecutionContext } from "ava";

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
  // test.log("Expected:", expected);
  test.deepEqual(event, expected, baseMsg);
}

function parseEvent(test: ExecutionContext, log: string, msg: string) {
  // FIXME::contracts::medium: standard has no space between colon and JSON
  test.is(log.slice(0, 11), "EVENT_JSON:", `${msg}: Not an event log`);
  // test.log("Sliced:", log.slice(12));
  const event = JSON.parse(log.slice(11).trimStart());
  // test.log("Parsed:", event);
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
