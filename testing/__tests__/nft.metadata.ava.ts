import avaTest from "ava";
import { failPromiseRejection } from "./utils/index.js";
import { setup } from "./setup.js";

const test = setup(avaTest);

test("metadata", async (test) => {
  const { alice, store } = test.context.accounts;
  test.deepEqual(await store.view("nft_metadata"), {
    base_uri: null,
    icon: null,
    name: "alice",
    reference: null,
    reference_hash: null,
    spec: "nft-1.0.0",
    symbol: "ALICE",
  });

  await alice
    .call(
      store,
      "nft_batch_mint",
      {
        owner_id: alice.accountId,
        metadata: {
          title: "Yadda",
          description: "Yadda, yadda!",
          reference: "reference",
          reference_hash: "cmVmZXJlbmNl",
          media: "media",
          media_hash: "bWVkaWE=",
          starts_at: "1672531200000000000",
          expires_at: "1672531200000000000",
          extra: "No more extras for you!",
        },
        num_to_mint: 2,
      },
      { attachedDeposit: "1" }
    )
    .catch(failPromiseRejection(test, "minting"));

  test.deepEqual(await store.view("nft_token_metadata", { token_id: "0" }), {
    copies: 2, // this is automagically inserted because we minted 2 :)
    title: "Yadda",
    description: "Yadda, yadda!",
    reference: "reference",
    reference_hash: "cmVmZXJlbmNl",
    media: "media",
    media_hash: "bWVkaWE=",
    starts_at: "1672531200000000000",
    expires_at: "1672531200000000000",
    extra: "No more extras for you!",
  });

  // TODO::testing::low: deploying with icon/base URI
  // TODO::testing::low: changing icon/base URI
});
