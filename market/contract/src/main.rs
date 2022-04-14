#![no_std]
#![no_main]

#[cfg(not(target_arch = "wasm32"))]
compile_error!("target arch should be wasm32: compile with '--target wasm32-unknown-unknown'");

// We need to explicitly import the std alloc crate and `alloc::string::String` as we're in a
// `no_std` environment.
extern crate alloc;

use alloc::{
    string::String,
    str,
    vec, vec::Vec,
    collections::BTreeMap
};

use casper_contract::{
    contract_api::{runtime, storage, system},
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{
    contracts::{EntryPoint, EntryPointAccess, EntryPointType, EntryPoints},
    runtime_args, RuntimeArgs, Parameter,
    Key, URef, ContractHash, CLTyped, U256, U512};
// use casper_types::{contracts::NamedKeys};

use event::{MarketEvent};
mod event;
use data::{
            Error, Listing, contract_package_hash, transfer_approved, get_id,
            get_token_owner, token_id_to_vec, get_listing,
            get_listing_dictionary, get_offers, get_purse, emit, force_cancel_listing};
mod data;

const OFFERS_PURSE: &str = "offers_purse";

const NFT_CONTRACT_HASH_ARG: &str = "token_contract_hash";
const TOKEN_ID_ARG: &str = "token_id";
const PRICE_ARG: &str = "price";
const BUYER_PURSE_ARG: &str = "purse";
const ACCEPTED_OFFER_ARG: &str = "accepted_offer";

#[no_mangle]
pub extern "C" fn create_listing() -> () {
    let token_owner = Key::Account(runtime::get_caller());
    let token_contract_string: String = runtime::get_named_arg(NFT_CONTRACT_HASH_ARG);
    let token_contract_hash: ContractHash = ContractHash::from_formatted_str(&token_contract_string).unwrap();
    let token_id: String = runtime::get_named_arg(TOKEN_ID_ARG);
    let price: U512 = runtime::get_named_arg(PRICE_ARG);

    if token_owner != get_token_owner(token_contract_hash, &token_id).unwrap() {
        runtime::revert(Error::PermissionDenied);
    }
    
    if !transfer_approved(token_contract_hash, &token_id, token_owner) {
        runtime::revert(Error::NeedsTransferApproval);
    }

    let listing = Listing {
        token_contract: token_contract_hash,
        token_id: token_id.clone(),
        price: price,
        seller: token_owner
    };

    let listing_id: String = get_id(&token_contract_string, &token_id);
    let dictionary_uref: URef = get_listing_dictionary();
    storage::dictionary_put(dictionary_uref, &listing_id, listing);

    emit(&MarketEvent::ListingCreated {
        package: contract_package_hash(),
        seller: token_owner,
        token_contract: token_contract_string,
        token_id: token_id,
        price: price
    })
}

#[no_mangle]
pub fn buy_listing() -> () {
    let buyer = Key::Account(runtime::get_caller());
    let token_contract_string: String = runtime::get_named_arg(NFT_CONTRACT_HASH_ARG);
    let token_contract_hash: ContractHash = ContractHash::from_formatted_str(&token_contract_string).unwrap();
    let token_id: String = runtime::get_named_arg(TOKEN_ID_ARG);
    let token_ids: Vec<U256> = token_id_to_vec(&token_id);
    let listing_id: String = get_id(&token_contract_string, &token_id);
    let (listing, dictionary_uref) = get_listing(&listing_id);
    let buyer_purse: URef = runtime::get_named_arg(BUYER_PURSE_ARG);
    let purse_balance: U512 = system::get_purse_balance(buyer_purse).unwrap();

    if purse_balance < listing.price {
        runtime::revert(Error::BalanceInsufficient);
    }

    let seller = get_token_owner(token_contract_hash, &token_id).unwrap();

    system::transfer_from_purse_to_account(
        buyer_purse,
        seller.into_account().unwrap_or_revert(),
        listing.price,
        None
    ).unwrap_or_revert();

    runtime::call_contract::<()>(
        token_contract_hash,
        "transfer_from",
        runtime_args! {
            "sender" => seller,
            "recipient" => buyer,
            "token_ids" => token_ids,
          }
    );

    storage::dictionary_put(dictionary_uref, &listing_id, None::<Listing>);

    emit(&MarketEvent::ListingPurchased {
        package: contract_package_hash(),
        seller: seller,
        buyer: buyer,
        token_contract: token_contract_string,
        token_id: token_id,
        price: listing.price
    })
}

#[no_mangle]
pub fn cancel_listing() -> () {
    let caller = Key::Account(runtime::get_caller());
    let token_contract_string: String = runtime::get_named_arg(NFT_CONTRACT_HASH_ARG);
    let token_contract_hash: ContractHash = ContractHash::from_formatted_str(&token_contract_string).unwrap();
    let token_id: String = runtime::get_named_arg(TOKEN_ID_ARG);
    let listing_id: String = get_id(&token_contract_string, &token_id);
    let seller = get_token_owner(token_contract_hash, &token_id).unwrap();

    if caller != seller {
        runtime::revert(Error::PermissionDenied);
    }

    let (_listing, dictionary_uref) = get_listing(&listing_id);
    storage::dictionary_put(dictionary_uref, &listing_id, None::<Listing>);

    emit(&MarketEvent::ListingCanceled {
        package: contract_package_hash(),
        token_contract: token_contract_string,
        token_id: token_id
    })
}

#[no_mangle]
pub extern "C" fn make_offer() -> () {
    let bidder = Key::Account(runtime::get_caller());
    let token_contract_string: String = runtime::get_named_arg(NFT_CONTRACT_HASH_ARG);
    let token_id: String = runtime::get_named_arg(TOKEN_ID_ARG);
    let offers_id: String = get_id(&token_contract_string, &token_id);
    let bidder_purse: URef = runtime::get_named_arg(BUYER_PURSE_ARG);
    let purse_balance: U512 = system::get_purse_balance(bidder_purse).unwrap();

    let (mut offers, dictionary_uref): (BTreeMap<Key, U512>, URef) = get_offers(&offers_id);
    
    let offers_purse = get_purse(OFFERS_PURSE);

    // TODO: increase current offer instead of error
    match offers.get(&bidder) {
        Some(_) => runtime::revert(Error::OfferExists),
        None => ()
    }

    offers.insert(bidder, purse_balance);
    system::transfer_from_purse_to_purse(bidder_purse, offers_purse, purse_balance, None).unwrap_or_revert();
    storage::dictionary_put(dictionary_uref, &offers_id, offers);

    emit(&MarketEvent::OfferCreated {
        package: contract_package_hash(),
        buyer: bidder,
        token_contract: token_contract_string,
        token_id: token_id,
        price: purse_balance
    })
}

#[no_mangle]
pub extern "C" fn withdraw_offer() -> () {
    let bidder = Key::Account(runtime::get_caller());
    let token_contract_string: String = runtime::get_named_arg(NFT_CONTRACT_HASH_ARG);
    let token_id: String = runtime::get_named_arg(TOKEN_ID_ARG);

    let offers_id: String = get_id(&token_contract_string, &token_id);

    let (mut offers, dictionary_uref):
        (BTreeMap<Key, U512>, URef) = get_offers(&offers_id);

    let amount: U512 = offers.get(&bidder)
        .unwrap_or_revert_with(Error::NoMatchingOffer)
        .clone();

    let offers_purse = get_purse(OFFERS_PURSE);

    system::transfer_from_purse_to_account(
        offers_purse,
        bidder.into_account().unwrap_or_revert(),
        amount.clone(),
        None
    ).unwrap_or_revert();

    offers.remove(&bidder);
    storage::dictionary_put(dictionary_uref, &offers_id, offers);

    emit(&MarketEvent::OfferWithdraw {
        package: contract_package_hash(),
        buyer: bidder,
        token_contract: token_contract_string,
        token_id: token_id
    })
}

#[no_mangle]
pub extern "C" fn accept_offer() -> () {
    let seller = Key::Account(runtime::get_caller());
    let token_contract_string: String = runtime::get_named_arg(NFT_CONTRACT_HASH_ARG);
    let token_contract_hash: ContractHash = ContractHash::from_formatted_str(&token_contract_string).unwrap();
    let token_id: String = runtime::get_named_arg(TOKEN_ID_ARG);
    let token_ids: Vec<U256> = token_id_to_vec(&token_id);
    let offer_account_hash: String = runtime::get_named_arg(ACCEPTED_OFFER_ARG);
    let accepted_bidder_hash: Key = Key::from_formatted_str(&offer_account_hash).unwrap();
    let offers_id: String = get_id(&token_contract_string, &token_id);
    let offers_purse = get_purse(OFFERS_PURSE);

    // TODO: remove these 2 checks after adjusting error codes around cep47 errors
    if seller != get_token_owner(token_contract_hash, &token_id).unwrap() {
        runtime::revert(Error::PermissionDenied);
    }
    
    if !transfer_approved(token_contract_hash, &token_id, seller) {
        runtime::revert(Error::NeedsTransferApproval);
    }

    let (mut offers, dictionary_uref):
        (BTreeMap<Key, U512>, URef) = get_offers(&offers_id);

    let amount: U512 = offers.get(&accepted_bidder_hash)
        .unwrap_or_revert_with(Error::NoMatchingOffer)
        .clone();    

    system::transfer_from_purse_to_account(
        offers_purse,
        seller.into_account().unwrap_or_revert(),
        amount.clone(),
        None
    ).unwrap_or_revert();
    offers.remove(&accepted_bidder_hash);
  
    runtime::call_contract::<()>(
        token_contract_hash,
        "transfer_from",
        runtime_args! {
            "sender" => seller,
            "recipient" => accepted_bidder_hash,
            "token_ids" => token_ids,
          }
    );

    // refund the other offers
    for (account, bid) in &offers {
        system::transfer_from_purse_to_account(
            offers_purse,
            account.into_account().unwrap_or_revert(),
            bid.clone(),
            None
        ).unwrap_or_revert();
    }

    offers.clear();
    force_cancel_listing(&token_contract_string, &token_id);
    storage::dictionary_put(dictionary_uref, &offers_id, offers);

    emit(&MarketEvent::OfferAccepted {
        package: contract_package_hash(),
        seller: seller,
        buyer: accepted_bidder_hash,
        token_contract: token_contract_string,
        token_id: token_id,
        price: amount
    })
}

#[no_mangle]
pub extern "C" fn call() {
    let (contract_package_hash, _) = storage::create_contract_package_at_hash();
    let (contract_hash, _) =
        storage::add_contract_version(contract_package_hash, get_entry_points(), Default::default());
    runtime::put_key("market_contract_hash", contract_hash.into());
    let contract_hash_pack = storage::new_uref(contract_hash);
    runtime::put_key("market_contract_hash_wrapped", contract_hash_pack.into());
    runtime::put_key("market_contract_package_hash", contract_package_hash.into());
}

fn get_entry_points() -> EntryPoints {
    let mut entry_points = EntryPoints::new();

    entry_points.add_entry_point(EntryPoint::new(
        "create_listing",
        vec![
            Parameter::new(NFT_CONTRACT_HASH_ARG, String::cl_type()),
            Parameter::new(TOKEN_ID_ARG, String::cl_type()),
            Parameter::new(PRICE_ARG, U256::cl_type())
        ],
        <()>::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));
    entry_points.add_entry_point(EntryPoint::new(
        "buy_listing",
        vec![
            Parameter::new(NFT_CONTRACT_HASH_ARG, String::cl_type()),
            Parameter::new(TOKEN_ID_ARG, String::cl_type()),
            Parameter::new(BUYER_PURSE_ARG, URef::cl_type())
        ],
        <()>::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));
    entry_points.add_entry_point(EntryPoint::new(
        "cancel_listing",
        vec![
            Parameter::new(NFT_CONTRACT_HASH_ARG, String::cl_type()),
            Parameter::new(TOKEN_ID_ARG, String::cl_type())
        ],
        <()>::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));
    entry_points.add_entry_point(EntryPoint::new(
        "make_offer",
        vec![
            Parameter::new(NFT_CONTRACT_HASH_ARG, String::cl_type()),
            Parameter::new(TOKEN_ID_ARG, String::cl_type()),
            Parameter::new(BUYER_PURSE_ARG, URef::cl_type())
        ],
        <()>::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));
    entry_points.add_entry_point(EntryPoint::new(
        "withdraw_offer",
        vec![
            Parameter::new(NFT_CONTRACT_HASH_ARG, String::cl_type()),
            Parameter::new(TOKEN_ID_ARG, String::cl_type())
        ],
        <()>::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));
    entry_points.add_entry_point(EntryPoint::new(
        "accept_offer",
        vec![
            Parameter::new(NFT_CONTRACT_HASH_ARG, String::cl_type()),
            Parameter::new(TOKEN_ID_ARG, String::cl_type()),
            Parameter::new(ACCEPTED_OFFER_ARG, URef::cl_type())
        ],
        <()>::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));
    entry_points
}
