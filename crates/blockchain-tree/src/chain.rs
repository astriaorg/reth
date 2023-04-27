//! A chain in a [`BlockchainTree`][super::BlockchainTree].
//!
//! A [`Chain`] contains the state of accounts for the chain after execution of its constituent
//! blocks, as well as a list of the blocks the chain is composed of.
use crate::{post_state::PostState, PostStateDataRef};
use reth_db::database::Database;
use reth_interfaces::{
    consensus::{Consensus, ConsensusError},
    executor::Error as ExecError,
    Error,
};
use reth_primitives::{
    constants, BlockHash, BlockNumber, Chain as PrimitivesChain, ForkBlock, Hardfork, Header,
    SealedBlockWithSenders, SealedHeader, EMPTY_OMMER_ROOT, U256,
};
use reth_provider::{
    providers::PostStateProvider, BlockExecutor, Chain, ExecutorFactory, PostStateDataProvider,
};
use std::{
    collections::BTreeMap,
    ops::{Deref, DerefMut},
    time::SystemTime,
};

use super::externals::TreeExternals;

/// The ID of a sidechain internally in a [`BlockchainTree`][super::BlockchainTree].
pub(crate) type BlockChainId = u64;

/// A chain if the blockchain tree, that has functionality to execute blocks and append them to the
/// it self.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AppendableChain {
    chain: Chain,
}

impl Deref for AppendableChain {
    type Target = Chain;

    fn deref(&self) -> &Self::Target {
        &self.chain
    }
}

impl DerefMut for AppendableChain {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.chain
    }
}

// added this
fn validate_header_extradata(header: &Header) -> Result<(), ConsensusError> {
    if header.extra_data.len() > 32 {
        Err(ConsensusError::ExtraDataExceedsMax { len: header.extra_data.len() })
    } else {
        Ok(())
    }
}

impl AppendableChain {
    /// Crate a new appendable chain from a given chain.
    pub fn new(chain: Chain) -> Self {
        Self { chain }
    }

    /// Get the chain.
    pub fn into_inner(self) -> Chain {
        self.chain
    }

    /// Create a new chain that forks off of the canonical chain.
    pub fn new_canonical_fork<DB, C, EF>(
        block: &SealedBlockWithSenders,
        parent_header: &SealedHeader,
        canonical_block_hashes: &BTreeMap<BlockNumber, BlockHash>,
        canonical_fork: ForkBlock,
        externals: &TreeExternals<DB, C, EF>,
    ) -> Result<Self, Error>
    where
        DB: Database,
        C: Consensus,
        EF: ExecutorFactory,
    {
        let state = PostState::default();
        let empty = BTreeMap::new();

        let state_provider = PostStateDataRef {
            state: &state,
            sidechain_block_hashes: &empty,
            canonical_block_hashes,
            canonical_fork,
        };

        let changeset = Self::validate_and_execute(
            block.clone(),
            parent_header,
            canonical_fork,
            state_provider,
            externals,
        )?;

        Ok(Self { chain: Chain::new(vec![(block.clone(), changeset)]) })
    }

    /// Create a new chain that forks off of an existing sidechain.
    pub fn new_chain_fork<DB, C, EF>(
        &self,
        block: SealedBlockWithSenders,
        side_chain_block_hashes: BTreeMap<BlockNumber, BlockHash>,
        canonical_block_hashes: &BTreeMap<BlockNumber, BlockHash>,
        canonical_fork: ForkBlock,
        externals: &TreeExternals<DB, C, EF>,
    ) -> Result<Self, Error>
    where
        DB: Database,
        C: Consensus,
        EF: ExecutorFactory,
    {
        let parent_number = block.number - 1;
        let parent = self
            .blocks()
            .get(&parent_number)
            .ok_or(ExecError::BlockNumberNotFoundInChain { block_number: parent_number })?;

        let mut state = self.state.clone();

        // Revert state to the state after execution of the parent block
        state.revert_to(parent.number);

        // Revert changesets to get the state of the parent that we need to apply the change.
        let post_state_data = PostStateDataRef {
            state: &state,
            sidechain_block_hashes: &side_chain_block_hashes,
            canonical_block_hashes,
            canonical_fork,
        };
        let block_state = Self::validate_and_execute(
            block.clone(),
            parent,
            canonical_fork,
            post_state_data,
            externals,
        )?;
        state.extend(block_state);

        let chain =
            Self { chain: Chain { state, blocks: BTreeMap::from([(block.number, block)]) } };

        // If all is okay, return new chain back. Present chain is not modified.
        Ok(chain)
    }

    /// Validate and execute the given block.
    fn validate_and_execute<PSDP, DB, C, EF>(
        block: SealedBlockWithSenders,
        parent_block: &SealedHeader,
        canonical_fork: ForkBlock,
        post_state_data_provider: PSDP,
        externals: &TreeExternals<DB, C, EF>,
    ) -> Result<PostState, Error>
    where
        PSDP: PostStateDataProvider,
        DB: Database,
        C: Consensus,
        EF: ExecutorFactory,
    {
        // === validate header with total difficulty ===
        // externals.consensus.validate_header_with_total_difficulty(&block, U256::MAX)?;
        // crates/consensus/beacon/src/beacon_consensus.rs:  40:    pub fn
        // validate_header_with_total_difficulty(
        if externals
            .chain_spec
            .fork(Hardfork::Paris)
            .active_at_ttd(U256::MAX, block.header.difficulty)
        {
            if block.header.difficulty != U256::ZERO {
                return Err(reth_interfaces::Error::Consensus(
                    ConsensusError::TheMergeDifficultyIsNotZero,
                ))
            }
            if block.header.nonce != 0 {
                return Err(reth_interfaces::Error::Consensus(
                    ConsensusError::TheMergeNonceIsNotZero,
                ))
            }
            if block.header.ommers_hash != EMPTY_OMMER_ROOT {
                return Err(reth_interfaces::Error::Consensus(
                    ConsensusError::TheMergeOmmerRootIsNotEmpty,
                ))
            }
            validate_header_extradata(&block.header)?;
        } else if externals.chain_spec.chain != PrimitivesChain::goerli() {
            validate_header_extradata(&block.header)?;
        }
        // === validate header with total difficulty ===

        // === validate header ===
        // externals.consensus.validate_header(&block)?;
        // Gas used needs to be less then gas limit. Gas used is going to be check after execution.
        if block.header.gas_used > block.header.gas_limit {
            return Err(reth_interfaces::Error::Consensus(
                ConsensusError::HeaderGasUsedExceedsGasLimit {
                    gas_used: block.header.gas_used,
                    gas_limit: block.header.gas_limit,
                },
            ))
        }
        // Check if timestamp is in future. Clock can drift but this can be consensus issue.
        let present_timestamp =
            SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
        if block.header.timestamp > present_timestamp {
            return Err(reth_interfaces::Error::Consensus(ConsensusError::TimestampIsInFuture {
                timestamp: block.header.timestamp,
                present_timestamp,
            }))
        }
        // Check if base fee is set.
        if externals.chain_spec.fork(Hardfork::London).active_at_block(block.header.number) &&
            block.header.base_fee_per_gas.is_none()
        {
            return Err(reth_interfaces::Error::Consensus(ConsensusError::BaseFeeMissing))
        }
        // EIP-4895: Beacon chain push withdrawals as operations
        if externals.chain_spec.fork(Hardfork::Shanghai).active_at_timestamp(block.header.timestamp) &&
            block.header.withdrawals_root.is_none()
        {
            return Err(reth_interfaces::Error::Consensus(ConsensusError::WithdrawalsRootMissing))
        } else if !externals
            .chain_spec
            .fork(Hardfork::Shanghai)
            .active_at_timestamp(block.header.timestamp) &&
            block.header.withdrawals_root.is_some()
        {
            return Err(reth_interfaces::Error::Consensus(ConsensusError::WithdrawalsRootUnexpected))
        }
        // === validate header ===

        // === validate header against parent ===
        // externals.consensus.validate_header_agains_parent(&block, parent_block)?;
        // Parent number is consistent.
        if parent_block.number + 1 != block.number {
            return Err(reth_interfaces::Error::Consensus(
                ConsensusError::ParentBlockNumberMismatch {
                    parent_block_number: parent_block.number,
                    block_number: block.number,
                },
            ))
        }
        // timestamp in past check
        if block.timestamp < parent_block.timestamp {
            return Err(reth_interfaces::Error::Consensus(ConsensusError::TimestampIsInPast {
                parent_timestamp: parent_block.timestamp,
                timestamp: block.timestamp,
            }))
        }
        // TODO Check difficulty increment between parent and child
        // Ace age did increment it by some formula that we need to follow.
        let mut parent_gas_limit = parent_block.gas_limit;
        // By consensus, gas_limit is multiplied by elasticity (*2) on
        // on exact block that hardfork happens.
        if externals.chain_spec.fork(Hardfork::London).transitions_at_block(block.number) {
            parent_gas_limit = parent_block.gas_limit * constants::EIP1559_ELASTICITY_MULTIPLIER;
        }
        // Check gas limit, max diff between child/parent gas_limit should be
        // max_diff=parent_gas/1024
        // eprintln!(
        //     "parent_gas_limit: {}, gas_limit: {}, sub: {}, val: {}",
        //     parent_gas_limit,
        //     block.gas_limit,
        //     block.gas_limit - parent_gas_limit,
        //     parent_gas_limit / 1024
        // );
        if block.gas_limit > parent_gas_limit {
            if block.gas_limit - parent_gas_limit >= parent_gas_limit / 1024 {
                return Err(reth_interfaces::Error::Consensus(
                    ConsensusError::GasLimitInvalidIncrease {
                        parent_gas_limit,
                        child_gas_limit: block.gas_limit,
                    },
                ))
            }
        } else if parent_gas_limit - block.gas_limit >= parent_gas_limit / 1024 {
            return Err(reth_interfaces::Error::Consensus(ConsensusError::GasLimitInvalidDecrease {
                parent_gas_limit,
                child_gas_limit: block.gas_limit,
            }))
        }
        // EIP-1559 check base fee
        if externals.chain_spec.fork(Hardfork::London).active_at_block(block.number) {
            let base_fee = block.base_fee_per_gas.ok_or(ConsensusError::BaseFeeMissing)?;

            let expected_base_fee =
                if externals.chain_spec.fork(Hardfork::London).transitions_at_block(block.number) {
                    constants::EIP1559_INITIAL_BASE_FEE
                } else {
                    // This BaseFeeMissing will not happen as previous blocks are checked to have
                    // them.
                    parent_block.next_block_base_fee().ok_or(ConsensusError::BaseFeeMissing)?
                };
            if expected_base_fee != base_fee {
                return Err(reth_interfaces::Error::Consensus(ConsensusError::BaseFeeDiff {
                    expected: expected_base_fee,
                    got: base_fee,
                }))
            }
        }
        // === validate header against parent ===

        // === validate block ===
        // externals.consensus.validate_block(&block)?;
        let ommers_hash = reth_primitives::proofs::calculate_ommers_root(block.ommers.iter());
        if block.header.ommers_hash != ommers_hash {
            return Err(reth_interfaces::Error::Consensus(ConsensusError::BodyOmmersHashDiff {
                got: ommers_hash,
                expected: block.header.ommers_hash,
            }))
        }
        // Check transaction root
        // TODO(onbjerg): This should probably be accessible directly on [Block]
        let transaction_root =
            reth_primitives::proofs::calculate_transaction_root(block.body.iter());
        if block.header.transactions_root != transaction_root {
            return Err(reth_interfaces::Error::Consensus(ConsensusError::BodyTransactionRootDiff {
                got: transaction_root,
                expected: block.header.transactions_root,
            }))
        }
        // EIP-4895: Beacon chain push withdrawals as operations
        if externals.chain_spec.fork(Hardfork::Shanghai).active_at_timestamp(block.timestamp) {
            let withdrawals =
                block.withdrawals.as_ref().ok_or(ConsensusError::BodyWithdrawalsMissing)?;
            let withdrawals_root =
                reth_primitives::proofs::calculate_withdrawals_root(withdrawals.iter());
            let header_withdrawals_root =
                block.withdrawals_root.as_ref().ok_or(ConsensusError::WithdrawalsRootMissing)?;
            if withdrawals_root != *header_withdrawals_root {
                return Err(reth_interfaces::Error::Consensus(
                    ConsensusError::BodyWithdrawalsRootDiff {
                        got: withdrawals_root,
                        expected: *header_withdrawals_root,
                    },
                ))
            }
            // Validate that withdrawal index is monotonically increasing within a block.
            if let Some(first) = withdrawals.first() {
                let mut prev_index = first.index;
                for withdrawal in withdrawals.iter().skip(1) {
                    let expected = prev_index + 1;
                    if expected != withdrawal.index {
                        return Err(reth_interfaces::Error::Consensus(
                            ConsensusError::WithdrawalIndexInvalid {
                                got: withdrawal.index,
                                expected,
                            },
                        ))
                    }
                    prev_index = withdrawal.index;
                }
            }
        }
        // === validate block ===

        let (unseal, senders) = block.into_components();
        let unseal = unseal.unseal();

        //get state provider.
        let db = externals.shareable_db();
        // TODO, small perf can check if caonical fork is the latest state.
        let history_provider = db.history_by_block_number(canonical_fork.number)?;
        let state_provider = history_provider;

        let provider = PostStateProvider::new(state_provider, post_state_data_provider);

        let mut executor = externals.executor_factory.with_sp(&provider);
        executor.execute_and_verify_receipt(&unseal, U256::MAX, Some(senders)).map_err(Into::into)
    }

    /// Validate and execute the given block, and append it to this chain.
    pub fn append_block<DB, C, EF>(
        &mut self,
        block: SealedBlockWithSenders,
        side_chain_block_hashes: BTreeMap<BlockNumber, BlockHash>,
        canonical_block_hashes: &BTreeMap<BlockNumber, BlockHash>,
        canonical_fork: ForkBlock,
        externals: &TreeExternals<DB, C, EF>,
    ) -> Result<(), Error>
    where
        DB: Database,
        C: Consensus,
        EF: ExecutorFactory,
    {
        let (_, parent_block) = self.blocks.last_key_value().expect("Chain has at least one block");

        let post_state_data = PostStateDataRef {
            state: &self.state,
            sidechain_block_hashes: &side_chain_block_hashes,
            canonical_block_hashes,
            canonical_fork,
        };

        let block_state = Self::validate_and_execute(
            block.clone(),
            parent_block,
            canonical_fork,
            post_state_data,
            externals,
        )?;
        self.state.extend(block_state);
        self.blocks.insert(block.number, block);
        Ok(())
    }
}
