use std::collections::BTreeMap;

use crate::{ForkCondition, Hardfork};

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