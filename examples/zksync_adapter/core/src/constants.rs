use zksync_types::U256;

pub const MAX_NUMBER_OF_BLOBS: U256 = 6;
pub const L2_TO_L1_LOG_SERIALIZE_SIZE: U256 = 88;

pub const L2_LOG_ADDRESS_OFFSET: U256 = 4;
pub const L2_LOG_KEY_OFFSET: U256 = 24;
pub const L2_LOG_VALUE_OFFSET: U256 = 56;

pub const L2_BOOTLOADER_ADDRESS: Address = Address(0x8001);
pub const L2_KNOWN_CODE_STORAGE_SYSTEM_CONTRACT_ADDR: Address = Address(0x8004);
pub const L2_DEPLOYER_SYSTEM_CONTRACT_ADDR: Address = Address(0x8006);
pub const L2_FORCE_DEPLOYER_ADDR: Address = Address(0x8007);
pub const L2_TO_L1_MESSENGER_SYSTEM_CONTRACT_ADDR: Address = Address(0x8008);
pub const L2_BASE_TOKEN_SYSTEM_CONTRACT_ADDR: Address = Address(0x800a);
pub const L2_SYSTEM_CONTEXT_SYSTEM_CONTRACT_ADDR: Address = Address(0x800b);
pub const L2_PUBDATA_CHUNK_PUBLISHER_ADDR: Address = Address(0x8011);

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