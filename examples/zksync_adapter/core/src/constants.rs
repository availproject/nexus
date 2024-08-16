use zksync_basic_types::Address;

pub const MAX_NUMBER_OF_BLOBS: usize = 6;
pub const L2_TO_L1_LOG_SERIALIZE_SIZE: usize = 88;

pub const L2_LOG_ADDRESS_OFFSET: usize = 4;
pub const L2_LOG_KEY_OFFSET: usize = 24;
pub const L2_LOG_VALUE_OFFSET: usize = 56;

pub const PUBDATA_COMMITMENT_SIZE: usize = 144;
pub const TOTAL_BLOBS_IN_COMMITMENT: usize = 16;

pub enum SystemLogKey {
    L2ToL1LogsTreeRootKey,
    TotalL2ToL1PubdataKey,
    StateDiffHashKey,
    PackedBatchAndL2BlockTimestampKey,
    PrevBatchHashKey,
    ChainedPriorityTxnHashKey,
    NumberOfLayer1TxsKey,
    BlobOneHashKey,
    BlobTwoHashKey,
    BlobThreeHashKey,
    BlobFourHashKey,
    BlobFiveHashKey,
    BlobSixHashKey,
    ExpectedSystemContractUpgradeTxHashKey,
}