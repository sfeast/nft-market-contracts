# Contracts
There are 2 contracts included:
- market: this includes nft market functionality: list, cancel, buy + offer, withdraw, acceptOffer
- payment: this is a small contract that is to be installed on the user's end as a mechanism to transfer payment to your contract. This is for security purposes.

To use the market contract you must install it & then make deployments to it's entry points either from a client or contract. It works with the standard cep47 contract implemented by the Casper team here https://github.com/casper-ecosystem/casper-nft-cep47 so it expects that the cep47 contract it interacts with will have the various cep47 entry points following the cep47 standard.

See the Tests section below for easy installation & testing.

# Tests

Contains unit tests based on the casper-contracts-js-clients project. The cep47 JS tests were turned into class methods & similar was done for the market & payment contracts in this repo. These classes can now be used to run sequential calls against your contracts as well as query their data.

### Setup

Before running the examples, you should copy the `.env.example.nctl` file, rename as "env.test.nctl" & input all the environment variables. Note that this also expects you have setup NCTL properly on your development local machine https://docs.casperlabs.io/dapp-dev-guide/setup-nctl/

Next before running any tests you'll need to install the contracts (market & cep47). You can do that by running the install scripts like so:
`ts-node ./e2e/cep47/install.ts`
`ts-node ./e2e/market/install.ts`

Note the payment contract does not need to be installed, it is part of the market tests & handled by the testing class.

## Usage examples

Full examples of the tests are in /e2e/tests, but as a quick example they look something like:

```
  await nftTester.mint(userKeys[1], token_id);
  await sleep(1 * 1000);
  await nftTester.approveContractForTransfer(userKeys[1], token_id);
  await sleep(1 * 1000);
  try{await marketTester.buyListing(userKeys[6], token_id, listing_price, userKeys[1]);}catch(e){console.log(e)}
```
where the last line is wrapped in try/catch since we know it will error as the item wasn't listed before trying to buy it. Wrapping it allows more tests to be placed after it.

You can run as many tests sequentially as you'd like, using loops, control statements, etc. Currently there are no examples of querying data from the test files but you can also add that there or modify the test classes.

## How to run

To run an example I am currently just running the files directly, e.g.

`ts-node ./e2e/tests/listing.ts`

Although I prefer to use

`node --inspect -r ts-node/register  ./e2e/market/install.ts`

which lets you open a debugger window from chrome://inspect (go to that URL in Chrome & choose the localhost remote target once you've run the node command above)
