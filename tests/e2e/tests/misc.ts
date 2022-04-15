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

  await nftTester.contractInfo();
  await sleep(1 * 1000);
  await nftTester.mint(userKeys[1], token_id);
  await sleep(1 * 1000);
  try{await marketTester.listForSale(userKeys[1], token_id, listing_price);}catch(e){console.log(e)}
  await sleep(1 * 1000);
  await nftTester.approveContractForTransfer(userKeys[1], token_id);
  await sleep(1 * 1000);
  await marketTester.listForSale(userKeys[1], token_id, listing_price);
  await sleep(1 * 1000);
  try{await marketTester.buyListing(userKeys[6], token_id, listing_price, userKeys[1]);}catch(e){console.log(e)}

  console.log('done!')
};

runTests();