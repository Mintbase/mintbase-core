import { Near, keyStores } from "near-api-js";
import { writeFile } from "fs/promises";

function getNear(network: string): Near {
  if (network === "testnet") {
    return new Near({
      networkId: "testnet",
      keyStore: new keyStores.InMemoryKeyStore(),
      nodeUrl: "https://rpc.testnet.near.org",
      walletUrl: "https://wallet.testnet.near.org",
      helperUrl: "https://helper.testnet.near.org",
      headers: {},
      // explorerUrl: "https://explorer.testnet.near.org",
    });
  }
  return new Near({
    networkId: "mainnet",
    keyStore: new keyStores.InMemoryKeyStore(),
    nodeUrl: "https://rpc.mainnet.near.org",
    // archivalUrl: "https://archival-rpc.mainnet.near.org",
    walletUrl: "https://wallet.mainnet.near.org",
    helperUrl: "https://helper.mainnet.near.org",
    headers: {},
    // explorerUrl: "https://explorer.mainnet.near.org",
  });
}

function base64ToBytes(strb64: string): Uint8Array {
  return Buffer.from(strb64, "base64");
}

async function downloadContract(
  what: string,
  network: string,
  account: string
) {
  const near = getNear(network);
  const { code_base64 } = (await near.connection.provider.query({
    account_id: account,
    finality: "final",
    request_type: "view_code",
  })) as any as { code_base64: string };

  // parse to raw bytes
  const bytes = base64ToBytes(code_base64);

  // write to file
  await writeFile(`./downloads/${network}-${what}.wasm`, bytes);
}

export async function downloadContracts() {
  await downloadContract("store", "testnet", "whatever123.mintspace2.testnet");
  await downloadContract("store", "mainnet", "mintbase.mintbase1.near");
  await downloadContract("factory", "testnet", "mintspace2.testnet");
  await downloadContract("factory", "mainnet", "mintbase1.near");
  await downloadContract("market", "testnet", "market.mintspace2.testnet");
  await downloadContract("market", "mainnet", "market.mintbase1.near");
}
