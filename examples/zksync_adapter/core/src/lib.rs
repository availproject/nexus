use adapter_sdk::types::AdapterPublicInputs;
use anyhow::anyhow;
use ethers_core::abi::{self, token};
use hex;
use std::collections::VecDeque;
#[cfg(any(feature = "native"))]
use nexus_core::types::AccountState;
use nexus_core::types::InitAccount;
use nexus_core::types::H256 as NexusH256;
use nexus_core::types::{AppAccountId, StatementDigest};
#[cfg(any(feature = "native"))]
use nexus_core::types::Proof as NexusProof;
#[cfg(any(feature = "native-risc0"))]
use nexus_core::zkvm::risczero::{RiscZeroProof as Proof, RiscZeroProver as Prover};
use nexus_core::zkvm::traits::ZKVMEnv;
#[cfg(any(feature = "risc0"))]
use risc0_zkvm::{SegmentReceipt,SuccinctReceipt,ReceiptClaim};
#[cfg(any(feature = "risc0"))]
use risc0_zkvm::recursion::{join,lift};
#[cfg(any(feature = "native"))]
use nexus_core::zkvm::traits::{ZKVMProof, ZKVMProver};
use serde::{Deserialize, Serialize};
#[cfg(any(feature = "native"))]
use types::L1BatchNumber;
pub use zksync_basic_types::ethabi::{Bytes, Token};
use zksync_basic_types::{web3::keccak256, H256, U256, U64};
// use zksync_types::commitment::serialize_commitments;
// use zksync_types::commitment::serialize_commitments;
pub mod constants;
pub mod transcript;
pub mod types;
pub mod utils;
pub mod verifier;
pub mod prover;

//pub use zksync_types::commitment::L1BatchWithMetadata;
pub use crate::constants::{
    SystemLogKey, L2_LOG_ADDRESS_OFFSET, L2_LOG_KEY_OFFSET, L2_LOG_VALUE_OFFSET,
    L2_TO_L1_LOG_SERIALIZE_SIZE, MAX_NUMBER_OF_BLOBS, PUBDATA_COMMITMENT_SIZE, PUBLIC_INPUT_SHIFT,
    TOTAL_BLOBS_IN_COMMITMENT,
};
pub use crate::types::{
    CommitBatchInfo, H256Vec, L1BatchPassThroughData, L1BatchWithMetadata, LogProcessingOutput,
    ProofWithCommitmentAndL1BatchMetaData, ProofWithL1BatchMetaData, RootState,
};
use crate::verifier::ZksyncVerifier;

pub struct STF {
    img_id: [u32; 8],
    elf: Vec<u8>,
}

//TODO: Add generics for risczero types, so SP1 could be used as well.
impl STF {
    pub fn new(img_id: [u32; 8], elf: Vec<u8>) -> Self {
        Self { img_id, elf }
    }

    fn process_l2_logs(
        new_batch: CommitBatchInfo,
        expected_system_contract_upgrade_tx_hash: H256,
    ) -> Result<LogProcessingOutput, anyhow::Error> {
        let mut log_output = LogProcessingOutput::new();
        let emitted_l2_logs = new_batch.system_logs;

        for i in (0..emitted_l2_logs.len()).step_by(L2_TO_L1_LOG_SERIALIZE_SIZE) {
            let (log_sender, _) = utils::read_address(&emitted_l2_logs, i + L2_LOG_ADDRESS_OFFSET);
            let (log_key, _) = utils::read_uint256(&emitted_l2_logs, i + L2_LOG_KEY_OFFSET);
            let (log_value, _) = utils::read_bytes32(&emitted_l2_logs, i + L2_LOG_VALUE_OFFSET);

            if (log_key == U256::from(SystemLogKey::L2ToL1LogsTreeRootKey as u16)) {
                log_output.l2_logs_tree_root = log_value;
            } else if (log_key == U256::from(SystemLogKey::TotalL2ToL1PubdataKey as u16)) {
                log_output.pubdata_hash = log_value;
            } else if (log_key == U256::from(SystemLogKey::StateDiffHashKey as u16)) {
                log_output.state_diff_hash = log_value;
            } else if (log_key
                == U256::from(SystemLogKey::PackedBatchAndL2BlockTimestampKey as u16))
            {
                log_output.packed_batch_and_l2_block_timestamp =
                    U256::from(log_value.as_fixed_bytes());
            } else if (log_key == U256::from(SystemLogKey::PrevBatchHashKey as u16)) {
                log_output.previous_batch_hash = log_value;
            } else if (log_key == U256::from(SystemLogKey::ChainedPriorityTxnHashKey as u16)) {
                log_output.chained_priority_txs_hash = log_value.as_fixed_bytes().to_vec();
            } else if (log_key == U256::from(SystemLogKey::NumberOfLayer1TxsKey as u16)) {
                log_output.number_of_layer1_txs = U256::from(log_value.as_fixed_bytes());
            } else if (log_key >= U256::from(SystemLogKey::BlobOneHashKey as u16)
                && log_key <= U256::from(SystemLogKey::BlobSixHashKey as u16))
            {
                let blob_number = log_key - U256::from(SystemLogKey::BlobOneHashKey as u16);
                log_output.blob_hashes[blob_number.low_u64() as usize] = log_value;
            }
            // TODO: not implemented for now
            // else if(log_key == U256::from(SystemLogKey::ExpectedSystemContractUpgradeTxHashKey)) {}
            // else if(log_key > U256::from(SystemLogKey::ExpectedSystemContractUpgradeTxHashKey)) {}
        }

        Ok(log_output)
    }

    #[cfg(any(feature = "native", feature = "risc0", feature = "sp1"))]
    fn get_commit_batch_info(
        new_rollup_pi: L1BatchWithMetadata,
        pubdata_commitments: Vec<u8>,
    ) -> CommitBatchInfo {
        let commit_batch_info = CommitBatchInfo {
            batch_number: new_rollup_pi.header.number.0 as u64,
            timestamp: new_rollup_pi.header.timestamp,
            index_repeated_storage_changes: new_rollup_pi.metadata.rollup_last_leaf_index,
            new_state_root: new_rollup_pi.metadata.root_hash,
            number_of_layer1_txs: new_rollup_pi.header.l1_tx_count.into(),
            priority_operations_hash: new_rollup_pi.header.priority_ops_onchain_data_hash(),
            bootloader_heap_initial_contents_hash: new_rollup_pi
                .metadata
                .bootloader_initial_content_commitment
                .unwrap(),
            events_queue_state_hash: new_rollup_pi.metadata.events_queue_commitment.unwrap(),
            system_logs: crate::utils::serialize_commitments(&new_rollup_pi.header.system_logs),
            // new_rollup_pi.header.system_logs, need to serialize it somehow
            // TODO: need to confirm calculation
            pubdata_commitments,
        };
        commit_batch_info
    }

    fn batch_pass_through_data(batch: CommitBatchInfo) -> H256 {
        let pass_through_data = L1BatchPassThroughData {
            shared_states: vec![
                RootState {
                    last_leaf_index: batch.index_repeated_storage_changes,
                    root_hash: batch.new_state_root,
                },
                // Despite the fact that `zk_porter` is not available we have to add params about it.
                RootState {
                    last_leaf_index: 0,
                    root_hash: H256::zero(),
                },
            ],
        };
        let hash = pass_through_data.hash();
        return hash;
    }

    fn calculate_public_input(
        prev_batch_commitment: String,
        current_batch_commitment: String,
    ) -> U256 {
        let current_batch_commitment = U256::from_str_radix(&current_batch_commitment, 16).unwrap();
        let prev_batch_commitment = U256::from_str_radix(&prev_batch_commitment, 16).unwrap();
        let val = abi::encode_packed(&[
            Token::Uint(prev_batch_commitment),
            Token::Uint(current_batch_commitment),
        ])
        .unwrap();

        let public_input = U256::from_big_endian(&keccak256(&val)) >> PUBLIC_INPUT_SHIFT;
        public_input
    }

    pub fn verify_continuity_and_proof(
        previous_adapter_pi: AdapterPublicInputs,
        new_rollup_proof: Vec<String>,
        new_rollup_pi: L1BatchWithMetadata,
        commit_batch_info: CommitBatchInfo,
        pubdata_commitments: Vec<u8>,
        versioned_hashes: Vec<[u8; 32]>,
        nexus_hash: NexusH256,
    ) -> Result<AdapterPublicInputs, anyhow::Error> {

        // TODO: need to change
        let expected_system_contract_upgrade_tx_hash = H256::zero(); // zero hash for now
        let mut log_output: LogProcessingOutput = Self::process_l2_logs(
            commit_batch_info.clone(),
            expected_system_contract_upgrade_tx_hash,
        )
        .unwrap();

        // alternate way to calculate commitment
        let mut result = vec![];
        let pass_through_data_hash = Self::batch_pass_through_data(commit_batch_info);
        result.extend_from_slice(pass_through_data_hash.as_bytes());
        let metadata_hash = new_rollup_pi.metadata.meta_parameters_hash;
        result.extend_from_slice(metadata_hash.as_bytes());
        let auxiliary_output_hash = new_rollup_pi.metadata.aux_data_hash;
        result.extend_from_slice(auxiliary_output_hash.as_bytes());

        let hash = keccak256(&result);
        let current_batch_commitment = H256::from(hash);
        let current_batch_commitment_string =
            format!("0x{}", hex::encode(current_batch_commitment.as_bytes()));

        // TODO: uncomment this else further batch proving won't work
        let prev_batch_commitment_string = format!(
            "0x{}",
            hex::encode(previous_adapter_pi.rollup_hash.unwrap().as_fixed_slice())
        );

        // let genesis_batch_commitment = "0x2d00e5f8d77afcebf58a6b82ae56ba967566fe7dfbcb6760319fb0d215d18ffd".to_string();
        // let prev_batch_commitment_string = genesis_batch_commitment;

        let public_input = Self::calculate_public_input(
            prev_batch_commitment_string,
            current_batch_commitment_string,
        );

        let verifier = ZksyncVerifier::new();
        let is_proof_verified = verifier.verify(public_input.to_string(), new_rollup_proof);

        if (!is_proof_verified) {
            return Err(anyhow!("Proof verification failed"));
        }

        if new_rollup_pi.header.number.0 > 1 {
            // state root of current proof should be same as batch hash of previous batch
            if log_output.previous_batch_hash.as_fixed_bytes()
                != previous_adapter_pi.state_root.as_fixed_slice()
            {
                return Err(anyhow!("Previous batch hash does not match"));
            }
        };

        let proof_public_input = AdapterPublicInputs {
            nexus_hash,
            state_root: NexusH256::from(new_rollup_pi.metadata.root_hash.as_fixed_bytes().clone()),
            height: new_rollup_pi.header.number.0.into(),
            start_nexus_hash: previous_adapter_pi.start_nexus_hash,
            app_id: previous_adapter_pi.app_id,
            img_id: previous_adapter_pi.img_id,
            rollup_hash: Some(NexusH256::from(
                current_batch_commitment.as_fixed_bytes().clone(),
            )),
        };

        Ok(proof_public_input)
    }

    #[cfg(any(feature = "native", feature = "risc0", feature = "sp1"))]
    pub fn create_recursive_proof<
        Z: ZKVMProver<P>,
        P: ZKVMProof + Serialize + Clone + TryFrom<NexusProof>,
        E: ZKVMEnv,
    >(
        &self,
        //previous_adapter_pi: AdapterPublicInputs,
        mut prev_adapter_proof: Option<P>,
        init_account: Option<(AppAccountId, AccountState)>,
        new_rollup_proof: Vec<String>,
        new_rollup_pi: L1BatchWithMetadata,
        pubdata_commitments: Vec<u8>,
        versioned_hashes: Vec<[u8; 32]>,
        nexus_hash: NexusH256,
    ) -> Result<P, anyhow::Error>
    where
        <P as TryFrom<NexusProof>>::Error: std::fmt::Debug,
    {
        // genesis rollup hash

        use std::str::FromStr;
        let genesis_batch_commitment =
            H256::from_str("0x2d00e5f8d77afcebf58a6b82ae56ba967566fe7dfbcb6760319fb0d215d18ffd")
                .unwrap();

        use types::L1BatchNumber;
        let prev_adapter_pi: AdapterPublicInputs = match &mut prev_adapter_proof {
            Some(i) => i.public_inputs()?,
            None => {
                if new_rollup_pi.header.number == L1BatchNumber(1) {
                    match init_account {
                        Some(i) => AdapterPublicInputs {
                            start_nexus_hash: NexusH256::from(i.1.start_nexus_hash),
                            nexus_hash,
                            state_root: NexusH256::zero(),
                            height: 0,
                            app_id: i.0,
                            img_id: i.1.statement,
                            rollup_hash: Some(NexusH256::from(genesis_batch_commitment.as_fixed_bytes().clone())),
                        },
                        None => return Err(anyhow!("Init account details not provided which is required for first recursive proof")),
                    }
                } else {
                    return Err(anyhow!("Previous public inputs not provided, and it should be provided if not first recursive proof."));
                }
            }
        };

        // TODO: need to take the input batch
        let new_batch =
            Self::get_commit_batch_info(new_rollup_pi.clone(), pubdata_commitments.clone());

        let check = Self::verify_continuity_and_proof(
            prev_adapter_pi.clone(),
            new_rollup_proof.clone(),
            new_rollup_pi.clone(),
            new_batch.clone(),
            pubdata_commitments.clone(),
            versioned_hashes.clone(),
            nexus_hash.clone(),
        )?;

        let mut prover = Z::new(self.elf.clone());

        prover.add_input(&prev_adapter_pi)?;
        prover.add_input(&new_rollup_proof)?;
        prover.add_input(&new_rollup_pi)?;
        prover.add_input(&self.img_id)?;
        prover.add_input(&new_batch)?;
        prover.add_input(&pubdata_commitments.clone())?;
        prover.add_input(&versioned_hashes.clone())?;
        prover.add_input(&nexus_hash)?;
        
        // TODO: Need to write a program for add proof for recursion
        // #[cfg(feature = "risc0")]
        // match prev_adapter_proof.clone() {
        //     Some(i) => prover.add_proof_for_recursion(i)?,
        //     None => (),
        // };
        
        let proof = prover.prove();  

        if cfg!(any(feature = "sp1")) {
            proof
        }else if cfg!(any(feature = "risc0")) { 
            // let proof = proof.unwrap();
            let segments_1: Vec<SegmentReceipt>;
             match proof {
                    Ok(proof) => {
                    segments_1 = proof.get_segments()?;
                },
                Err(_) => {
                    segments_1 = Vec::new();
                }
            }
            
            let segments_2: Vec<SegmentReceipt> = match prev_adapter_proof.clone() {
                Some(ref result) => result.get_segments()?, 
                None => Vec::new(), 
            };
            let mut succinct_receipts_vec : Vec<SuccinctReceipt<ReceiptClaim>> = Vec::new();
            for segment in segments_1 {
                succinct_receipts_vec.push(lift(&segment)?);
            }
            for segment in segments_2 {
                succinct_receipts_vec.push(lift(&segment)?);
            }
            let mut queue = VecDeque::from(succinct_receipts_vec);
            while queue.len() > 1 {
                let first = queue.pop_front().unwrap();
                let second = queue.pop_front().unwrap();
                let succinct_receipt = join(&first,&second); 
            }
           let aggregated_receipt : SuccinctReceipt<ReceiptClaim> = queue.pop_front().unwrap();
           //need to add how to handle after this receipt is generated since now sp1 and risc0 are generating different types of proofs
        }

    }
}