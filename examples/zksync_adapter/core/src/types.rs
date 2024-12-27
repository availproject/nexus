use crate::constants::MAX_NUMBER_OF_BLOBS;
use substrate_bn::{Fr,Fq,AffineG1,AffineG2};
use serde::{Deserialize, Serialize};
use zksync_basic_types::{
    ethabi::ethereum_types::Bloom as H2048, ethabi::Bytes, protocol_version::ProtocolVersionId,
    web3::keccak256, Address, H160, H256, U256, ethabi::Token
};
#[cfg(any(feature = "native"))]
use zksync_types::commitment::SerializeCommitment;

pub type G1Point = AffineG1;
pub type G2Point = AffineG2;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LogProcessingOutput {
    pub number_of_layer1_txs: U256,
    pub chained_priority_txs_hash: Bytes,
    pub previous_batch_hash: H256,
    pub pubdata_hash: H256,
    pub state_diff_hash: H256,
    pub l2_logs_tree_root: H256,
    pub packed_batch_and_l2_block_timestamp: U256,
    pub blob_hashes: [H256; MAX_NUMBER_OF_BLOBS as usize],
}

pub type H256Vec = [H256; MAX_NUMBER_OF_BLOBS as usize];

impl LogProcessingOutput {
    pub fn new() -> LogProcessingOutput {
        Self {
            number_of_layer1_txs: U256::zero(),
            chained_priority_txs_hash: vec![],
            previous_batch_hash: H256::zero(),
            pubdata_hash: H256::zero(),
            state_diff_hash: H256::zero(),
            l2_logs_tree_root: H256::zero(),
            packed_batch_and_l2_block_timestamp: U256::zero(),
            blob_hashes: [H256::zero(); MAX_NUMBER_OF_BLOBS as usize],
        }
    }
}
#[derive(Clone, Debug, Deserialize, Serialize)]

pub struct CommitBatchInfo {
    pub batch_number: u64,
    pub timestamp: u64,
    pub index_repeated_storage_changes: u64,
    pub new_state_root: H256,
    pub number_of_layer1_txs: U256,
    pub priority_operations_hash: H256,
    pub bootloader_heap_initial_contents_hash: H256,
    pub events_queue_state_hash: H256,
    pub system_logs: Bytes,
    pub pubdata_commitments: Bytes,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProofWithL1BatchMetaData {
    pub bytes: Token,
    pub metadata: L1BatchWithMetadata,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProofWithCommitmentAndL1BatchMetaData {
    pub proof_with_l1_batch_metadata: ProofWithL1BatchMetaData,
    pub blob_commitments: Vec<H256>,
    pub pubdata_commitments: Vec<u8>,
    pub versioned_hashes: Vec<[u8; 32]>
}


#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct L1BatchWithMetadata {
    pub header: L1BatchHeader,
    pub metadata: L1BatchMetadata,
    pub raw_published_factory_deps: Vec<Vec<u8>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct L1BatchHeader {
    /// Numeric ID of the block. Starts from 1, 0 block is considered genesis block and has no transactions.
    pub number: L1BatchNumber,
    /// Timestamp when block was first created.
    pub timestamp: u64,
    /// Total number of processed priority operations in the block
    pub l1_tx_count: u16,
    /// Total number of processed txs that was requested offchain
    pub l2_tx_count: u16,
    /// The data of the processed priority operations hash which must be sent to the smart contract.
    pub priority_ops_onchain_data: Vec<PriorityOpOnchainData>,
    /// All user generated L2 -> L1 logs in the block.
    pub l2_to_l1_logs: Vec<UserL2ToL1Log>,
    /// Preimages of the hashes that were sent as value of L2 logs by special system L2 contract.
    pub l2_to_l1_messages: Vec<Vec<u8>>,
    /// Bloom filter for the event logs in the block.
    pub bloom: H2048,
    /// Hashes of contracts used this block
    pub used_contract_hashes: Vec<U256>,
    pub base_system_contracts_hashes: BaseSystemContractsHashes,
    /// System logs are those emitted as part of the Vm execution.
    pub system_logs: Vec<SystemL2ToL1Log>,
    /// Version of protocol used for the L1 batch.
    pub protocol_version: Option<ProtocolVersionId>,
    pub pubdata_input: Option<Vec<u8>>,
}

impl L1BatchHeader {
    pub fn new(
        number: L1BatchNumber,
        timestamp: u64,
        base_system_contracts_hashes: BaseSystemContractsHashes,
        protocol_version: ProtocolVersionId,
    ) -> L1BatchHeader {
        Self {
            number,
            timestamp,
            l1_tx_count: 0,
            l2_tx_count: 0,
            priority_ops_onchain_data: vec![],
            l2_to_l1_logs: vec![],
            l2_to_l1_messages: vec![],
            bloom: H2048::default(),
            used_contract_hashes: vec![],
            base_system_contracts_hashes,
            system_logs: vec![],
            protocol_version: Some(protocol_version),
            pubdata_input: Some(vec![]),
        }
    }

    /// Creates a hash of the priority ops data.
    pub fn priority_ops_onchain_data_hash(&self) -> H256 {
        let mut rolling_hash: H256 = keccak256(&[]).into();
        for onchain_data in &self.priority_ops_onchain_data {
            let mut preimage = Vec::new();
            preimage.extend(rolling_hash.as_fixed_bytes());
            preimage.extend(onchain_data.onchain_data_hash.as_fixed_bytes());

            rolling_hash = keccak256(&preimage).into();
        }

        rolling_hash
    }

    pub fn tx_count(&self) -> usize {
        (self.l1_tx_count + self.l2_tx_count) as usize
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct L1BatchMetadata {
    pub root_hash: H256,
    pub rollup_last_leaf_index: u64,
    pub initial_writes_compressed: Option<Vec<u8>>,
    pub repeated_writes_compressed: Option<Vec<u8>>,
    pub commitment: H256,
    pub l2_l1_merkle_root: H256,
    pub block_meta_params: L1BatchMetaParameters,
    pub aux_data_hash: H256,
    pub meta_parameters_hash: H256,
    pub pass_through_data_hash: H256,
    /// The commitment to the final events queue state after the batch is committed.
    /// Practically, it is a commitment to all events that happened on L2 during the batch execution.
    pub events_queue_commitment: Option<H256>,
    /// The commitment to the initial heap content of the bootloader. Practically it serves as a
    /// commitment to the transactions in the batch.
    pub bootloader_initial_content_commitment: Option<H256>,
    pub state_diffs_compressed: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct L1BatchMetaParameters {
    pub zkporter_is_available: bool,
    pub bootloader_code_hash: H256,
    pub default_aa_code_hash: H256,
    pub protocol_version: Option<ProtocolVersionId>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct L1BatchNumber(pub u32);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PriorityOpOnchainData {
    pub layer_2_tip_fee: U256,
    pub onchain_data_hash: H256,
}

// #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
// pub struct U256(pub [u64; 4]);

// #[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default, Eq)]
// pub struct H256(pub [u8; 32]);

// #[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default, Eq)]
// pub struct H160(pub [u8; 20]);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default, Eq)]
pub struct UserL2ToL1Log(pub L2ToL1Log);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default, Eq)]
pub struct L2ToL1Log {
    pub shard_id: u8,
    pub is_service: bool,
    pub tx_number_in_block: u16,
    pub sender: Address,
    pub key: H256,
    pub value: H256,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq)]
pub struct BaseSystemContractsHashes {
    pub bootloader: H256,
    pub default_aa: H256,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default, Eq)]
pub struct SystemL2ToL1Log(pub L2ToL1Log);

#[cfg(any(feature = "native"))]
impl SerializeCommitment for L2ToL1Log {
    const SERIALIZED_SIZE: usize = 88;

    fn serialize_commitment(&self, buffer: &mut [u8]) {
        buffer[0] = self.shard_id;
        buffer[1] = self.is_service as u8;
        buffer[2..4].copy_from_slice(&self.tx_number_in_block.to_be_bytes());
        buffer[4..24].copy_from_slice(self.sender.as_bytes());
        buffer[24..56].copy_from_slice(self.key.as_bytes());
        buffer[56..88].copy_from_slice(self.value.as_bytes());
    }
}

#[cfg(any(feature = "native"))]
impl SerializeCommitment for UserL2ToL1Log {
    const SERIALIZED_SIZE: usize = L2ToL1Log::SERIALIZED_SIZE;

    fn serialize_commitment(&self, buffer: &mut [u8]) {
        self.0.serialize_commitment(buffer);
    }
}

#[cfg(any(feature = "native"))]
impl SerializeCommitment for SystemL2ToL1Log {
    const SERIALIZED_SIZE: usize = L2ToL1Log::SERIALIZED_SIZE;

    fn serialize_commitment(&self, buffer: &mut [u8]) {
        self.0.serialize_commitment(buffer);
    }
}

#[derive(Debug, Clone)]
pub struct Proof {
    pub state_poly_0: G1Point,
    pub state_poly_1: G1Point,
    pub state_poly_2: G1Point,
    pub state_poly_3: G1Point,

    pub copy_permutation_grand_product: G1Point,

    pub lookup_s_poly: G1Point,
    pub lookup_grand_product: G1Point,

    pub quotient_poly_parts_0: G1Point,
    pub quotient_poly_parts_1: G1Point,
    pub quotient_poly_parts_2: G1Point,
    pub quotient_poly_parts_3: G1Point,

    pub state_poly_0_opening_at_z: Fr,
    pub state_poly_1_opening_at_z: Fr,
    pub state_poly_2_opening_at_z: Fr,
    pub state_poly_3_opening_at_z: Fr,

    pub state_poly_3_opening_at_z_omega: Fr,
    pub gate_selectors_0_opening_at_z: Fr,

    pub copy_permutation_polys_0_opening_at_z: Fr,
    pub copy_permutation_polys_1_opening_at_z: Fr,
    pub copy_permutation_polys_2_opening_at_z: Fr,

    pub copy_permutation_grand_product_opening_at_z_omega: Fr,
    pub lookup_s_poly_opening_at_z_omega: Fr,
    pub lookup_grand_product_opening_at_z_omega: Fr,
    pub lookup_t_poly_opening_at_z: Fr,
    pub lookup_t_poly_opening_at_z_omega: Fr,
    pub lookup_selector_poly_opening_at_z: Fr,
    pub lookup_table_type_poly_opening_at_z: Fr,
    pub quotient_poly_opening_at_z: Fr,
    pub linearisation_poly_opening_at_z: Fr,

    pub opening_proof_at_z: G1Point,
    pub opening_proof_at_z_omega: G1Point,
}

#[derive(Debug, Clone)]
pub struct ProofWithPubSignal {
    pub proof: Proof,
    pub pub_signal: Fr,
}

#[derive(Debug, Clone)]
pub struct PartialVerifierState {
    pub alpha: Fr,
    pub beta: Fr,
    pub gamma: Fr,
    pub power_of_alpha_2: Fr,
    pub power_of_alpha_3: Fr,
    pub power_of_alpha_4: Fr,
    pub power_of_alpha_5: Fr,
    pub power_of_alpha_6: Fr,
    pub power_of_alpha_7: Fr,
    pub power_of_alpha_8: Fr,
    pub eta: Fr,
    pub beta_lookup: Fr,
    pub gamma_lookup: Fr,
    pub beta_plus_one: Fr,
    pub beta_gamma_plus_gamma: Fr,
    pub v: Fr,
    pub u: Fr,
    pub z: Fr,
    pub z_minus_last_omega: Fr,
    pub l_0_at_z: Fr,
    pub l_n_minus_one_at_z: Fr,
    pub z_in_domain_size: Fr,
}

impl PartialVerifierState {
    pub fn new() -> Self {
        PartialVerifierState {
            alpha: Fr::zero(),
            beta: Fr::zero(),
            gamma: Fr::zero(),
            power_of_alpha_2: Fr::zero(),
            power_of_alpha_3: Fr::zero(),
            power_of_alpha_4: Fr::zero(),
            power_of_alpha_5: Fr::zero(),
            power_of_alpha_6: Fr::zero(),
            power_of_alpha_7: Fr::zero(),
            power_of_alpha_8: Fr::zero(),
            eta: Fr::zero(),
            beta_lookup: Fr::zero(),
            gamma_lookup: Fr::zero(),
            beta_plus_one: Fr::zero(),
            beta_gamma_plus_gamma: Fr::zero(),
            v: Fr::zero(),
            u: Fr::zero(),
            z: Fr::zero(),
            z_minus_last_omega: Fr::zero(),
            l_0_at_z: Fr::zero(),
            l_n_minus_one_at_z: Fr::zero(),
            z_in_domain_size: Fr::zero(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct VerificationKey {
    pub gate_setup: Vec<G1Point>,
    pub gate_selectors: Vec<G1Point>,
    pub permutation: Vec<G1Point>,
    pub lookup_table: Vec<G1Point>,
    pub lookup_selector: G1Point,
    pub lookup_table_type: G1Point,
    pub recursive_flag: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RootState {
    pub last_leaf_index: u64,
    pub root_hash: H256,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct L1BatchPassThroughData {
    pub shared_states: Vec<RootState>,
}

impl L1BatchPassThroughData {
    pub fn to_bytes(&self) -> Vec<u8> {
        // We assume that currently we have only two shared state: Rollup and ZkPorter where porter is always zero
        const SERIALIZED_SIZE: usize = 8 + 32 + 8 + 32;
        let mut result = Vec::with_capacity(SERIALIZED_SIZE);
        for state in self.shared_states.iter() {
            result.extend_from_slice(&state.last_leaf_index.to_be_bytes());
            result.extend_from_slice(state.root_hash.as_bytes());
        }
        assert_eq!(
            result.len(),
            SERIALIZED_SIZE,
            "Serialized size for BlockPassThroughData is bigger than expected"
        );
        result
    }

    pub fn hash(&self) -> H256 {
        H256::from_slice(&keccak256(&self.to_bytes()))
    }
}