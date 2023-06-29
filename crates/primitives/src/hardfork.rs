use serde::{Deserialize, Serialize};
use parse_display::{Display, FromStr};

use crate::{ChainSpec, ForkCondition, ForkFilter, ForkId};

/// The name of an Ethereum hardfork.
#[derive(Debug, Copy, Clone, Eq, PartialEq, PartialOrd, Ord, Hash, Serialize, Deserialize, Display, FromStr)]
#[non_exhaustive]
#[display(style="lowercase")]
pub enum Hardfork {
    /// Acapella
    Acapella 
}

impl Hardfork {
    /// Get the [ForkId] for this hardfork in the given spec, if the fork is activated at any point.
    pub fn fork_id(&self, spec: &ChainSpec) -> Option<ForkId> {
        match spec.fork(*self) {
            ForkCondition::Never => None,
            _ => Some(spec.fork_id(&spec.fork(*self).satisfy())),
        }
    }

    /// Get the [ForkFilter] for this hardfork in the given spec, if the fork is activated at any
    /// point.
    pub fn fork_filter(&self, spec: &ChainSpec) -> Option<ForkFilter> {
        match spec.fork(*self) {
            ForkCondition::Never => None,
            _ => Some(spec.fork_filter(spec.fork(*self).satisfy())),
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    use crate::{Chain, Genesis};
    use std::{collections::BTreeMap, str::FromStr};

    #[test]
    fn check_hardfork_from_str() {
        let hardfork_str = [
            "acapella"
        ];
        let expected_hardforks = [
            Hardfork::Acapella,
        ];

        let hardforks: Vec<Hardfork> =
            hardfork_str.iter().map(|h| Hardfork::from_str(h).unwrap()).collect();

        assert_eq!(hardforks, expected_hardforks);
    }

    #[test]
    fn check_nonexistent_hardfork_from_str() {
        assert!(Hardfork::from_str("not a hardfork").is_err());
    }

    #[test]
    fn check_fork_id_chainspec_with_fork_condition_never() {
        let spec = ChainSpec {
            chain: Chain::mainnet(),
            genesis: Genesis::default(),
            genesis_hash: None,
            hardforks: BTreeMap::from([(Hardfork::Acapella, ForkCondition::Never)]),
            fork_timestamps: Default::default(),
            paris_block_and_final_difficulty: None,
        };

        assert_eq!(Hardfork::Acapella.fork_id(&spec), None);
    }

    #[test]
    fn check_fork_filter_chainspec_with_fork_condition_never() {
        let spec = ChainSpec {
            chain: Chain::mainnet(),
            genesis: Genesis::default(),
            genesis_hash: None,
            hardforks: BTreeMap::from([(Hardfork::Acapella, ForkCondition::Never)]),
            fork_timestamps: Default::default(),
            paris_block_and_final_difficulty: None,
        };

        assert_eq!(Hardfork::Acapella.fork_filter(&spec), None);
    }
}
