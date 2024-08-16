use nexus_core::types::H256;
use zksync_types::{ethabi::Bytes, U256, U64};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LogProcessingOutput {
    pub number_of_layer1_txs: U256,
    pub chained_priority_txs_hash: bytes,
    pub previous_batch_hash: H256,
    pub pubdata_hash: H256,
    pub state_diff_hash: H256,
    pub l2_logs_tree_root: H256,
    pub packed_batch_and_l2_block_timestamp: U256,
    pub blob_hashes: Vec<H256>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]

pub struct CommitBatchInfo {
    pub batch_number: U64,
    pub timestamp: U64,
    pub index_repeated_storage_changes: U64,
    pub new_state_root: H256,
    pub number_of_layer1_txs: U256,
    pub priority_operations_hash: H256,
    pub bootloader_heap_initial_contents_hash: H256,
    pub events_queue_state_hash: H256,
    pub system_logs: Bytes,
    pub pubdata_commitments: Bytes,
}

