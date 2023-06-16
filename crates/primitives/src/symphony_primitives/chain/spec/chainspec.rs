
use crate::{Chain, Genesis};

use ethers_core::types::H256;
use serde::{Serialize, Deserialize};

/// An Ethereum chain specification.
///
/// A chain specification describes:
///
/// - Meta-information about the chain (the chain ID)
/// - The genesis block of the chain ([`Genesis`])
/// - What hardforks are activated, and under which conditions
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChainSpec {
    /// The chain ID
    pub chain: Chain,

    /// The hash of the genesis block.
    ///
    /// This acts as a small cache for known chains. If the chain is known, then the genesis hash
    /// is also known ahead of time, and this will be `Some`.
    #[serde(skip, default)]
    pub genesis_hash: Option<H256>,

    /// The genesis block
    pub genesis: Genesis,

    /// The block at which [Hardfork::Paris] was activated and the final difficulty at this block.
    #[serde(skip, default)]
    pub paris_block_and_final_difficulty: Option<(u64, U256)>,

    /// Timestamps of various hardforks
    ///
    /// This caches entries in `hardforks` map
    #[serde(skip, default)]
    pub fork_timestamps: ForkTimestamps,

    /// The active hard forks and their activation conditions
    pub hardforks: BTreeMap<Hardfork, ForkCondition>,
}

impl ChainSpec {
    /// Get information about the chain itself
    pub fn chain(&self) -> Chain {
        self.chain
    }

    /// Get the genesis block specification.
    ///
    /// To get the header for the genesis block, use [`Self::genesis_header`] instead.
    pub fn genesis(&self) -> &Genesis {
        &self.genesis
    }

    /// Get the header for the genesis block.
    pub fn genesis_header(&self) -> Header {
        // If London is activated at genesis, we set the initial base fee as per EIP-1559.
        let base_fee_per_gas = self.initial_base_fee();

        // If shanghai is activated, initialize the header with an empty withdrawals hash, and
        // empty withdrawals list.
        let withdrawals_root =
            (self.fork(Hardfork::Shanghai).active_at_timestamp(self.genesis.timestamp))
                .then_some(EMPTY_WITHDRAWALS);

        Header {
            gas_limit: self.genesis.gas_limit,
            difficulty: self.genesis.difficulty,
            nonce: self.genesis.nonce,
            extra_data: self.genesis.extra_data.clone(),
            state_root: genesis_state_root(&self.genesis.alloc),
            timestamp: self.genesis.timestamp,
            mix_hash: self.genesis.mix_hash,
            beneficiary: self.genesis.coinbase,
            base_fee_per_gas,
            withdrawals_root,
            ..Default::default()
        }
    }

    /// Get the sealed header for the genesis block.
    pub fn sealed_genesis_header(&self) -> SealedHeader {
        SealedHeader { header: self.genesis_header(), hash: self.genesis_hash() }
    }

    /// Get the initial base fee of the genesis block.
    pub fn initial_base_fee(&self) -> Option<u64> {
        // If London is activated at genesis, we set the initial base fee as per EIP-1559.
        (self.fork(Hardfork::London).active_at_block(0)).then_some(EIP1559_INITIAL_BASE_FEE)
    }

    /// Get the hash of the genesis block.
    pub fn genesis_hash(&self) -> H256 {
        if let Some(hash) = self.genesis_hash {
            hash
        } else {
            self.genesis_header().hash_slow()
        }
    }

    /// Returns the final difficulty if the given block number is after the Paris hardfork.
    ///
    /// Note: technically this would also be valid for the block before the paris upgrade, but this
    /// edge case is omitted here.
    pub fn final_paris_difficulty(&self, block_number: u64) -> Option<U256> {
        self.paris_block_and_final_difficulty.and_then(|(activated_at, final_difficulty)| {
            if block_number >= activated_at {
                Some(final_difficulty)
            } else {
                None
            }
        })
    }

    /// Returns the forks in this specification and their activation conditions.
    pub fn hardforks(&self) -> &BTreeMap<Hardfork, ForkCondition> {
        &self.hardforks
    }

    /// Get the fork condition for the given fork.
    pub fn fork(&self, fork: Hardfork) -> ForkCondition {
        self.hardforks.get(&fork).copied().unwrap_or(ForkCondition::Never)
    }

    /// Get an iterator of all hardforks with their respective activation conditions.
    pub fn forks_iter(&self) -> impl Iterator<Item = (Hardfork, ForkCondition)> + '_ {
        self.hardforks.iter().map(|(f, b)| (*f, *b))
    }

    /// Convenience method to check if a fork is active at a given timestamp.
    #[inline]
    pub fn is_fork_active_at_timestamp(&self, fork: Hardfork, timestamp: u64) -> bool {
        self.fork(fork).active_at_timestamp(timestamp)
    }

    /// Convenience method to check if [Hardfork::Shanghai] is active at a given timestamp.
    #[inline]
    pub fn is_shanghai_activated_at_timestamp(&self, timestamp: u64) -> bool {
        self.fork_timestamps
            .shanghai
            .map(|shanghai| timestamp >= shanghai)
            .unwrap_or_else(|| self.is_fork_active_at_timestamp(Hardfork::Shanghai, timestamp))
    }

    /// Creates a [`ForkFilter`](crate::ForkFilter) for the block described by [Head].
    pub fn fork_filter(&self, head: Head) -> ForkFilter {
        let forks = self.forks_iter().filter_map(|(_, condition)| {
            // We filter out TTD-based forks w/o a pre-known block since those do not show up in the
            // fork filter.
            Some(match condition {
                ForkCondition::Block(block) => ForkFilterKey::Block(block),
                ForkCondition::Timestamp(time) => ForkFilterKey::Time(time),
                ForkCondition::TTD { fork_block: Some(block), .. } => ForkFilterKey::Block(block),
                _ => return None,
            })
        });

        ForkFilter::new(head, self.genesis_hash(), forks)
    }

    /// Compute the [`ForkId`] for the given [`Head`]
    pub fn fork_id(&self, head: &Head) -> ForkId {
        let mut curr_forkhash = ForkHash::from(self.genesis_hash());
        let mut current_applied_value = 0;

        for (_, cond) in self.forks_iter() {
            let value = match cond {
                ForkCondition::Block(block) => block,
                ForkCondition::Timestamp(time) => time,
                ForkCondition::TTD { fork_block: Some(block), .. } => block,
                _ => continue,
            };

            if cond.active_at_head(head) {
                if value != current_applied_value {
                    curr_forkhash += value;
                    current_applied_value = value;
                }
            } else {
                return ForkId { hash: curr_forkhash, next: value }
            }
        }
        ForkId { hash: curr_forkhash, next: 0 }
    }

    // Build a chainspec using [`ChainSpecBuilder`]
    // pub fn builder() -> ChainSpecBuilder {
        // ChainSpecBuilder::default()
    // }
}
