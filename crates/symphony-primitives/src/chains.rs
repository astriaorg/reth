
use std::{fmt::{Display, Debug}};

use serde::{Serialize, Deserialize};
use crate::constants::{DEVNET_ID, MAINNET_ID, TESTNET_ID};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[repr(u64)]
pub enum SymphonyChains {
    Mainnet = MAINNET_ID,
    Devnet = DEVNET_ID,
    Testnet = TESTNET_ID
}

#[derive(Debug)]
pub enum SymphonyChainError {
    UnrecognizedChainId,
    UnrecognizedStr
}

impl TryFrom<u64> for SymphonyChains {
    type Error = SymphonyChainError;
    fn try_from(value: u64) -> Result<Self, Self::Error> {
        match value {
            MAINNET_ID => Ok(Self::Mainnet),
            DEVNET_ID => Ok(Self::Devnet),
            TESTNET_ID => Ok(Self::Testnet),
            _ => Err(SymphonyChainError::UnrecognizedChainId)
        }
    }
}

impl TryFrom<SymphonyChains> for u64 {
    type Error = SymphonyChainError;
    fn try_from(value: SymphonyChains) -> Result<Self, Self::Error> {
        match value {
            SymphonyChains::Mainnet => Ok(MAINNET_ID),
            SymphonyChains::Devnet => Ok(DEVNET_ID),
            SymphonyChains::Testnet => Ok(TESTNET_ID),
            _ => Err(SymphonyChainError::UnrecognizedChainId)
        }
    }
}


impl TryFrom<&str> for SymphonyChains {
    type Error = SymphonyChainError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "mainnet" => Ok(Self::Mainnet),
            "devnet" => Ok(Self::Devnet),
            "testnet" => Ok(Self::Testnet),
            _ => Err(SymphonyChainError::UnrecognizedStr)
        }
    }
}

impl Display for SymphonyChains {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let chain_name = match self {
            SymphonyChains::Mainnet => "mainnet",
            SymphonyChains::Devnet => "devnet",
            SymphonyChains::Testnet => "testnet"
        };

        write!(f, "symphony-{chain_name}") 
    }
}