import { TestConfig } from "../packages/configure"
import { CEP47Events } from "casper-cep47-js-client";
import { EventParser } from "../packages/events"
import { sleep } from "../utils"

import {
  EventStream,
  EventName
} from "casper-js-sdk";

export enum MarketEvents {
  ListingCreated= "market_listing_created",
  ListingPurchased = "market_listing_purchased",
  ListingCanceled = "market_listing_canceled",
  OfferCreated = "market_offer_created",
  OfferWithdraw = "market_offer_withdraw",
  OfferAccepted = "market_offer_accepted"
};


const config = new TestConfig(".env.test.nctl");

const watchEvents = async (eventNames: string[], contractPackageHash: string) => {
  const es = new EventStream(config.eventSteamAddress);

  es.subscribe(EventName.DeployProcessed, (event) => {
    const parsedEvents = EventParser({
      contractPackageHash,
      eventNames
    }, event);

    if (parsedEvents && parsedEvents.success) {
      console.log("*** EVENT ***");
      console.log(parsedEvents.data);
      console.log("*** ***");
    }
  });

  es.start();  
}

const events = async () => {
  await config.setup();

  watchEvents([
        CEP47Events.MintOne,
        CEP47Events.TransferToken,
        CEP47Events.BurnOne,
        CEP47Events.MetadataUpdate,
        CEP47Events.ApproveToken
      ], 
      config.nftContractPackageHash);

  watchEvents([
        MarketEvents.ListingCreated,
        MarketEvents.ListingCanceled,
        MarketEvents.ListingPurchased,
        MarketEvents.OfferCreated,
        MarketEvents.OfferWithdraw
      ], 
      config.marketContractPackageHash);
}

events();