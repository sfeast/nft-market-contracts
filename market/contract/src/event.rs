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
    // GuestVictory {
    //     host: String,
    //     guest: String,
    //     host_score: U512,
    //     guest_score: U512,
    // },
    // HostMove {
    //     host: String,
    //     guest: String,
    //     player_move: usize,
    // },
    // GuestMove {
    //     host: String,
    //     guest: String,
    //     player_move: usize,
    // },
    // Draw {
    //     host: String,
    //     guest: String,
    // },
    // Reset {
    //     host: String,
    //     guest: String,
    // },
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
        // T3Event::GameStart {
        //     host,
        //     guest,
        //     player_move,
        // } => {
        //     let mut param = BTreeMap::new();
        //     param.insert(HOST, host.to_string());
        //     param.insert(GUEST, guest.to_string());
        //     param.insert(PLAYER_MOVE, player_move.to_string());
        //     param.insert(EVENT_TYPE, "GameStart".to_string());
        //     param
        // }
        // T3Event::HostMove {
        //     host,
        //     guest,
        //     player_move,
        // } => {
        //     let mut param = BTreeMap::new();
        //     param.insert(HOST, host.to_string());
        //     param.insert(GUEST, guest.to_string());
        //     param.insert(PLAYER_MOVE, player_move.to_string());
        //     param.insert(EVENT_TYPE, "HostMove".to_string());
        //     param
        // }
        // T3Event::GuestMove {
        //     host,
        //     guest,
        //     player_move,
        // } => {
        //     let mut param = BTreeMap::new();
        //     param.insert(HOST, host.to_string());
        //     param.insert(GUEST, guest.to_string());
        //     param.insert(PLAYER_MOVE, player_move.to_string());
        //     param.insert(EVENT_TYPE, "GuestMove".to_string());
        //     param
        // }
        // T3Event::Draw { host, guest } => {
        //     let mut param = BTreeMap::new();
        //     param.insert(HOST, host.to_string());
        //     param.insert(GUEST, guest.to_string());
        //     param.insert(EVENT_TYPE, "Draw".to_string());
        //     param
        // }
        // T3Event::Reset { host, guest } => {
        //     let mut param = BTreeMap::new();
        //     param.insert(HOST, host.to_string());
        //     param.insert(GUEST, guest.to_string());
        //     param.insert(EVENT_TYPE, "Reset".to_string());
        //     param
        // }
        // T3Event::InitContract => {
        //     let mut param = BTreeMap::new();
        //     param.insert(EVENT_TYPE, "InitContract".to_string());
        //     param
        // }
    };
    let latest_event: URef = storage::new_uref(push_event);
    runtime::put_key("latest_event", latest_event.into());
}