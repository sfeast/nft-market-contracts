use alloc::{
    collections::BTreeMap,
    string::{String, ToString},
};
use casper_contract::contract_api::{runtime, storage};
use casper_types::{ContractPackageHash, URef, Key, U512};

use crate::{EVENT_TYPE, CONTRACT_PACKAGE_HASH, SELLER, BUYER, TOKEN_CONTRACT, TOKEN_ID, PRICE};

pub enum MarketEvent {
    ListingCreated {
        package: ContractPackageHash,
        seller: Key, //Key vs AccountHash so we know what we're getting client side
        token_contract: String,
        token_id: String,
        price: U512
    },
    ListingPurchased {
        package: ContractPackageHash,
        seller: Key,
        buyer: Key,
        token_contract: String,
        token_id: String,
        price: U512
    },
    ListingCanceled {
        package: ContractPackageHash,
        token_contract: String,
        token_id: String
    },
    OfferCreated {
        package: ContractPackageHash,
        buyer: Key,
        token_contract: String,
        token_id: String,
        price: U512
    },
    OfferWithdraw {
        package: ContractPackageHash,
        buyer: Key,
        token_contract: String,
        token_id: String
    },
    OfferAccepted {
        package: ContractPackageHash,
        seller: Key,
        buyer: Key,
        token_contract: String,
        token_id: String,
        price: U512
    }
    // InitContract,
}

pub fn emit(event: &MarketEvent) {
    let push_event = match event {
        MarketEvent::ListingCreated {
            package,
            seller,
            token_contract,
            token_id,
            price
        } => {
            let mut param = BTreeMap::new();
            param.insert(CONTRACT_PACKAGE_HASH, package.to_string());
            param.insert(SELLER, seller.to_string());
            param.insert(TOKEN_CONTRACT, token_contract.to_string());
            param.insert(TOKEN_ID, token_id.to_string());
            param.insert(PRICE, price.to_string());
            param.insert(EVENT_TYPE, "market_listing_created".to_string());
            param
        }
        MarketEvent::ListingPurchased {
            package,
            seller,
            buyer,
            token_contract,
            token_id,
            price
        } => {
            let mut param = BTreeMap::new();
            param.insert(CONTRACT_PACKAGE_HASH, package.to_string());
            param.insert(SELLER, seller.to_string());
            param.insert(BUYER, buyer.to_string());
            param.insert(TOKEN_CONTRACT, token_contract.to_string());
            param.insert(TOKEN_ID, token_id.to_string());
            param.insert(PRICE, price.to_string());
            param.insert(EVENT_TYPE, "market_listing_purchased".to_string());
            param
        }
        MarketEvent::ListingCanceled {
            package,
            token_contract,
            token_id
        } => {
            let mut param = BTreeMap::new();
            param.insert(CONTRACT_PACKAGE_HASH, package.to_string());
            param.insert(TOKEN_CONTRACT, token_contract.to_string());
            param.insert(TOKEN_ID, token_id.to_string());
            param.insert(EVENT_TYPE, "market_listing_canceled".to_string());
            param
        }
        MarketEvent::OfferCreated {
            package,
            buyer,
            token_contract,
            token_id,
            price
        } => {
            let mut param = BTreeMap::new();
            param.insert(CONTRACT_PACKAGE_HASH, package.to_string());
            param.insert(BUYER, buyer.to_string());
            param.insert(TOKEN_CONTRACT, token_contract.to_string());
            param.insert(TOKEN_ID, token_id.to_string());
            param.insert(PRICE, price.to_string());
            param.insert(EVENT_TYPE, "market_offer_created".to_string());
            param
        },
        MarketEvent::OfferWithdraw {
            package,
            buyer,
            token_contract,
            token_id
        } => {
            let mut param = BTreeMap::new();
            param.insert(CONTRACT_PACKAGE_HASH, package.to_string());
            param.insert(BUYER, buyer.to_string());
            param.insert(TOKEN_CONTRACT, token_contract.to_string());
            param.insert(TOKEN_ID, token_id.to_string());
            param.insert(EVENT_TYPE, "market_offer_withdraw".to_string());
            param
        },
        MarketEvent::OfferAccepted {
            package,
            seller,
            buyer,
            token_contract,
            token_id,
            price
        } => {
            let mut param = BTreeMap::new();
            param.insert(CONTRACT_PACKAGE_HASH, package.to_string());
            param.insert(SELLER, seller.to_string());
            param.insert(BUYER, buyer.to_string());
            param.insert(TOKEN_CONTRACT, token_contract.to_string());
            param.insert(TOKEN_ID, token_id.to_string());
            param.insert(PRICE, price.to_string());
            param.insert(EVENT_TYPE, "market_offer_accepted".to_string());
            param
        }
    };
    let latest_event: URef = storage::new_uref(push_event);
    runtime::put_key("latest_event", latest_event.into());
}