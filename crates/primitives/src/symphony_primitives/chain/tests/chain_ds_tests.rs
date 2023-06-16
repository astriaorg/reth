use ethers_core::types::{U64};
use reth_rlp::Encodable;
use revm_primitives::U256 as revmU256;
use std::str::FromStr;

use crate::{symphony_chains::SymphonyChains, symphony_constants::DEVNET_ID, Chain};

#[test]
fn test_from_str_named_chain_error() {
    let result = Chain::from_str("chain");

    assert!(result.is_err());
}

#[test]
fn test_from_str_id_chain() {
    let result = Chain::from_str("1234");
    let expected = Chain::Id(1234);

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), expected);
}

#[test]
fn test_default() {
    let default = Chain::default();
    let expected = Chain::Named(SymphonyChains::Mainnet);

    assert_eq!(default, expected);
}

#[test]
fn test_id_chain_encodable_length() {
    let chain = Chain::Id(1234);

    assert_eq!(chain.length(), 3);
}

#[test]
fn test_from_str_named_chain() {
    let result = Chain::from_str("mainnet");
    let expected = Chain::Named(SymphonyChains::Mainnet);

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), expected);
}

#[test]
fn test_id() {
    let chain = Chain::Id(1234);
    assert_eq!(chain.id(), 1234);
}

#[test]
fn test_named_id() {
    let chain = Chain::Named(SymphonyChains::Devnet);
    assert_eq!(chain.id(), DEVNET_ID);
}

#[test]
fn test_display_named_chain() {
    let chain = Chain::Named(SymphonyChains::Mainnet);
    assert_eq!(format!("{chain}"), "symphony-mainnet");
}

#[test]
fn test_display_id_chain() {
    let chain = Chain::Id(1234);
    assert_eq!(format!("{chain}"), "1234");
}

#[test]
fn test_from_u256() {
    let n = revmU256::from(1234);
    let chain = Chain::from(n);
    let expected = Chain::Id(1234);

    assert_eq!(chain, expected);
}

#[test]
fn test_into_u256() {
    let chain = Chain::Named(SymphonyChains::Devnet);
    let n: revmU256 = chain.into();
    let expected = revmU256::from(DEVNET_ID);

    assert_eq!(n, expected);
}

#[test]
#[allow(non_snake_case)]
fn test_into_U64() {
    let chain = Chain::Named(SymphonyChains::Devnet);
    let n: U64 = chain.into();
    let expected = U64::from(DEVNET_ID);

    assert_eq!(n, expected);
}

//     #[test]
//     fn test_dns_network() {
//         let s = "enrtree://AKA3AM6LPBYEUDMVNU3BSVQJ5AD45Y7YPOHJLEF6W26QOE4VTUDPE@all.mainnet.ethdisco.net";
//         let chain: Chain = ethers_core::types::Chain::Mainnet.into();
//         assert_eq!(s, chain.public_dns_network_protocol().unwrap().as_str());
//     }