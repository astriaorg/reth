use crate::{
    // FIXME: gotta import nodes once they are available
    // net::{goerli_nodes, mainnet_nodes, sepolia_nodes},
    NodeRecord, U256 as revmU256, symphony_primitives::symphony_chains::SymphonyChains,
};
use ethers_core::types::U64;
// use reth_codecs::add_arbitrary_tests;
use reth_rlp::{Decodable, Encodable};
use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};

// The chain spec module.
mod spec;
mod static_specs;

// Re-exports
pub use static_specs::MAINNET_SPEC;

pub use spec::{
    chainspec::ChainSpec, ForkCondition, // ChainSpecBuilder,  
};

// The chain info module.
mod info;
pub use info::ChainInfo;

#[cfg(test)]
mod tests;


/// Either a named or chain id or the actual id value
// #[add_arbitrary_tests(rlp)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Chain {
    /// Contains a known chain
    Named(SymphonyChains),
    /// Contains the id of a chain
    Id(u64),
}

impl Chain {
    /// Returns the mainnet chain.
    pub const fn mainnet() -> Self {
        Chain::Named(SymphonyChains::Mainnet)
    }

    /// Returns the devnet chain.
    pub const fn devnet() -> Self {
        Chain::Named(SymphonyChains::Devnet)
    }

    /// Returns the testnet chain.
    pub const fn testnet() -> Self {
        Chain::Named(SymphonyChains::Testnet)
    }

    /// The id of the chain
    pub fn id(&self) -> u64 {
        match self {
            Chain::Named(chain) => *chain as u64,
            Chain::Id(id) => *id,
        }
    }

    /// Returns the address of the public DNS node list for the given chain.
    ///
    /// See also <https://github.com/ethereum/discv4-dns-lists>
    pub fn public_dns_network_protocol(self) -> Option<String> {
        // use ethers_core::types::Chain::*;
        // const DNS_PREFIX: &str = "enrtree://AKA3AM6LPBYEUDMVNU3BSVQJ5AD45Y7YPOHJLEF6W26QOE4VTUDPE@";

        // let named: ethers_core::types::Chain = self.try_into().ok()?;

        // if matches!(named, Mainnet | Goerli | Sepolia | Ropsten | Rinkeby) {
        //     return Some(format!("{DNS_PREFIX}all.{}.ethdisco.net", named.as_ref().to_lowercase()))
        // }
        // None

        unimplemented!()
    }

    /// Returns bootnodes for the given chain.
    pub fn bootnodes(self) -> Option<Vec<NodeRecord>> {
        // use ethers_core::types::Chain::*;
        // match self.try_into().ok()? {
        //     Mainnet => Some(mainnet_nodes()),
        //     Goerli => Some(goerli_nodes()),
        //     Sepolia => Some(sepolia_nodes()),
        //     _ => None,
        // }

        unimplemented!()
    }
}

impl fmt::Display for Chain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Chain::Named(chain) => chain.fmt(f),
            Chain::Id(id) => {
                if let Ok(chain) = ethers_core::types::Chain::try_from(*id) {
                    chain.fmt(f)
                } else {
                    id.fmt(f)
                }
            }
        }
    }
}

impl From<SymphonyChains> for Chain {
    fn from(id: SymphonyChains) -> Self {
        Chain::Named(id)
    }
}

impl From<u64> for Chain {
    fn from(id: u64) -> Self {
        match SymphonyChains::try_from(id) {
            Ok(x) => Chain::Named(x),
            _ => Chain::Id(id)
        }
    }
}

impl From<revmU256> for Chain {
    fn from(id: revmU256) -> Self {
        id.to::<u64>().into()
    }
}

impl From<Chain> for u64 {
    fn from(c: Chain) -> Self {
        match c {
            Chain::Named(c) => c as u64,
            Chain::Id(id) => id,
        }
    }
}

impl From<Chain> for U64 {
    fn from(c: Chain) -> Self {
        u64::from(c).into()
    }
}

impl From<Chain> for revmU256 {
    fn from(c: Chain) -> Self {
        revmU256::from(u64::from(c))
    }
}

impl FromStr for Chain {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(chain) = SymphonyChains::try_from(s) {
            Ok(Chain::Named(chain))
        } else {
            s.parse::<u64>()
                .map(Chain::Id)
                .map_err(|_| format!("Expected known chain or integer, found: {s}"))
        }
    }
}

impl Encodable for Chain {
    fn encode(&self, out: &mut dyn reth_rlp::BufMut) {
        match self {
            Self::Named(chain) => (*chain as u64).encode(out),
            Self::Id(id) => id.encode(out),
        }
    }
    fn length(&self) -> usize {
        match self {
            Self::Named(chain) => (*chain as u64).length(),
            Self::Id(id) => id.length(),
        }
    }
}

impl Decodable for Chain {
    fn decode(buf: &mut &[u8]) -> Result<Self, reth_rlp::DecodeError> {
        Ok(u64::decode(buf)?.into())
    }
}

impl Default for Chain {
    fn default() -> Self {
        Chain::Named(SymphonyChains::Mainnet)
    }
}

// #[cfg(any(test, feature = "arbitrary"))]
// impl<'a> arbitrary::Arbitrary<'a> for Chain {
//     fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
//         if u.ratio(1, 2)? {
//             let chain = u.int_in_range(0..=(ethers_core::types::Chain::COUNT - 1))?;

//             return Ok(Chain::Named(ethers_core::types::Chain::iter().nth(chain).expect("in range")))
//         }

//         Ok(Self::Id(u64::arbitrary(u)?))
//     }
// }

// #[cfg(any(test, feature = "arbitrary"))]
// use strum::{EnumCount, IntoEnumIterator};

// #[cfg(any(test, feature = "arbitrary"))]
// use proptest::{
//     arbitrary::ParamsFor,
//     prelude::{any, Strategy},
//     sample::Selector,
//     strategy::BoxedStrategy,
// };

// #[cfg(any(test, feature = "arbitrary"))]
// impl proptest::arbitrary::Arbitrary for Chain {
//     type Parameters = ParamsFor<u32>;
//     fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
//         let named = any::<Selector>()
//             .prop_map(move |sel| Chain::Named(sel.select(ethers_core::types::Chain::iter())));
//         let id = any::<u64>().prop_map(Chain::from);
//         proptest::strategy::Union::new_weighted(vec![(50, named.boxed()), (50, id.boxed())]).boxed()
//     }

//     type Strategy = BoxedStrategy<Chain>;
// }
