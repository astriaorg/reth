use serde::{Deserialize, Serialize};

use crate::{ForkFilter, ForkId};
use std::{fmt::Display, str::FromStr};

use crate::ForkCondition;

/// The name of an Ethereum hardfork.
#[derive(Debug, Copy, Clone, Eq, PartialEq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Hardfork {
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

impl FromStr for Hardfork {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.to_lowercase();
        let hardfork = match s.as_str() {
            _ => return Err(format!("Unknown hardfork: {s}")),
        };
        Ok(hardfork)
    }
}

impl Display for Hardfork {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}