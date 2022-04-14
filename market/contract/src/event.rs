use alloc::{
    string::String
};
use casper_types::{ContractPackageHash, Key, U512};

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
    },
}