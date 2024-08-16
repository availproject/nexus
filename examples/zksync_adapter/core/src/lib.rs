use crate::types::LogProcessingOutput;
use adapter_sdk::types::AdapterPublicInputs;
use constants::{
    SystemLogKey, L2_LOG_ADDRESS_OFFSET, L2_LOG_KEY_OFFSET, L2_LOG_VALUE_OFFSET,
    L2_TO_L1_LOG_SERIALIZE_SIZE, MAX_NUMBER_OF_BLOBS,
};
use nexus_core::types::{RollupPublicInputsV2, H256};
#[cfg(any(feature = "native"))]
use nexus_core::zkvm::risczero::RiscZeroProof;
use serde::{Deserialize, Serialize};
use types::{CommitBatchInfo, MAX_NUMBER_OF_BLOBS};
use utils::*;
pub use zksync_types::{
    commitment::{serialize_commitments, L1BatchWithMetadata},
    ethabi::Token,
    ethabi::encode,
};
use zksync_types::{hasher::{keccak, Hasher}, log, web3::Bytes, U256, U64};
pub use crate::types::L1BatchWithMetadata;
pub mod constants;
pub mod types;
pub mod utils;
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MockProof(pub ());

pub struct STF {
    img_id: [u32; 8],
}

//TODO: Add generics for risczero types, so SP1 could be used as well.
impl STF {
    // TODO: do we need to implement all constraints checks as well ?
    fn process_l2_logs(
        &self,
        new_batch: CommitBatchInfo,
        expected_system_contract_upgrade_tx_hash: H256,
    ) -> Result<LogProcessingOutput, anyhow::Error> {
        let mut log_output: LogProcessingOutput;
        log_output.blob_hashes = [H256; MAX_NUMBER_OF_BLOBS];

        let emitted_l2_logs = new_batch.system_logs;

        for i in (0..emitted_l2_logs.len()).step_by(L2_TO_L1_LOG_SERIALIZE_SIZE) {
            let (log_sender, _) = utils::read_address(&emitted_l2_logs, i + L2_LOG_ADDRESS_OFFSET);
            let (log_key, _) = utils::read_uint256(&emitted_l2_logs, i + L2_LOG_KEY_OFFSET);
            let (log_value, _) = utils::read_bytes32(&emitted_l2_logs, i + L2_LOG_VALUE_OFFSET);

            if (log_key == U256::from(SystemLogKey::L2ToL1LogsTreeRootKey)) {
                log_output.l2_logs_tree_root = log_value;
            } else if (log_key == U256::from(SystemLogKey::TotalL2ToL1PubdataKey)) {
                log_output.pubdata_hash = log_value;
            } else if (log_key == U256::from(SystemLogKey::StateDiffHashKey)) {
                log_output.state_diff_hash = log_value;
            } else if (log_key == U256::from(SystemLogKey::PackedBatchAndL2BlockTimestampKey)) {
                log_output.packed_batch_and_l2_block_timestamp = log_value;
            } else if (log_key == U256::from(SystemLogKey::PrevBatchHashKey)) {
                log_output.previous_batch_hash = log_value;
            } else if (log_key == U256::from(SystemLogKey::ChainedPriorityTxnHashKey)) {
                log_output.chained_priority_txs_hash = log_value;
            } else if (log_key == U256::from(SystemLogKey::NumberOfLayer1TxsKey)) {
                log_output.number_of_layer1_txs = log_value;
            } else if (log_key >= U256::from(SystemLogKey::BlobOneHashKey)
                && log_key <= U256::from(SystemLogKey::BlobSixHashKey))
            {
                let blob_number = log_key - U256::from(SystemLogKey::BlobOneHashKey);
                log_output.blob_hashes[blob_number] = log_value;
            }
            // TODO: not implemented for now
            // else if(log_key == U256::from(SystemLogKey::ExpectedSystemContractUpgradeTxHashKey)) {}
            // else if(log_key > U256::from(SystemLogKey::ExpectedSystemContractUpgradeTxHashKey)) {}
        }

        Ok(log_output)
    }

    fn get_commit_batch_info(&self, new_rollup_pi: L1BatchWithMetadata) -> CommitBatchInfo {
        let commit_batch_info = CommitBatchInfo {
            batch_number: Token::Uint(U256::from(new_rollup_pi.header.number.0)),
            timestamp: Token::Uint(U256::from(new_rollup_pi.header.timestamp.0)),
            index_repeated_storage_changes: Token::Uint(U256::from(
                new_rollup_pi.metadata.rollup_last_leaf_index,
            )),
            new_state_root: Token::FixedBytes(new_rollup_pi.metadata.root_hash.as_bytes().to_vec()),
            number_of_layer1_txs: Token::Uint(U256::from(new_rollup_pi.header.l1_tx_count)),
            priority_operations_hash: Token::FixedBytes(
                new_rollup_pi
                    .header
                    .priority_ops_onchain_data_hash()
                    .as_bytes()
                    .to_vec(),
            ),
            bootloader_heap_initial_contents_hash: Token::FixedBytes(
                new_rollup_pi
                    .metadata
                    .bootloader_initial_content_commitment
                    .unwrap()
                    .as_bytes()
                    .to_vec(),
            ),
            events_queue_state_hash: Token::FixedBytes(
                new_rollup_pi
                    .metadata
                    .events_queue_commitment
                    .unwrap()
                    .as_bytes()
                    .to_vec(),
            ),
            system_logs: Token::Bytes(serialize_commitments(&new_rollup_pi.header.system_logs)),
            // TODO: need to confirm calculation
            pubdata_commitments: Token::Bytes(serialize_commitments(
                &new_rollup_pi.header.pubdata_input,
            )),
        };
    }

    fn create_batch_commitment(&self, new_batch: CommitBatchInfo, state_diff_hash: H256, blob_commitments: Vec<H256>, blob_hashes: Vec<H256>) -> H256 {
        // TODO: implementing taking batch commitment
        unimplemented!()
    }

    fn batch_pass_through_data(&self, batch: CommitBatchInfo) -> Bytes {
        encode(&[Token::Uint(batch.index_repeated_storage_changes), Token::FixedBytes(batch.new_state_root), Token::Uint(U64::zero()), Token::FixedBytes(H256::zero())])  
    }

    pub fn new(img_id: [u32; 8]) -> Self {
        Self { img_id }
    }

    pub fn verify_continuity_and_proof(
        &self,
        previous_adapter_pi: AdapterPublicInputs,
        new_rollup_proof: MockProof,
        new_rollup_pi: L1BatchWithMetadata,
    ) -> Result<AdapterPublicInputs, anyhow::Error> {
        let new_batch = get_commit_batch_info(new_rollup_pi);
        // TODO: need to change
        let expected_system_contract_upgrade_tx_hash = H256::zero(); // zero hash for now
        let log_output: LogProcessingOutput =
            self.process_l2_logs(new_batch, expected_system_contract_upgrade_tx_hash);

        let mut blob_commitments: [H256; MAX_NUMBER_OF_BLOBS];

        // TODO: considering pricing mode to be validium since we don't have access for s
        for i in U256::from(SystemLogKey::BlobOneHashKey)..=U256::from(SystemLogKey::BlobSixHashKey)
        {
            log_output.blob_hashes[i - U256::from(SystemLogKey::BlobOneHashKey)] = H256::zero();
        }
    }

    #[cfg(any(feature = "native"))]
    pub fn create_recursive_proof(
        &self,
        //previous_adapter_pi: AdapterPublicInputs,
        prev_adapter_proof: RiscZeroProof,
        new_rollup_proof: MockProof,
        new_rollup_pi: L1BatchWithMetadata,
    ) -> Result<RiscZeroProof, anyhow::Error> {
        use nexus_core::zkvm::traits::ZKVMProof;
        let prev_adapter_pi: AdapterPublicInputs = prev_adapter_proof.public_inputs()?;
        //prev_adapter_proof.verify(self.img_id);
        let check =
            Self::verify_continuity_and_proof(prev_adapter_pi, new_rollup_proof, new_rollup_pi)?;

        //Run elf and generate proof.
        unimplemented!()
    }
}
