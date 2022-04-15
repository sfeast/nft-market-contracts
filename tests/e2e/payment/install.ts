import { config } from "dotenv";
config({ path: ".env.test.nctl" });
import { PaymentClient } from "./payment_client";
import { getDeploy, getAccountInfo, getAccountNamedKeyValue } from "../utils";
import { utils } from "casper-js-client-helper";
import * as fs from "fs";

import {
  Keys
} from "casper-js-sdk";

const {
  NODE_ADDRESS,
  CHAIN_NAME,
  PAYMENT_WASM_PATH,
  USER_9_KEY_PAIR_PATH,
  MARKET_MASTER_KEY_PAIR_PATH,
  NFT_MASTER_KEY_PAIR_PATH,
  MARKET_CONTRACT_NAME,
  NFT_CONTRACT_NAME,
  NFT_TOKEN_ID,
  INSTALL_PAYMENT_AMOUNT
} = process.env;

const KEYS = Keys.Ed25519.parseKeyFiles(
  `${USER_9_KEY_PAIR_PATH}/public_key.pem`,
  `${USER_9_KEY_PAIR_PATH}/secret_key.pem`
);

const getMarketContractHash = async () => {
  const MARKET_KEYS = Keys.Ed25519.parseKeyFiles(
    `${MARKET_MASTER_KEY_PAIR_PATH}/public_key.pem`,
    `${MARKET_MASTER_KEY_PAIR_PATH}/secret_key.pem`
  );

  let accountInfo = await getAccountInfo(NODE_ADDRESS, MARKET_KEYS.publicKey);

  console.log(`... Account Info: `);
  console.log(JSON.stringify(accountInfo, null, 2));
  
  const contractHash = await getAccountNamedKeyValue(
    accountInfo,
    `${MARKET_CONTRACT_NAME!}_contract` //TODO should we change in contract to _contract_hash ?
  );

  console.log(`... Market Contract Hash: ${contractHash}`);

  // was trying to check named keys here but named keys get stored to contract in call() it looks like
  // but not entry points...
  // const stateRootHash = await utils.getStateRootHash(NODE_ADDRESS!);
  // const bidder =  await utils.getContractData(NODE_ADDRESS!, stateRootHash, contractHash.replace('hash-', ''), ['bidder']);
  // console.log('bidder = ', bidder);
  // console.log('-------------------------------------------------');

  return contractHash;
}


const getNftContractHash = async () => {
  const NFT_KEYS = Keys.Ed25519.parseKeyFiles(
    `${NFT_MASTER_KEY_PAIR_PATH}/public_key.pem`,
    `${NFT_MASTER_KEY_PAIR_PATH}/secret_key.pem`
  );

  let accountInfo = await getAccountInfo(NODE_ADDRESS, NFT_KEYS.publicKey);

  console.log(`... Account Info: `);
  console.log(JSON.stringify(accountInfo, null, 2));
  
  const contractHash = await getAccountNamedKeyValue(
    accountInfo,
    `${NFT_CONTRACT_NAME!}_contract_hash`
  );

  console.log(`... Market Contract Hash: ${contractHash}`);

  return contractHash;
}

const test = async () => {
  const payment = new PaymentClient(
    NODE_ADDRESS!,
    CHAIN_NAME!
  );

  const marketContractHash = await getMarketContractHash();

  const installDeployHash = await payment.install(
    PAYMENT_WASM_PATH!,
    {
      market_contract_hash: marketContractHash.replace('hash', 'contract'),
      entry_point_name: 'buy_listing',
      token_contract_hash: (await getNftContractHash()).replace('hash', 'contract'),
      token_id: NFT_TOKEN_ID!,
      amount: 100
    },
    INSTALL_PAYMENT_AMOUNT!,
    KEYS.publicKey,
    [KEYS],
  );

  const hash = await installDeployHash.send(NODE_ADDRESS!);

  console.log(`... Contract installation deployHash: ${hash}`);

  await getDeploy(NODE_ADDRESS!, hash);

  console.log(`... Contract installed successfully.`);

  // console.log('-------------------------------------------------');
  const stateRootHash = await utils.getStateRootHash(NODE_ADDRESS!);
  const bidder =  await utils.getContractData(NODE_ADDRESS!, stateRootHash, marketContractHash.replace('hash-', ''), ['bidder']);
  console.log('bidder = ', bidder);
  // console.log('-------------------------------------------------');

};

test();

