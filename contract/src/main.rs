#![no_std]
#![no_main]

#[cfg(not(target_arch = "wasm32"))]
compile_error!("target arch should be wasm32: compile with '--target wasm32-unknown-unknown'");

// We need to explicitly import the std alloc crate and `alloc::string::String` as we're in a
// `no_std` environment.
extern crate alloc;

use alloc::{
    string::{String, ToString},
    str,
    format,
    vec, vec::Vec,
    collections::BTreeMap
};

use core::convert::TryInto;

use casper_contract::{
    contract_api::{runtime, storage},
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{
    contracts::{EntryPoint, EntryPointAccess, EntryPointType, EntryPoints},
    system::CallStackElement,
    bytesrepr::ToBytes,
    ApiError, Key, CLType, CLTyped, Parameter, U512};
use casper_types_derive::{CLTyped, FromBytes, ToBytes};

// use casper_types::{ApiError, contracts::NamedKeys, U512, Key, ContractHash, URef, CLTyped, bytesrepr::FromBytes, runtime_args, RuntimeArgs, system::CallStackElement};

const MAKE_OFFER: &str = "make_offer";
const CREATE_LISTING: &str = "create_listing";
const FIND_PRICE: &str = "find_price";

const KEY_NAME: &str = "bidder";
const KEY_PRICE: &str = "price";

const LISTING_DICTIONARY: &str = "listing_id_dictionary"; //TODO: rename to listings?
const NFT_CONTRACT_HASH_ARG: &str = "token_contract_hash";
const TOKEN_ID_ARG: &str = "token_id";
const PRICE_ARG: &str = "price";


const ERROR_INVALID_CALLER: u16 = 1;


/// An error enum which can be converted to a `u16` so it can be returned as an `ApiError::User`.
#[repr(u16)]
enum Error {
    KeyAlreadyExists = 0,
    KeyMismatch = 1,
}

impl From<Error> for ApiError {
    fn from(error: Error) -> Self {
        ApiError::User(error as u16)
    }
}

#[derive(CLTyped, ToBytes, FromBytes)]
struct Listing {
    seller: Key,
    token_contract: Key,
    token_id: String,
    price: U512
}

fn get_id<T: CLTyped + ToBytes>(token_contract: &T, token_id: &T) -> String {
    let mut bytes_a = token_contract.to_bytes().unwrap_or_revert();
    let mut bytes_b = token_id.to_bytes().unwrap_or_revert();

    bytes_a.append(&mut bytes_b);

    let bytes = runtime::blake2b(bytes_a);
    hex::encode(bytes)
}

#[no_mangle]
pub extern "C" fn create_listing() -> () {
    let token_owner = Key::Account(runtime::get_caller());
    // TODO: check that it actually is the token owner - otherwise anyone can list someones token for any price

    // let token_contract_hash: Key = Key::Hash(runtime::get_named_arg::<Key>(NFT_CONTRACT_HASH_ARG).into_hash().unwrap_or_revert());
    let token_contract_string: String = runtime::get_named_arg(NFT_CONTRACT_HASH_ARG);
    let token_contract_hash: Key = Key::from_formatted_str(&token_contract_string).unwrap();
    let token_id: String = runtime::get_named_arg(TOKEN_ID_ARG);
    let price: U512 = runtime::get_named_arg(PRICE_ARG);

    let listing_id: String = get_id(&token_contract_string, &token_id);

    let listing = Listing {
        token_contract: token_contract_hash,
        token_id: token_id,
        price: price,
        seller: token_owner
    };

    // The key shouldn't already exist in the named keys.
    // let missing_key = runtime::get_key(KEY_NAME);
    // if missing_key.is_some() {
    //     runtime::revert(Error::KeyAlreadyExists);
    // }

    let mut bids: BTreeMap<i32, (&str, i32)> = BTreeMap::new();
    bids.insert(134,("num1", 10));
    bids.insert(256,("num2", 27));
    bids.insert(789,("num1", 100));

    let dictionary_uref = match runtime::get_key(LISTING_DICTIONARY) {
        Some(uref_key) => uref_key.into_uref().unwrap_or_revert(),
        None => storage::new_dictionary(LISTING_DICTIONARY).unwrap_or_revert(),
    };
    // let score: U512 = runtime::get_named_arg("score");
    // if score
    //     > storage::dictionary_get::<U512>(dictionary_uref, &get_caller().to_string())
    //         .unwrap_or_revert()
    //         .unwrap_or_default()
    // {
        storage::dictionary_put(dictionary_uref, &listing_id, listing);//(10,"test", bids));
    // }
}

fn get_bidder() -> Key {
    // Figure out who is trying to bid and what their bid is
    let mut call_stack = runtime::get_call_stack();
    call_stack.pop();

    //if session { () } else { call_stack.pop(); () };

    let caller: CallStackElement = call_stack.last().unwrap_or_revert().clone();
    // TODO: Contracts should probably be disallowed, since they can't be verified by Civic in a meaningful way
    let bidder = match caller {
        CallStackElement::Session { account_hash: account_hash_caller} => Key::Account(account_hash_caller),
        CallStackElement::StoredContract { contract_package_hash: _, contract_hash: contract_hash_addr_caller} => Key::Hash(contract_hash_addr_caller.value()),
        _ => runtime::revert(ApiError::User(ERROR_INVALID_CALLER)),
    };

    return bidder;
}


#[no_mangle]
pub extern "C" fn find_price() -> () {
    let token_contract_string: String = runtime::get_named_arg(NFT_CONTRACT_HASH_ARG);
    let token_id: String = runtime::get_named_arg(TOKEN_ID_ARG);

    let listing_id: String = get_id(&token_contract_string, &token_id);
    let dictionary_uref = match runtime::get_key(LISTING_DICTIONARY) {
        Some(uref_key) => uref_key.into_uref().unwrap_or_revert(),
        None => storage::new_dictionary(LISTING_DICTIONARY).unwrap_or_revert(),
    };

    let listing: Listing = storage::dictionary_get(dictionary_uref, &listing_id)
        .unwrap()
        .unwrap();

    match runtime::get_key(KEY_PRICE) {
        Some(key) => {
            let key_ref = key.try_into().unwrap_or_revert();
            storage::write(key_ref, listing.price);
        }
        None => {
            let key = storage::new_uref(listing.price).into();
            runtime::put_key(KEY_PRICE, key);
        }
    }
}

#[no_mangle]
pub extern "C" fn make_offer() -> () {
    let bidder= get_bidder().into_account().unwrap_or_revert_with(ApiError::User(ERROR_INVALID_CALLER));

    // The key shouldn't already exist in the named keys.
    // let missing_key = runtime::get_key(KEY_NAME);
    // if missing_key.is_some() {
    //     runtime::revert(Error::KeyAlreadyExists);
    // }

    match runtime::get_key(KEY_NAME) {
        Some(key) => {
            let key_ref = key.try_into().unwrap_or_revert();
            storage::write(key_ref, bidder);
        }
        None => {
            let key = storage::new_uref(bidder).into();
            runtime::put_key(KEY_NAME, key);
        }
    }

    // This contract expects a single runtime argument to be provided.  The arg is named "message"
    // and will be of type `String`.
    // let value: String = runtime::get_named_arg(RUNTIME_ARG_NAME);

    // Store this value under a new unforgeable reference a.k.a `URef`.
    // let bidder_ref = storage::new_uref(bidder);

    // Store the new `URef` as a named key with a name of `KEY_NAME`.
    // let key = Key::URef(bidder_ref);
    // runtime::put_key(KEY_NAME, key);

    // The key should now be able to be retrieved.  Note that if `get_key()` returns `None`, then
    // `unwrap_or_revert()` will exit the process, returning `ApiError::None`.
    // let retrieved_key = runtime::get_key(KEY_NAME).unwrap_or_revert();
    // if retrieved_key != key {
    //     runtime::revert(Error::KeyMismatch);`
    // }
}

#[no_mangle]
pub extern "C" fn call() {
    let (contract_package_hash, _) = storage::create_contract_package_at_hash();

        // Initialize counter to 0.
    // let counter_local_key = storage::new_uref(0_i32);

    // // Create initial named keys of the contract.
    // let mut counter_named_keys: BTreeMap<String, Key> = BTreeMap::new();
    // let key_name = String::from(COUNT_KEY);
    // counter_named_keys.insert(key_name, counter_local_key.into());


    let mut market_entry_points = EntryPoints::new();
    market_entry_points.add_entry_point(endpoint(MAKE_OFFER));
    market_entry_points.add_entry_point(endpoint(CREATE_LISTING));
    market_entry_points.add_entry_point(endpoint(FIND_PRICE));

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
    runtime::put_key("market_contract", contract_hash.into());
    let contract_hash_pack = storage::new_uref(contract_hash);
    runtime::put_key("market_contract_hash", contract_hash_pack.into())
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