import { sleep } from "../utils";
import { TestConfig } from "../packages/configure"
import { MarketTester } from "../packages/market"
import { NFTTester } from "../packages/nft"

const runTests = async () => {
  const config = new TestConfig(".env.test.nctl");
  await config.setup();
  await sleep(1 * 1000);
  const nftTester = new NFTTester(config);
  const marketTester = new MarketTester(config);

  const userKeys = config.userKeys;
  const token_id = config.token_id;
  const listing_price = config.listing_price;
  const offer_amount = config.offer_amount;

  await nftTester.contractInfo();
  await sleep(1 * 1000);
  await nftTester.mint(userKeys[6], token_id);
  await sleep(1 * 1000);
  await marketTester.saveBalances([userKeys[1], userKeys[2], userKeys[3]]);
  await sleep(1 * 1000);
  await marketTester.makeOffer(userKeys[1], token_id, offer_amount);
  try{await marketTester.makeOffer(userKeys[1], token_id, offer_amount);}catch(e){console.log(e)} // make a 2nd offer
  await sleep(1 * 1000);
  await marketTester.makeOffer(userKeys[2], token_id, offer_amount);
  await sleep(1 * 1000);
  await marketTester.makeOffer(userKeys[3], token_id, offer_amount);
  await sleep(1 * 1000);
  try{await marketTester.withdrawOffer(userKeys[1], token_id);}catch(e){console.log(e)}
  await sleep(1 * 1000);
  await marketTester.makeOffer(userKeys[1], token_id, offer_amount);
  await sleep(1 * 1000);
  try{await marketTester.withdrawOffer(userKeys[2], token_id);}catch(e){console.log(e)}
  await sleep(1 * 1000);
  try{await marketTester.acceptOffer(userKeys[6], userKeys[2], token_id);}catch(e){console.log(e)} // no offer there
  await sleep(1 * 1000);
  try{await marketTester.acceptOffer(userKeys[3], userKeys[1], token_id);}catch(e){console.log(e)} // not owner
  await sleep(1 * 1000);
  try{await marketTester.acceptOffer(userKeys[6], userKeys[1], token_id);}catch(e){console.log(e)} // no transfer approval
  await sleep(1 * 1000);
  await nftTester.approveContractForTransfer(userKeys[6], token_id);
  await sleep(1 * 1000);
  try{await marketTester.acceptOffer(userKeys[6], userKeys[1], token_id);}catch(e){console.log(e)}
  await sleep(1 * 1000);
  
  await marketTester.saveBalances([userKeys[1], userKeys[2], userKeys[3]]);
  await marketTester.reportBalances([userKeys[1], userKeys[2], userKeys[3]]);

  console.log('done!')
};

runTests();