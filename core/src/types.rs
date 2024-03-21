use anyhow::anyhow;
use anyhow::Error;
#[cfg(any(feature = "native"))]
pub use avail_core::{AppExtrinsic, OpaqueExtrinsic};
#[cfg(any(feature = "native"))]
use avail_subxt::api::runtime_types::avail_core::header::extension::HeaderExtension;
#[cfg(any(feature = "native"))]
pub use avail_subxt::{config::substrate::DigestItem as SpDigestItem, primitives::Header};
use parity_scale_codec::{Decode, Encode};
use risc0_zkvm::sha::rust_crypto::{Digest as RiscZeroDigestTrait, Sha256};
use risc0_zkvm::sha::Digest as RiscZeroDigest;
#[cfg(any(feature = "native"))]
use risc0_zkvm::CompositeReceipt;
#[cfg(any(feature = "native"))]
use risc0_zkvm::Journal;
use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;
use sparse_merkle_tree::traits::{Hasher, Value};
use sparse_merkle_tree::MerkleProof;
pub use sparse_merkle_tree::H256;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, Encode, Decode)]
pub struct AppAccountId(pub [u8; 32]);

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, Encode, Decode)]
pub struct AppId(#[codec(compact)] pub u32);

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, Encode, Decode)]
pub struct TxSignature(#[serde(with = "BigArray")] pub [u8; 64]);

#[derive(Default)]
pub struct ShaHasher(pub Sha256);

#[derive(Clone, Debug, Serialize, Deserialize, Encode, Decode)]
pub struct UpdatedBlob {
    commitment: [u8; 32],
    state_root: [u8; 32],
    //TODO: messages will be added a bit later.
}

//TODO: Need to check PartialEq to Eq difference, to ensure there is not security vulnerability.
#[derive(Clone, Serialize, Deserialize, Debug, Encode, Decode, PartialEq, Eq)]
pub struct AccountState {
    pub statement: [u8; 32],
    pub state_root: [u8; 32],
    pub last_avail_block_hash: [u8; 32],
}

//TODO: Need to check PartialEq to Eq difference, to ensure there is not security vulnerability.
#[derive(Clone, Serialize, Deserialize, Debug, Encode, Decode, PartialEq, Eq)]
pub struct SimpleAccountState {
    pub statement: [u8; 32],
    pub state_root: [u8; 32],
}

#[derive(Clone, Serialize, Deserialize, Debug, Encode, Decode, PartialEq, Eq)]
pub enum TxParamsV2 {
    SubmitProof(SubmitProof),
    InitAccount(InitAccount),
}

#[cfg(any(feature = "native"))]
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct TransactionV2 {
    pub signature: TxSignature,
    pub params: TxParamsV2,
    pub proof: Option<CompositeReceipt>,
}

#[derive(Clone, Serialize, Deserialize, Debug, Encode, Decode, PartialEq, Eq)]
pub struct SubmitProof {
    //Disabled for now.
    //pub proof: risc0_zkvm::InnerReceipt,
    pub public_inputs: RollupPublicInputsV2,
}

#[derive(Clone, Serialize, Deserialize, Debug, Encode, Decode, PartialEq, Eq)]
pub struct InitAccount {
    pub app_id: AppAccountId,
    pub statement: [u8; 32],
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, Encode, Decode)]
pub struct RollupPublicInputs {
    pub pre_state_root: H256,
    pub next_state_root: H256,
    pub start_avail_hash: H256,
    pub proof_at_avail_hash: H256,
    pub app_account_id: AppAccountId,
    //Assuming that nexus sequencer provides linkability to current avail hash.
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, Encode, Decode)]
pub struct StatementDigest(pub [u32; 8]);

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, Encode, Decode)]
pub struct RollupPublicInputsV2 {
    pub header_hash: H256,
    pub state_root: H256,
    pub avail_start_hash: H256,
    pub app_id: AppAccountId,
    pub img_id: StatementDigest,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NexusHeader {
    pub prev_state_root: H256,
    pub state_root: H256,
    pub avail_header_hash: H256,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SimpleNexusHeader {
    pub prev_state_root: H256,
    pub state_root: H256,
    pub tx_root: H256,
    pub avail_blob_root: H256,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StateUpdate {
    pub pre_state_root: H256,
    pub post_state_root: H256,
    pub pre_state: Vec<(AppAccountId, AccountState)>,
    pub post_state: Vec<(AppAccountId, AccountState)>,
    pub proof: Option<MerkleProof>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SimpleStateUpdate {
    pub pre_state_root: H256,
    pub post_state_root: H256,
    pub pre_state: Vec<(AppAccountId, SimpleAccountState)>,
    pub post_state: Vec<(AppAccountId, SimpleAccountState)>,
    pub proof: Option<MerkleProof>,
}

//TODO: Store on hash list, instead of headers.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HeaderStore {
    inner: Vec<AvailHeader>,
    max_size: usize,
}

#[derive(Encode, Decode, Eq, PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct AvailHeader {
    pub parent_hash: H256,
    #[codec(compact)]
    pub number: u32,
    pub state_root: H256,
    pub extrinsics_root: H256,
    pub digest: Digest,
    pub extension: Extension,
}

#[derive(PartialEq, Eq, Clone, Encode, Decode, Debug, Serialize, Deserialize)]
#[repr(u8)]
pub enum Extension {
    V3(V3Extension) = 2,
}

#[derive(PartialEq, Eq, Clone, Encode, Decode, Debug, Serialize, Deserialize)]
pub struct V3Extension {
    app_lookup: DataLookup,
    commitment: KateCommitment,
}

#[derive(PartialEq, Eq, Clone, Encode, Decode, Debug, Default, Serialize, Deserialize)]
pub struct Digest {
    pub logs: Vec<DigestItem>,
}

pub type ConsensusEngineId = [u8; 4];

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum DigestItem {
    PreRuntime([u8; 4], Vec<u8>),
    Consensus([u8; 4], Vec<u8>),
    Seal([u8; 4], Vec<u8>),
    Other(Vec<u8>),
    RuntimeEnvironmentUpdated,
}

#[repr(u32)]
#[derive(Encode, Decode, Serialize, Deserialize)]
enum DigestItemType {
    Other = 0u32,
    Consensus = 4u32,
    Seal = 5u32,
    PreRuntime = 6u32,
    RuntimeEnvironmentUpdated = 8u32,
}

#[derive(PartialEq, Eq, Clone, Encode, Decode, Debug, Serialize, Deserialize)]
pub struct DataLookup {
    #[codec(compact)]
    pub size: ::core::primitive::u32,
    pub index: Vec<DataLookupItem>,
}

#[derive(PartialEq, Eq, Clone, Encode, Decode, Debug, Serialize, Deserialize)]
pub struct DataLookupItem {
    pub app_id: AppId,
    #[codec(compact)]
    pub start: u32,
}

#[derive(PartialEq, Eq, Clone, Encode, Decode)]
pub struct Range<Idx> {
    pub start: Idx,
    pub end: Idx,
}

#[derive(PartialEq, Eq, Clone, Encode, Decode)]
pub struct DataLookupRange {}

#[derive(PartialEq, Eq, Clone, Encode, Decode, Debug, Serialize, Deserialize)]
pub struct KateCommitment {
    #[codec(compact)]
    pub rows: u16,
    #[codec(compact)]
    pub cols: u16,
    pub commitment: Vec<u8>,
    pub data_root: H256,
}

//--------------
//Implementations
//--------------

impl Encode for DigestItem {
    fn encode(&self) -> Vec<u8> {
        let mut v = Vec::new();

        match self {
            Self::Consensus(val, data) => {
                DigestItemType::Consensus.encode_to(&mut v);
                (val, data).encode_to(&mut v);
            }
            Self::Seal(val, sig) => {
                DigestItemType::Seal.encode_to(&mut v);
                (val, sig).encode_to(&mut v);
            }
            Self::PreRuntime(val, data) => {
                DigestItemType::PreRuntime.encode_to(&mut v);
                (val, data).encode_to(&mut v);
            }
            Self::Other(val) => {
                DigestItemType::Other.encode_to(&mut v);
                val.encode_to(&mut v);
            }
            Self::RuntimeEnvironmentUpdated => {
                DigestItemType::RuntimeEnvironmentUpdated.encode_to(&mut v);
            }
        }

        v
    }
}
impl Decode for DigestItem {
    fn decode<I: parity_scale_codec::Input>(
        input: &mut I,
    ) -> Result<Self, parity_scale_codec::Error> {
        let item_type: DigestItemType = Decode::decode(input)?;
        match item_type {
            DigestItemType::PreRuntime => {
                let vals: (ConsensusEngineId, Vec<u8>) = Decode::decode(input)?;
                Ok(Self::PreRuntime(vals.0, vals.1))
            }
            DigestItemType::Consensus => {
                let vals: (ConsensusEngineId, Vec<u8>) = Decode::decode(input)?;
                Ok(Self::Consensus(vals.0, vals.1))
            }
            DigestItemType::Seal => {
                let vals: (ConsensusEngineId, Vec<u8>) = Decode::decode(input)?;
                Ok(Self::Seal(vals.0, vals.1))
            }
            DigestItemType::Other => Ok(Self::Other(Decode::decode(input)?)),
            DigestItemType::RuntimeEnvironmentUpdated => Ok(Self::RuntimeEnvironmentUpdated),
        }
    }
}

#[cfg(any(feature = "native"))]
impl From<&Header> for AvailHeader {
    fn from(header: &Header) -> Self {
        let extension: Extension = match &header.extension {
            HeaderExtension::V1(header) => unreachable!("Not expecting these headers"),
            HeaderExtension::V2(header) => unreachable!("Not expecting these headers"),
            HeaderExtension::V3(header) => Extension::V3(V3Extension {
                app_lookup: DataLookup {
                    size: header.app_lookup.size,
                    index: header
                        .app_lookup
                        .index
                        .iter()
                        .map(|v| DataLookupItem {
                            app_id: AppId(v.app_id.0),
                            start: v.start,
                        })
                        .collect(),
                },
                commitment: KateCommitment {
                    rows: header.commitment.rows,
                    cols: header.commitment.cols,
                    commitment: header.commitment.commitment.clone(),
                    data_root: H256::from(header.commitment.data_root.to_fixed_bytes()),
                },
            }),
        };

        Self {
            parent_hash: H256::from(header.parent_hash.to_fixed_bytes()),
            number: header.number,
            state_root: H256::from(header.state_root.to_fixed_bytes()),
            extrinsics_root: H256::from(header.extrinsics_root.to_fixed_bytes()),
            digest: Digest {
                logs: header
                    .digest
                    .logs
                    .iter()
                    .map(|f| match f {
                        SpDigestItem::PreRuntime(i, v) => {
                            DigestItem::PreRuntime(i.clone(), v.clone())
                        }
                        SpDigestItem::Consensus(i, v) => {
                            DigestItem::Consensus(i.clone(), v.clone())
                        }
                        SpDigestItem::Seal(i, v) => DigestItem::Seal(i.clone(), v.clone()),
                        SpDigestItem::Other(v) => DigestItem::Other(v.clone()),
                        SpDigestItem::RuntimeEnvironmentUpdated => {
                            DigestItem::RuntimeEnvironmentUpdated
                        }
                    })
                    .collect(),
            },
            extension,
        }
    }
}

impl AvailHeader {
    pub fn hash(&self) -> H256 {
        let hash: [u8; 32] = blake2b_simd::Params::new()
            .hash_length(32)
            .hash(&self.encode())
            .as_bytes()
            .try_into()
            .expect("slice is always the necessary length");

        H256::from(hash)
        //blake2_256(&self.encode()).into()
    }
}

impl Value for AccountState {
    fn to_h256(&self) -> H256 {
        if self.statement == [0u8; 32] {
            return H256::zero();
        }

        let mut hasher = ShaHasher::new();
        let serialized = self.encode();
        hasher.0.update(&serialized);

        hasher.finish()
    }

    fn zero() -> Self {
        Self {
            state_root: [0; 32],
            statement: [0; 32],
            last_avail_block_hash: [0; 32],
        }
    }
}

impl Value for SimpleAccountState {
    fn to_h256(&self) -> H256 {
        if self.statement == [0u8; 32] {
            return H256::zero();
        }

        let mut hasher = ShaHasher::new();
        let serialized = self.encode();
        hasher.0.update(&serialized);

        hasher.finish()
    }

    fn zero() -> Self {
        Self {
            state_root: [0; 32],
            statement: [0; 32],
        }
    }
}

impl HeaderStore {
    pub fn new(max_size: usize) -> Self {
        Self {
            inner: Vec::with_capacity(max_size),
            max_size,
        }
    }

    pub fn push_front(&mut self, header: &AvailHeader) -> () {
        if self.inner.len() == self.max_size {
            self.inner.remove(self.max_size - 1); // Remove the last element if size is at max
        }
        self.inner.insert(0, header.clone());
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn first(&self) -> &AvailHeader {
        return &self.inner[0];
    }

    pub fn inner(&self) -> &Vec<AvailHeader> {
        &self.inner
    }
}

impl ShaHasher {
    pub fn new() -> Self {
        Self(Sha256::new())
    }
}
impl Hasher for ShaHasher {
    fn write_h256(&mut self, h: &H256) {
        self.0.update(h.as_slice())
    }

    fn write_byte(&mut self, b: u8) {
        self.0.update([b])
    }

    fn finish(self) -> H256 {
        let bytes = self.0.finalize();
        let sha2_array: [u8; 32] = bytes.as_slice().try_into().unwrap();
        H256::from(sha2_array)
    }
}

#[cfg(any(feature = "native"))]
impl From<AppId> for AppAccountId {
    fn from(value: AppId) -> Self {
        let mut hasher = ShaHasher::new();

        hasher.0.update(&value.0.to_be_bytes());

        let hash: H256 = hasher.finish();

        Self(hash.as_fixed_slice().clone())
    }
}

impl From<H256> for AppAccountId {
    fn from(value: H256) -> Self {
        Self(value.as_fixed_slice().clone())
    }
}

impl AppAccountId {
    pub fn as_h256(&self) -> H256 {
        H256::from(self.0)
    }
}

impl RollupPublicInputsV2 {
    pub fn check_consistency(&self, img_id: &StatementDigest) -> Result<(), anyhow::Error> {
        if img_id != &self.img_id {
            Err(anyhow::anyhow!("The same img_id not used for recursion"))
        } else {
            Ok(())
        }
    }
}

impl From<RiscZeroDigest> for StatementDigest {
    fn from(item: RiscZeroDigest) -> Self {
        let words = item.as_words();
        let mut new_digest = [0u32; 8];

        for (i, &element) in words.iter().take(8).enumerate() {
            new_digest[i] = element;
        }

        Self(new_digest)
    }
}

// impl TryFrom<risc0_zkvm::Journal> for RollupPublicInputs {
//     type Error = anyhow::Error;

//     fn try_from(value: risc0_zkvm::Journal) -> Result<Self, Self::Error> {
//         match from_slice(&value.bytes) {
//             Ok(i) => Ok(i),
//             Err(e) => Err(anyhow::anyhow!(e)),
//         }
//     }
// }

// #[derive(Clone, Serialize, Deserialize, Debug, Encode, Decode)]
// pub struct AccountState {
//     pub statement: [u8; 32],
//     pub state_root: [u8; 32],
//     pub last_avail_block_hash: [u8; 32],
//     //TODO: Need to think about a structure to avoid bloat.
//     pending_blob_hashes: Vec<[u8; 32]>,
//     //hash(previous hash, h(blob1))
//     archived_hash: [u8; 32],
//     updated_blobs: Vec<UpdatedBlob>,
//     timeout: Option<u32>,
//     owner: Option<[u8; 32]>,
// }

// impl RollupPublicInputs {
//     pub fn to_vec(&self) -> Result<Vec<u32>, anyhow::Error> {
//         match risc0_zkvm::serde::to_vec(self) {
//             Err(e) => Err(anyhow!(e)),
//             Ok(i) => Ok(i),
//         }
//     }
// }

// impl AccountState {
//     pub fn pending_blob_hashes_filled(&self) -> bool {
//         if self.pending_blob_hashes.len() > 256 {
//             true
//         } else {
//             false
//         }
//     }

//     pub fn add_hash(&mut self, hash: &[u8; 32]) -> () {
//         if !self.pending_blob_hashes_filled() {
//             self.pending_blob_hashes.push(hash.clone())
//         } else {
//             let mut concat: Vec<u8> = vec![];
//             let hash_to_archive: [u8; 32] = self.pending_blob_hashes.remove(0);

//             concat.extend_from_slice(&self.archived_hash);
//             concat.extend_from_slice(&hash_to_archive);

//             //Need to update to shahasher.
//             let hash = keccak_256(&concat);

//             self.archived_hash = hash;
//             //TODO: Remove is the worst for number of zkvm cycles. Need to optimise this.
//             self.pending_blob_hashes.remove(0);
//             self.pending_blob_hashes.push(hash.clone())
//         }
//     }
// }

// pub struct MachineCallParams {
//     pub header: Header,
// }

// pub struct Transaction {
//     pub signature: Option<TxSignature>,
//     //TODO: To be replaced by an enum
//     pub params: TxParams,
// }

// pub enum TxParams {
//     MachineCall(MachineCallParams),
// }
