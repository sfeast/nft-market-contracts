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
    system::CallStackElement,
    bytesrepr::ToBytes,
    // bytesrepr::{FromBytes, ToBytes},
    runtime_args, RuntimeArgs,
    ApiError, Key, URef, ContractHash, ContractPackageHash, CLType, CLTyped, U256, U512};
use casper_types_derive::{CLTyped, FromBytes, ToBytes};
// use casper_types::{contracts::NamedKeys};

use event::{emit, MarketEvent};

mod event;

pub const EVENT_TYPE: &str = "event_type";
pub const CONTRACT_PACKAGE_HASH: &str = "contract_package_hash";
pub const SELLER: &str = "seller";
pub const BUYER: &str = "buyer";
pub const TOKEN_CONTRACT: &str = "token_contract";
pub const TOKEN_ID: &str = "token_id";
pub const PRICE: &str = "price";

const CONTRACT_HASH: &str = "market_contract_hash";
const CONTRACT_PACKAGE_HASH_NAME: &str = "market_contract_package_hash";
const OFFERS_PURSE: &str = "offers_purse";

const CREATE_LISTING: &str = "create_listing";
const BUY_LISTING: &str = "buy_listing";
const CANCEL_LISTING: &str = "cancel_listing";
const MAKE_OFFER: &str = "make_offer";
const WITHDRAW_OFFER: &str = "withdraw_offer";
const ACCEPT_OFFER: &str = "accept_offer";

const LISTING_DICTIONARY: &str = "listings";
const OFFER_DICTIONARY: &str = "offers";
const NFT_CONTRACT_HASH_ARG: &str = "token_contract_hash";
const TOKEN_ID_ARG: &str = "token_id";
const PRICE_ARG: &str = "price";
const BUYER_PURSE_ARG: &str = "purse";
const ACCEPTED_OFFER_ARG: &str = "accepted_offer";

/// An error enum which can be converted to a `u16` so it can be returned as an `ApiError::User`.
#[repr(u16)]
enum Error {
    ListingDoesNotExist = 0,
    ListingCanceledOrSold = 1,
    BalanceInsufficient = 2,
    PermissionDenied = 3,
    NoMatchingOffer = 4,
    OfferExists = 5,
    OfferPurseRetrieval = 6
}

impl From<Error> for ApiError {
    fn from(error: Error) -> Self {
        ApiError::User(error as u16)
    }
}

#[derive(CLTyped, ToBytes, FromBytes)]
struct Listing {
    seller: Key,
    token_contract: ContractHash,
    token_id: String,
    price: U512
}

// fn write_named_key_value<T: CLTyped + ToBytes>(name: &str, value: T) -> () {
//     match runtime::get_key(name) {
//         Some(key) => {
//             let key_ref = key.try_into().unwrap_or_revert();
//             storage::write(key_ref, value);
//         }
//         None => {
//             let key = storage::new_uref(value).into();
//             runtime::put_key(name, key);
//         }
//     }
// }

// TODO: what does this do - overkill??
fn contract_package_hash() -> ContractPackageHash {
    let call_stacks = runtime::get_call_stack();
    let last_entry = call_stacks.last().unwrap_or_revert();
    let package_hash: Option<ContractPackageHash> = match last_entry {
        CallStackElement::StoredContract {
            contract_package_hash,
            contract_hash: _,
        } => Some(*contract_package_hash),
        _ => None,
    };
    package_hash.unwrap_or_revert()
}

fn get_id<T: CLTyped + ToBytes>(token_contract: &T, token_id: &T) -> String {
    let mut bytes_a = token_contract.to_bytes().unwrap_or_revert();
    let mut bytes_b = token_id.to_bytes().unwrap_or_revert();

    bytes_a.append(&mut bytes_b);

    let bytes = runtime::blake2b(bytes_a);
    hex::encode(bytes)
}

fn get_dictionary_uref(key: &str) -> URef {
    match runtime::get_key(key) {
        Some(uref_key) => uref_key.into_uref().unwrap_or_revert(),
        None => storage::new_dictionary(key).unwrap_or_revert(),
    }
}

fn get_token_owner(token_contract_hash: ContractHash, token_id: &str) -> Option<Key> {
    runtime::call_contract::<Option<Key>>(
        token_contract_hash,
        "owner_of",
        runtime_args! {
            "token_id" => U256::from_dec_str(&token_id).unwrap()
          }
    )
}

fn token_id_to_vec(token_id: &str) -> Vec<U256> {
    let token_id: U256 = U256::from_str_radix(&token_id, 10).unwrap();
    vec![token_id]
}

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
    
    // TODO: check that token can be transfered by contract, otherwise listing will be in unusable state

    let listing = Listing {
        token_contract: token_contract_hash,
        token_id: token_id.clone(),
        price: price,
        seller: token_owner
    };

    let listing_id: String = get_id(&token_contract_string, &token_id);
    let dictionary_uref = match runtime::get_key(LISTING_DICTIONARY) {
        Some(uref_key) => uref_key.into_uref().unwrap_or_revert(),
        None => storage::new_dictionary(LISTING_DICTIONARY).unwrap_or_revert(),
    };

    storage::dictionary_put(dictionary_uref, &listing_id, listing);

    emit(&MarketEvent::ListingCreated {
        package: contract_package_hash(),
        seller: token_owner,
        token_contract: token_contract_string,
        token_id: token_id,
        price: price
    })
}

fn get_listing(listing_id: &str) -> (Listing, URef) {
    let dictionary_uref = match runtime::get_key(LISTING_DICTIONARY) {
        Some(uref_key) => uref_key.into_uref().unwrap_or_revert(),
        None => storage::new_dictionary(LISTING_DICTIONARY).unwrap_or_revert(),
    };

    // TODO: see correct approach for match on dictionaries below
    let listing = match storage::dictionary_get(dictionary_uref, &listing_id) {
        Ok(value) => value.unwrap_or_revert_with(Error::ListingDoesNotExist),
        Err(_error) => runtime::revert(Error::ListingCanceledOrSold),
    };

    (listing, dictionary_uref)
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

    let dictionary_uref = get_dictionary_uref(LISTING_DICTIONARY);
    storage::dictionary_put(dictionary_uref, &listing_id, None::<Listing>);

    emit(&MarketEvent::ListingCanceled {
        package: contract_package_hash(),
        token_contract: token_contract_string,
        token_id: token_id
    })
}

fn get_offers(offers_id: &str) -> (BTreeMap<Key, U512>, URef) {
    let dictionary_uref = match runtime::get_key(OFFER_DICTIONARY) {
        Some(uref_key) => uref_key.into_uref().unwrap_or_revert(),
        None => storage::new_dictionary(OFFER_DICTIONARY).unwrap_or_revert(),
    };

    let offers: BTreeMap<Key, U512> =
        match storage::dictionary_get(dictionary_uref, &offers_id)  {
            Ok(item) => match item {
                None => BTreeMap::new(),
                Some(offers) => offers,
            },
            Err(_error) => BTreeMap::new()
        };

    return (offers, dictionary_uref);
}

fn get_purse(purse_name: &str) -> URef {
    let purse = if !runtime::has_key(&purse_name) {
        let purse = system::create_purse();
        runtime::put_key(&purse_name, purse.into());
        purse
    } else {
        let destination_purse_key = runtime::get_key(&purse_name).unwrap_or_revert_with(
            Error::OfferPurseRetrieval
        );
        match destination_purse_key.as_uref() {
            Some(uref) => *uref,
            None => runtime::revert(Error::OfferPurseRetrieval),
        }
    };
    return purse;
}

#[no_mangle]
pub extern "C" fn make_offer() -> () {
    let bidder = Key::Account(runtime::get_caller());
    let token_contract_string: String = runtime::get_named_arg(NFT_CONTRACT_HASH_ARG);
    let token_id: String = runtime::get_named_arg(TOKEN_ID_ARG);
    // TEST: will purse transfer fail in payment contract if they don't have enough balance? then no need to worry about this
    // let offer: U512 = runtime::get_named_arg(PRICE_ARG);
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
        price: system::get_purse_balance(offers_purse).unwrap()
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

    if seller != get_token_owner(token_contract_hash, &token_id).unwrap() {
        runtime::revert(Error::PermissionDenied);
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

    cancel_listing(); // do before transfer
  
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

    let mut market_entry_points = EntryPoints::new();
    market_entry_points.add_entry_point(endpoint(CREATE_LISTING));
    market_entry_points.add_entry_point(endpoint(BUY_LISTING));
    market_entry_points.add_entry_point(endpoint(CANCEL_LISTING));
    market_entry_points.add_entry_point(endpoint(MAKE_OFFER));
    market_entry_points.add_entry_point(endpoint(WITHDRAW_OFFER));
    market_entry_points.add_entry_point(endpoint(ACCEPT_OFFER));

    // market_entry_points.add_entry_point(EntryPoint::new(
    //     MAKE_OFFER,
    //     vec![Parameter::new(KEY_NAME, Key::cl_type())],
    //     CLType::Unit,
    //     EntryPointAccess::Public,
    //     EntryPointType::Contract,
    // ));

    // let (stored_contract_hash, _) =
    //     storage::new_locked_contract(market_entry_points, Some(counter_named_keys), None, None);
    // runtime::put_key(COUNTER_KEY, stored_contract_hash.into());

    let (contract_hash, _) =
        storage::add_contract_version(contract_package_hash, market_entry_points, Default::default());
    runtime::put_key(CONTRACT_HASH, contract_hash.into());
    let contract_hash_pack = storage::new_uref(contract_hash);
    runtime::put_key("market_contract_hash_wrapped", contract_hash_pack.into());
    runtime::put_key(CONTRACT_PACKAGE_HASH_NAME, contract_package_hash.into());
}

fn endpoint(name: &str) -> EntryPoint {
    EntryPoint::new(
        String::from(name),
        Vec::new(),
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}