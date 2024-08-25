use adapter_sdk::types::AdapterPublicInputs;
use anyhow::anyhow;
use ethers_core::abi::{self, token};
use hex;
use nexus_core::types::InitAccount;
use nexus_core::types::H256 as NexusH256;
use nexus_core::types::{AppAccountId, RollupPublicInputsV2, StatementDigest};
#[cfg(any(feature = "native"))]
use nexus_core::zkvm::{
    risczero::{RiscZeroProof, RiscZeroProver},
    traits::{ZKVMProof, ZKVMProver},
};
use serde::{Deserialize, Serialize};
use zksync_basic_types::{
    ethabi::{Bytes, Token},
    web3::keccak256,
    H256, U256, U64,
};
// use zksync_types::commitment::serialize_commitments;
// use zksync_types::commitment::serialize_commitments;
pub mod constants;
pub mod transcript;
pub mod types;
pub mod utils;
pub mod verifier;
//pub use zksync_types::commitment::L1BatchWithMetadata;
pub use crate::constants::{
    SystemLogKey, L2_LOG_ADDRESS_OFFSET, L2_LOG_KEY_OFFSET, L2_LOG_VALUE_OFFSET,
    L2_TO_L1_LOG_SERIALIZE_SIZE, MAX_NUMBER_OF_BLOBS, PUBDATA_COMMITMENT_SIZE,
    TOTAL_BLOBS_IN_COMMITMENT,
};
pub use crate::types::{CommitBatchInfo, H256Vec, L1BatchWithMetadata, LogProcessingOutput};
use crate::verifier::ZksyncVerifier;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MockProof(pub ());

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
        println!("Emitted logs length: {}", emitted_l2_logs.len());

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

        println!("Log output: {:?}", log_output);

        Ok(log_output)
    }

    #[cfg(any(feature = "native"))]
    fn get_commit_batch_info(new_rollup_pi: L1BatchWithMetadata) -> CommitBatchInfo {
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
            pubdata_commitments: [0u8; 32].to_vec(), // new_rollup_pi.header.pubdata_input.unwrap(),
        };
        commit_batch_info
    }

    fn get_mock_commit_batch_info() -> CommitBatchInfo {
        let mut new_state_root_array = [0u8; 32];
        let mut priority_operations_hash_array = [0u8; 32];
        let mut bootloader_heap_initial_contents_hash_array = [0u8; 32];
        let mut events_queue_state_hash_array = [0u8; 32];

        new_state_root_array.copy_from_slice(
            hex::decode("0x0345c077e53d9fe567c519ccc03d938a699257b562378d557c28bd828f949f2e")
                .unwrap()
                .as_slice(),
        );

        priority_operations_hash_array.copy_from_slice(
            hex::decode("0xd179d9ca318bc891ff8fb7f7f7c8fc5ce420c7f7a715baf5dde385c557d75f27")
                .unwrap()
                .as_slice(),
        );

        bootloader_heap_initial_contents_hash_array.copy_from_slice(
            hex::decode("0x65e138992d67d68d760f784cecafd6a6ae31d124b4aab4d33e1a373756e427d1")
                .unwrap()
                .as_slice(),
        );

        events_queue_state_hash_array.copy_from_slice(
            hex::decode("0x626fb1e4d667d0ea712d56013a9301ed9155de9acc6bce05ed28e972e6fe3b2b")
                .unwrap()
                .as_slice(),
        );

        let commit_batch_info = CommitBatchInfo {
            batch_number: 490903 as u64,
            timestamp: 1723815968 as u64,
            index_repeated_storage_changes: 354452643 as u64,
            new_state_root: H256::from(new_state_root_array),
            number_of_layer1_txs: U256::from(1u32),
            priority_operations_hash: H256::from(priority_operations_hash_array),
            bootloader_heap_initial_contents_hash: H256::from(bootloader_heap_initial_contents_hash_array),
            events_queue_state_hash: H256::from(events_queue_state_hash_array),
            system_logs: hex::decode("00000000000000000000000000000000000000000000800b000000000000000000000000000000000000000000000000000000000000000438579039c2dd55d0f47ef37bb27c00c6413902c4df25f72e4a6eb8d7298e6f850000103a000000000000000000000000000000000000800b000000000000000000000000000000000000000000000000000000000000000300000000000000000000000066bf582000000000000000000000000066bf5bd40001103a00000000000000000000000000000000000080010000000000000000000000000000000000000000000000000000000000000005d179d9ca318bc891ff8fb7f7f7c8fc5ce420c7f7a715baf5dde385c557d75f270001103a0000000000000000000000000000000000008001000000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000010001103a0000000000000000000000000000000000008011000000000000000000000000000000000000000000000000000000000000000720a1eb4f1d0f7809a2e26e3348eed0f1e436c51c65f9802e4cbf4848ac1181920001103a00000000000000000000000000000000000080110000000000000000000000000000000000000000000000000000000000000008a1fcb4c0d53715348ed06904f3ff66bce8f01075babb3e6639738350092305250001103a00000000000000000000000000000000000080110000000000000000000000000000000000000000000000000000000000000009f61f17b104e64b51383b2cee3e7220de49cce6178fd0bfcb2f4959d1689340840001103a0000000000000000000000000000000000008011000000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000000001103a0000000000000000000000000000000000008011000000000000000000000000000000000000000000000000000000000000000b00000000000000000000000000000000000000000000000000000000000000000001103a0000000000000000000000000000000000008011000000000000000000000000000000000000000000000000000000000000000c00000000000000000000000000000000000000000000000000000000000000000001103a000000000000000000000000000000000000800800000000000000000000000000000000000000000000000000000000000000003f88682da9e5aa2df57fd428955f311a86e78e88811993d780ea0ea82628a8b20001103a0000000000000000000000000000000000008008000000000000000000000000000000000000000000000000000000000000000121ad7e51e756f53b0565221908defc8b2d307e3a68d2f18ccd6f506ffd100fde0001103a00000000000000000000000000000000000080080000000000000000000000000000000000000000000000000000000000000002bb9a8f77b09f24cef7fb173608c60766b42d7bb52c1ef929c2b2617191fe9191").unwrap(),
            pubdata_commitments: hex::decode("01baefa19e5387e5951883734381544dac043cc6e9bb8e16c94b73c4e51e4c25f914af9c810c62d631364aa2e9f5b751a486d45d1f588a4a3d155c69206763d936310bbfe50962fa7496bceddc74e54da89d2919ea7a83af20e29c5311a69db41dae8c3f7ebae0ac5946cc2b97a982c011c5b6c6dd217cf4a66d93c70c7eff9f7faa62621671b589ae4bd1877879ce089ecfb37219d328c5a436db67ec2a64a0ab4f56914dc2718c031725fe3c12da33b629a6125ee4ded159c026177bb7758072b227e30107a9fc2be1acda2b593ac6713d125ea8c746c99107a8a42b8d9d7f03be7c5227aaeb5fb9de9d008bde09fd318312640618e5b66764e314e2aa4baa851850650ed1458ee43aeee04e655a4e3d1f360e5da777cbaee2e12e8cb88747e2729d42115cda47f77fce5ab04fd71e444a60a14695937a7bf29649e9f003f58c957473eb1e5ddede8c1fe791835b89aba0117e48aeef3aa5d0ffce5144d8df84c3854221eab82ab1bbe2351227b880b7d0b755dbeca4a59d7e7cb43b6394386286e70997e8df3424fb98e0b7e4cfab1f2797851c07e0f677fd09a7b9721480f4f5d9a2c4510970249395ec635ef7eb8a").unwrap(),
        };
        commit_batch_info
    }

    fn batch_pass_through_data(batch: CommitBatchInfo) -> Bytes {
        abi::encode_packed(&[
            Token::Uint(U256::from(batch.index_repeated_storage_changes)),
            Token::FixedBytes(batch.new_state_root.as_fixed_bytes().to_vec()),
            Token::Uint(U256::zero()),
            Token::FixedBytes(H256::zero().as_fixed_bytes().to_vec()),
        ])
        .unwrap()
    }

    fn batch_meta_parameters() -> Bytes {
        // TODO: hardcoding for now
        let mut l2_default_account_bytecode_hash_byte_array = [0u8; 32];
        l2_default_account_bytecode_hash_byte_array.copy_from_slice(
            hex::decode("01000563374c277a2c1e34659a2a1e87371bb6d852ce142022d497bfb50b9e32")
                .expect("Failed to decode hex string")
                .as_slice(),
        );
        let l2_default_account_bytecode_hash =
            H256::from(l2_default_account_bytecode_hash_byte_array);
        let s_zk_porter_is_avalaible = false;
        let mut s_l2_boot_loader_bytecode_hash_byte_array = [0u8; 32];
        s_l2_boot_loader_bytecode_hash_byte_array.copy_from_slice(
            hex::decode("010008e742608b21bf7eb23c1a9d0602047e3618b464c9b59c0fba3b3d7ab66e")
                .expect("Failed to decode hex string")
                .as_slice(),
        );
        let s_l2_boot_loader_bytecode_hash = H256::from(s_l2_boot_loader_bytecode_hash_byte_array);

        abi::encode_packed(&[
            Token::FixedBytes(l2_default_account_bytecode_hash.as_fixed_bytes().to_vec()),
            Token::Bool(s_zk_porter_is_avalaible),
            Token::FixedBytes(s_l2_boot_loader_bytecode_hash.as_fixed_bytes().to_vec()),
        ])
        .unwrap()
    }

    fn encode_blob_auxillary_output(
        blob_commitments: H256Vec,
        blob_hashes: H256Vec,
    ) -> [H256; 2 * TOTAL_BLOBS_IN_COMMITMENT] {
        let mut blob_aux_output_words: [H256; 2 * TOTAL_BLOBS_IN_COMMITMENT] =
            [H256::zero(); 2 * TOTAL_BLOBS_IN_COMMITMENT];

        for i in 0..MAX_NUMBER_OF_BLOBS {
            blob_aux_output_words[2 * i] = blob_hashes[i];
            blob_aux_output_words[2 * i + 1] = blob_commitments[i];
        }

        blob_aux_output_words
    }

    fn batch_auxillary_output(
        batch: CommitBatchInfo,
        state_diff_hash: H256,
        blob_commitments: H256Vec,
        blob_hashes: H256Vec,
    ) -> Bytes {
        let l2_to_l1_log_hash = keccak256(&batch.system_logs);
        let mut tokens = vec![
            Token::FixedBytes(l2_to_l1_log_hash.to_vec()),
            Token::FixedBytes(state_diff_hash.as_fixed_bytes().to_vec()),
            Token::FixedBytes(
                batch
                    .bootloader_heap_initial_contents_hash
                    .as_fixed_bytes()
                    .to_vec(),
            ),
            Token::FixedBytes(batch.events_queue_state_hash.as_fixed_bytes().to_vec()),
        ];
        tokens.extend(
            Self::encode_blob_auxillary_output(blob_commitments, blob_hashes)
                .map(|x| Token::FixedBytes(x.as_fixed_bytes().to_vec())),
        );

        abi::encode_packed(&tokens).unwrap()
    }

    fn create_batch_commitment(
        new_batch: CommitBatchInfo,
        state_diff_hash: H256,
        blob_commitments: H256Vec,
        blob_hashes: H256Vec,
    ) -> H256 {
        let pass_through_data = Self::batch_pass_through_data(new_batch.clone());
        let pass_through_data_hash = keccak256(&pass_through_data);
        let meta_data = Self::batch_meta_parameters();
        let meta_data_hash = keccak256(&meta_data);
        let batch_aux_output = Self::batch_auxillary_output(
            new_batch.clone(),
            state_diff_hash,
            blob_commitments,
            blob_hashes,
        );
        let batch_aux_output_hash = keccak256(&batch_aux_output);
        let encoded_hashes = abi::encode(&[
            Token::FixedBytes(pass_through_data_hash.to_vec()),
            Token::FixedBytes(meta_data_hash.to_vec()),
            Token::FixedBytes(batch_aux_output_hash.to_vec()),
        ]);

        H256::from(keccak256(&encoded_hashes))
    }

    // used in case of blobs which is currently used
    fn verify_blob_information(
        pub_data_commitments: Vec<u8>,
        blob_hashes: H256Vec,
        blob_commitments: &mut H256Vec,
    ) {
        // hardcoding blobhash versioned hashes for now
        let (mut versioned_hash_1, mut versioned_hash_2, mut versioned_hash_3) =
            ([0u8; 32], [0u8; 32], [0u8; 32]);

        versioned_hash_1.copy_from_slice(
            hex::decode("0x0161d816a23b802acea49bd7d7454e0199f397111aca36072ad394c95944cd56")
                .expect("Failed to decode hex string")
                .as_slice(),
        );

        versioned_hash_2.copy_from_slice(
            hex::decode("0x012c23680e6d501dd4ce10ad3fe0b64963985983ceb816cc6457e002e5b84274")
                .expect("Failed to decode hex string")
                .as_slice(),
        );
        versioned_hash_3.copy_from_slice(
            hex::decode("0x016cd3477770892305d2e62a13f19fc18581fe0972c66ea4cc10b782c5611ec8")
                .expect("Failed to decode hex string")
                .as_slice(),
        );

        let versioned_hashes = [
            H256::from(versioned_hash_1),
            H256::from(versioned_hash_2),
            H256::from(versioned_hash_3),
        ];

        let mut version_hash_index = 0;
        for i in (0..pub_data_commitments.len()).step_by(PUBDATA_COMMITMENT_SIZE) {
            // TODO: skipping requires

            let mut encoded_hashes = vec![
                Token::FixedBytes(
                    versioned_hashes[version_hash_index]
                        .as_fixed_bytes()
                        .to_vec(),
                ),
                Token::FixedBytes(pub_data_commitments[i..i + PUBDATA_COMMITMENT_SIZE].to_vec()),
            ];

            let encoded_hashes = abi::encode_packed(&encoded_hashes).unwrap();
            blob_commitments[version_hash_index] = H256::from(keccak256(&encoded_hashes));

            version_hash_index += 1;
        }
    }

    pub fn verify_continuity_and_proof(
        previous_adapter_pi: AdapterPublicInputs,
        new_rollup_proof: MockProof,
        new_rollup_pi: L1BatchWithMetadata,
        commit_batch_info: CommitBatchInfo,
        nexus_hash: NexusH256,
    ) -> Result<AdapterPublicInputs, anyhow::Error> {
        let new_batch = Self::get_mock_commit_batch_info();

        // TODO: need to change
        let expected_system_contract_upgrade_tx_hash = H256::zero(); // zero hash for now
        let mut log_output: LogProcessingOutput = Self::process_l2_logs(
            commit_batch_info.clone(),
            expected_system_contract_upgrade_tx_hash,
        )
        .unwrap();

        let mut blob_commitments: H256Vec = [H256::zero(); MAX_NUMBER_OF_BLOBS];

        // when pub data source is selected as blob which is default
        Self::verify_blob_information(
            new_batch.pubdata_commitments.clone(),
            log_output.blob_hashes,
            &mut blob_commitments,
        );

        // In case when pricing mode is validium
        // let start = U256::from(SystemLogKey::BlobOneHashKey as u16).low_u32() as usize;
        // let end = U256::from(SystemLogKey::BlobSixHashKey as u16).low_u32() as usize;

        // for i in start..=end {
        //     log_output.blob_hashes
        //         [i - (U256::from(SystemLogKey::BlobOneHashKey as u16).low_u64() as usize)] =
        //         H256::zero();
        // }

        let commitment: H256 = Self::create_batch_commitment(
            commit_batch_info.clone(),
            log_output.state_diff_hash,
            blob_commitments,
            log_output.blob_hashes,
        );

        let mut public_inputs = previous_adapter_pi.clone();

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
        };

        Ok(proof_public_input)
    }

    // pub fn verify_mock_zksync_proof() {
    //     let verifier = ZksyncVerifier::new();
    // }

    #[cfg(any(feature = "native"))]
    pub fn create_recursive_proof(
        &self,
        //previous_adapter_pi: AdapterPublicInputs,
        prev_adapter_proof: Option<RiscZeroProof>,
        init_account: Option<InitAccount>,
        new_rollup_proof: MockProof,
        new_rollup_pi: L1BatchWithMetadata,
        nexus_hash: NexusH256,
    ) -> Result<RiscZeroProof, anyhow::Error> {
        use types::L1BatchNumber;

        let prev_adapter_pi: AdapterPublicInputs = match &prev_adapter_proof {
            Some(i) => i.public_inputs()?,
            None => {
                if new_rollup_pi.header.number == L1BatchNumber(1) {
                    match init_account {
                        Some(i) => AdapterPublicInputs {
                            start_nexus_hash: i.start_nexus_hash,
                            nexus_hash,
                            state_root: NexusH256::zero(),
                            height: 0,
                            app_id: i.app_id,
                            img_id: i.statement,
                        },
                        None => return Err(anyhow!("Init account details not provided which is required for first recursive proof")),
                    }
                } else {
                    return Err(anyhow!("Previous public inputs not provided, and it should be provided if not first recursive proof."));
                }
            }
        };

        // TODO: need to take the input batch
        let new_batch = Self::get_commit_batch_info(new_rollup_pi.clone());

        //prev_adapter_proof.verify(self.img_id);
        let check = Self::verify_continuity_and_proof(
            prev_adapter_pi.clone(),
            new_rollup_proof.clone(),
            new_rollup_pi.clone(),
            new_batch.clone(),
            nexus_hash.clone(),
        )?;

        let mut prover: RiscZeroProver = RiscZeroProver::new(self.elf.clone());

        prover.add_input(&prev_adapter_pi)?;
        prover.add_input(&new_rollup_proof)?;
        prover.add_input(&new_rollup_pi)?;
        prover.add_input(&self.img_id)?;
        prover.add_input(&new_batch)?;
        prover.add_input(&nexus_hash)?;
        match prev_adapter_proof {
            Some(i) => prover.add_proof_for_recursion(i)?,
            None => (),
        };

        prover.prove()
    }
}
