pub mod chainspec;

use crate::{
    constants::{EIP1559_INITIAL_BASE_FEE, EMPTY_WITHDRAWALS},
    forkid::ForkFilterKey,
    header::Head,
    proofs::genesis_state_root,
    BlockNumber, Chain, ForkFilter, ForkHash, ForkId, Genesis, GenesisAccount, Hardfork, Header,
    SealedHeader, H160, H256, U256,
};
use ethers_core::utils::Genesis as EthersGenesis;
use hex_literal::hex;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, HashMap},
    sync::Arc,
};






/// Various timestamps of forks
#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct ForkTimestamps {
    /// The timestamp of the shanghai fork
    pub shanghai: Option<u64>,
}

impl ForkTimestamps {
    /// Creates a new [`ForkTimestamps`] from the given hardforks by extracing the timestamps
    fn from_hardforks(forks: &BTreeMap<Hardfork, ForkCondition>) -> Self {
        let mut timestamps = ForkTimestamps::default();
        if let Some(shanghai) = forks.get(&Hardfork::Shanghai).and_then(|f| f.as_timestamp()) {
            timestamps = timestamps.shanghai(shanghai);
        }
        timestamps
    }

    /// Sets the given shanghai timestamp
    pub fn shanghai(mut self, shanghai: u64) -> Self {
        self.shanghai = Some(shanghai);
        self
    }
}

/// A helper type for compatibility with geth's config
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum AllGenesisFormats {
    /// The geth genesis format
    Geth(EthersGenesis),
    /// The reth genesis format
    Reth(ChainSpec),
}

impl From<EthersGenesis> for AllGenesisFormats {
    fn from(genesis: EthersGenesis) -> Self {
        Self::Geth(genesis)
    }
}

impl From<Arc<ChainSpec>> for AllGenesisFormats {
    fn from(mut genesis: Arc<ChainSpec>) -> Self {
        let cloned_genesis = Arc::make_mut(&mut genesis).clone();
        Self::Reth(cloned_genesis)
    }
}

impl From<AllGenesisFormats> for ChainSpec {
    fn from(genesis: AllGenesisFormats) -> Self {
        match genesis {
            AllGenesisFormats::Geth(genesis) => genesis.into(),
            AllGenesisFormats::Reth(genesis) => genesis,
        }
    }
}

/// A helper to build custom chain specs
// #[derive(Debug, Default)]
// pub struct ChainSpecBuilder {
//     chain: Option<Chain>,
//     genesis: Option<Genesis>,
//     hardforks: BTreeMap<Hardfork, ForkCondition>,
// }

// impl ChainSpecBuilder {
//     /// Construct a new builder from the mainnet chain spec.
//     pub fn mainnet() -> Self {
//         Self {
//             chain: Some(MAINNET.chain),
//             genesis: Some(MAINNET.genesis.clone()),
//             hardforks: MAINNET.hardforks.clone(),
//         }
//     }

//     /// Set the chain ID
//     pub fn chain(mut self, chain: Chain) -> Self {
//         self.chain = Some(chain);
//         self
//     }

//     /// Set the genesis block.
//     pub fn genesis(mut self, genesis: Genesis) -> Self {
//         self.genesis = Some(genesis);
//         self
//     }

//     /// Add the given fork with the given activation condition to the spec.
//     pub fn with_fork(mut self, fork: Hardfork, condition: ForkCondition) -> Self {
//         self.hardforks.insert(fork, condition);
//         self
//     }

//     /// Enable the Paris hardfork at the given TTD.
//     ///
//     /// Does not set the merge netsplit block.
//     pub fn paris_at_ttd(self, ttd: U256) -> Self {
//         self.with_fork(
//             Hardfork::Paris,
//             ForkCondition::TTD { total_difficulty: ttd, fork_block: None },
//         )
//     }

//     /// Enable Frontier at genesis.
//     pub fn frontier_activated(mut self) -> Self {
//         self.hardforks.insert(Hardfork::Frontier, ForkCondition::Block(0));
//         self
//     }

//     /// Enable Homestead at genesis.
//     pub fn homestead_activated(mut self) -> Self {
//         self = self.frontier_activated();
//         self.hardforks.insert(Hardfork::Homestead, ForkCondition::Block(0));
//         self
//     }

//     /// Enable Tangerine at genesis.
//     pub fn tangerine_whistle_activated(mut self) -> Self {
//         self = self.homestead_activated();
//         self.hardforks.insert(Hardfork::Tangerine, ForkCondition::Block(0));
//         self
//     }

//     /// Enable Spurious Dragon at genesis.
//     pub fn spurious_dragon_activated(mut self) -> Self {
//         self = self.tangerine_whistle_activated();
//         self.hardforks.insert(Hardfork::SpuriousDragon, ForkCondition::Block(0));
//         self
//     }

//     /// Enable Byzantium at genesis.
//     pub fn byzantium_activated(mut self) -> Self {
//         self = self.spurious_dragon_activated();
//         self.hardforks.insert(Hardfork::Byzantium, ForkCondition::Block(0));
//         self
//     }

//     /// Enable Petersburg at genesis.
//     pub fn petersburg_activated(mut self) -> Self {
//         self = self.byzantium_activated();
//         self.hardforks.insert(Hardfork::Petersburg, ForkCondition::Block(0));
//         self
//     }

//     /// Enable Istanbul at genesis.
//     pub fn istanbul_activated(mut self) -> Self {
//         self = self.petersburg_activated();
//         self.hardforks.insert(Hardfork::Istanbul, ForkCondition::Block(0));
//         self
//     }

//     /// Enable Berlin at genesis.
//     pub fn berlin_activated(mut self) -> Self {
//         self = self.istanbul_activated();
//         self.hardforks.insert(Hardfork::Berlin, ForkCondition::Block(0));
//         self
//     }

//     /// Enable London at genesis.
//     pub fn london_activated(mut self) -> Self {
//         self = self.berlin_activated();
//         self.hardforks.insert(Hardfork::London, ForkCondition::Block(0));
//         self
//     }

//     /// Enable Paris at genesis.
//     pub fn paris_activated(mut self) -> Self {
//         self = self.london_activated();
//         self.hardforks.insert(
//             Hardfork::Paris,
//             ForkCondition::TTD { fork_block: Some(0), total_difficulty: U256::ZERO },
//         );
//         self
//     }

//     /// Enable Shanghai at genesis.
//     pub fn shanghai_activated(mut self) -> Self {
//         self = self.paris_activated();
//         self.hardforks.insert(Hardfork::Shanghai, ForkCondition::Timestamp(0));
//         self
//     }

//     /// Build the resulting [`ChainSpec`].
//     ///
//     /// # Panics
//     ///
//     /// This function panics if the chain ID and genesis is not set ([`Self::chain`] and
//     /// [`Self::genesis`])
//     pub fn build(self) -> ChainSpec {
//         ChainSpec {
//             chain: self.chain.expect("The chain is required"),
//             genesis: self.genesis.expect("The genesis is required"),
//             genesis_hash: None,
//             fork_timestamps: ForkTimestamps::from_hardforks(&self.hardforks),
//             hardforks: self.hardforks,
//             paris_block_and_final_difficulty: None,
//         }
//     }
// }

// impl From<&Arc<ChainSpec>> for ChainSpecBuilder {
//     fn from(value: &Arc<ChainSpec>) -> Self {
//         Self {
//             chain: Some(value.chain),
//             genesis: Some(value.genesis.clone()),
//             hardforks: value.hardforks.clone(),
//         }
//     }
// }

/// The condition at which a fork is activated.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ForkCondition {
    /// The fork is activated after a certain block.
    Block(BlockNumber),
    /// The fork is activated after a total difficulty has been reached.
    TTD {
        /// The block number at which TTD is reached, if it is known.
        ///
        /// This should **NOT** be set unless you want this block advertised as [EIP-2124][eip2124]
        /// `FORK_NEXT`. This is currently only the case for Sepolia.
        ///
        /// [eip2124]: https://eips.ethereum.org/EIPS/eip-2124
        fork_block: Option<BlockNumber>,
        /// The total difficulty after which the fork is activated.
        total_difficulty: U256,
    },
    /// The fork is activated after a specific timestamp.
    Timestamp(u64),
    /// The fork is never activated
    #[default]
    Never,
}

impl ForkCondition {
    /// Returns true if the fork condition is timestamp based.
    pub fn is_timestamp(&self) -> bool {
        matches!(self, ForkCondition::Timestamp(_))
    }

    /// Checks whether the fork condition is satisfied at the given block.
    ///
    /// For TTD conditions, this will only return true if the activation block is already known.
    ///
    /// For timestamp conditions, this will always return false.
    pub fn active_at_block(&self, current_block: BlockNumber) -> bool {
        match self {
            ForkCondition::Block(block) => current_block >= *block,
            ForkCondition::TTD { fork_block: Some(block), .. } => current_block >= *block,
            _ => false,
        }
    }

    /// Checks if the given block is the first block that satisfies the fork condition.
    ///
    /// This will return false for any condition that is not block based.
    pub fn transitions_at_block(&self, current_block: BlockNumber) -> bool {
        match self {
            ForkCondition::Block(block) => current_block == *block,
            _ => false,
        }
    }

    /// Checks whether the fork condition is satisfied at the given total difficulty and difficulty
    /// of a current block.
    ///
    /// The fork is considered active if the _previous_ total difficulty is above the threshold.
    /// To achieve that, we subtract the passed `difficulty` from the current block's total
    /// difficulty, and check if it's above the Fork Condition's total difficulty (here:
    /// 58_750_000_000_000_000_000_000)
    ///
    /// This will return false for any condition that is not TTD-based.
    pub fn active_at_ttd(&self, ttd: U256, difficulty: U256) -> bool {
        if let ForkCondition::TTD { total_difficulty, .. } = self {
            ttd.saturating_sub(difficulty) >= *total_difficulty
        } else {
            false
        }
    }

    /// Checks whether the fork condition is satisfied at the given timestamp.
    ///
    /// This will return false for any condition that is not timestamp-based.
    pub fn active_at_timestamp(&self, timestamp: u64) -> bool {
        if let ForkCondition::Timestamp(time) = self {
            timestamp >= *time
        } else {
            false
        }
    }

    /// Checks whether the fork condition is satisfied at the given head block.
    ///
    /// This will return true if:
    ///
    /// - The condition is satisfied by the block number;
    /// - The condition is satisfied by the timestamp;
    /// - or the condition is satisfied by the total difficulty
    pub fn active_at_head(&self, head: &Head) -> bool {
        self.active_at_block(head.number) ||
            self.active_at_timestamp(head.timestamp) ||
            self.active_at_ttd(head.total_difficulty, head.difficulty)
    }

    /// Get the total terminal difficulty for this fork condition.
    ///
    /// Returns `None` for fork conditions that are not TTD based.
    pub fn ttd(&self) -> Option<U256> {
        match self {
            ForkCondition::TTD { total_difficulty, .. } => Some(*total_difficulty),
            _ => None,
        }
    }

    /// An internal helper function that gives a value that satisfies this condition.
    pub(crate) fn satisfy(&self) -> Head {
        match *self {
            ForkCondition::Block(number) => Head { number, ..Default::default() },
            ForkCondition::Timestamp(timestamp) => Head { timestamp, ..Default::default() },
            ForkCondition::TTD { total_difficulty, .. } => {
                Head { total_difficulty, ..Default::default() }
            }
            ForkCondition::Never => unreachable!(),
        }
    }

    /// Returns the timestamp of the fork condition, if it is timestamp based.
    pub fn as_timestamp(&self) -> Option<u64> {
        match self {
            ForkCondition::Timestamp(timestamp) => Some(*timestamp),
            _ => None,
        }
    }
}

