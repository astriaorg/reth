use std::fmt::{Debug, Display};

use crate::constants::{
    DEVNET_ID, DEVNET_NAME, MAINNET_ID, MAINNET_NAME, TESTNET_ID, TESTNET_NAME,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[repr(u64)]
pub enum SymphonyChains {
    Mainnet = MAINNET_ID,
    Devnet = DEVNET_ID,
    Testnet = TESTNET_ID,
}

#[derive(Debug)]
pub enum SymphonyChainError {
    UnrecognizedChainId,
    UnrecognizedStr,
}

impl TryFrom<u64> for SymphonyChains {
    type Error = SymphonyChainError;
    fn try_from(value: u64) -> Result<Self, Self::Error> {
        match value {
            MAINNET_ID => Ok(Self::Mainnet),
            DEVNET_ID => Ok(Self::Devnet),
            TESTNET_ID => Ok(Self::Testnet),
            _ => Err(SymphonyChainError::UnrecognizedChainId),
        }
    }
}

impl From<SymphonyChains> for u64 {
    fn from(value: SymphonyChains) -> Self {
        match value {
            SymphonyChains::Mainnet => MAINNET_ID,
            SymphonyChains::Devnet => DEVNET_ID,
            SymphonyChains::Testnet => TESTNET_ID,
        }
    }
}

impl TryFrom<&str> for SymphonyChains {
    type Error = SymphonyChainError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_uppercase().as_str() {
            MAINNET_NAME => Ok(Self::Mainnet),
            DEVNET_NAME => Ok(Self::Devnet),
            TESTNET_NAME => Ok(Self::Testnet),
            _ => Err(SymphonyChainError::UnrecognizedStr),
        }
    }
}

impl Display for SymphonyChains {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let chain_name = match self {
            SymphonyChains::Mainnet => MAINNET_NAME,
            SymphonyChains::Devnet => DEVNET_NAME,
            SymphonyChains::Testnet => TESTNET_NAME,
        };

        let chain_name = chain_name.to_lowercase();

        write!(f, "symphony-{chain_name}")
    }
}
