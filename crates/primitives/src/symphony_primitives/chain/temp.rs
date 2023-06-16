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