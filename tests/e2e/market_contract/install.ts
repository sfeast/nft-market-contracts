import { config } from "dotenv";
config({ path: ".env.test.nctl" });
// config({ path: ".env.test.casper-test" });
import { getDeploy, getAccountInfo, getAccountNamedKeyValue } from "../utils";
import * as fs from "fs";

import {
  RuntimeArgs,
  Contracts,
  CasperClient,
  DeployUtil,
  CLValueBuilder,
  Keys,
  CLPublicKey,
  CLPublicKeyType,
} from "casper-js-sdk";

const {
  NODE_ADDRESS,
  EVENT_STREAM_ADDRESS,
  CHAIN_NAME,
  MARKET_WASM_PATH,
  MARKET_MASTER_KEY_PAIR_PATH,
  MARKET_CONTRACT_NAME,
  MARKET_INSTALL_PAYMENT_AMOUNT
} = process.env;

export const getBinary = (pathToBinary: string) => {
  return new Uint8Array(fs.readFileSync(pathToBinary, null).buffer);
};

const KEYS = Keys.Ed25519.parseKeyFiles(
  `${MARKET_MASTER_KEY_PAIR_PATH}/public_key.pem`,
  `${MARKET_MASTER_KEY_PAIR_PATH}/secret_key.pem`
);

const test = async () => {
  const client = new CasperClient(NODE_ADDRESS!);
  const contract = new Contracts.Contract(client);

  const runtimeArgs = RuntimeArgs.fromMap({
    contract_name: CLValueBuilder.string(MARKET_CONTRACT_NAME!)
  });

  const installDeployHash = await contract.install(getBinary(MARKET_WASM_PATH!), runtimeArgs, MARKET_INSTALL_PAYMENT_AMOUNT!, KEYS.publicKey, CHAIN_NAME!, [KEYS]);

  const hash = await installDeployHash.send(NODE_ADDRESS!);

  console.log(`... Contract installation deployHash: ${hash}`);

  await getDeploy(NODE_ADDRESS!, hash);

  console.log(`... Contract installed successfully.`);

  let accountInfo = await getAccountInfo(NODE_ADDRESS, KEYS.publicKey);

  console.log(`... Account Info: `);
  console.log(JSON.stringify(accountInfo, null, 2));

  const contractHash = await getAccountNamedKeyValue(
    accountInfo,
    `${MARKET_CONTRACT_NAME!}_contract_hash`
  );

  const contractPackageHash = await getAccountNamedKeyValue(
    accountInfo,
    `${MARKET_CONTRACT_NAME}_contract_package_hash`
  );


  console.log(`... Contract Hash: ${contractHash}`);
  console.log(`... Contract Package Hash: ${contractPackageHash}`);
};

test();

