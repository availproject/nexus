use std::collections::HashMap;
use std::fmt;
use std::hash::Hash;

use crate::traits::NexusTransaction;
pub use crate::utils::hasher::Sha256;
use crate::utils::hasher::{Digest as RiscZeroDigestTrait, ShaHasher};
#[cfg(any(feature = "native"))]
pub use avail_core::{AppExtrinsic, OpaqueExtrinsic};
#[cfg(any(feature = "native"))]
use avail_subxt::api::runtime_types::avail_core::header::extension::HeaderExtension;
#[cfg(any(feature = "native"))]
pub use avail_subxt::{config::substrate::DigestItem as SpDigestItem, primitives::Header};
use jmt::proof::{SparseMerkleLeafNode, SparseMerkleNode, SparseMerkleProof, UpdateMerkleProof};
use jmt::storage::TreeUpdateBatch;
use parity_scale_codec::{Decode, Encode};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;
use serde_json::{from_slice, to_vec, Error};
use solabi::{
    decode::{Decode as SolabiDecode, DecodeError, Decoder},
    encode::{Encode as SolabiEncode, Encoder, Size},
};
//TODO: Implement formatter for H256, to display as hex.
pub use crate::h256::H256;
pub use crate::state::types::{AccountState, StatementDigest};
#[cfg(any(feature = "native"))]
use crate::zkvm::traits::ZKVMProof;
use core::fmt::Debug as DebugTrait;
#[cfg(any(feature = "native"))]
use utoipa::ToSchema;

#[derive(Clone, Deserialize, Eq, PartialEq, Serialize, Encode, Decode)]
#[cfg_attr(feature = "native", derive(ToSchema))]
pub struct AppAccountId(pub [u8; 32]);

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, Encode, Decode)]
pub struct AppId(#[codec(compact)] pub u32);

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq, Encode, Decode)]
#[cfg_attr(feature = "native", derive(ToSchema))]
pub struct TxSignature(#[serde(with = "BigArray")] pub [u8; 64]);

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "native", derive(ToSchema))]
pub struct AccountWithProof {
    pub account: AccountState,
    pub proof: Vec<[u8; 32]>,
    pub value_hash: [u8; 32],
    pub nexus_header: NexusHeader,
    pub account_encoded: String,
    pub proof_hex: Vec<String>,
    pub value_hash_hex: String,
    pub nexus_state_root_hex: String,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq, Encode, Decode)]
pub struct UpdatedBlob {
    commitment: [u8; 32],
    state_root: [u8; 32],
    //TODO: messages will be added a bit later.
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq, Encode, Decode)]
#[cfg_attr(feature = "native", derive(ToSchema))]
pub enum TxParams {
    SubmitProof(SubmitProof),
    InitAccount(InitAccount),
}

#[cfg(any(feature = "native"))]
#[derive(Clone, Serialize, Eq, PartialEq, Deserialize, Debug, Encode, Decode)]
#[cfg_attr(feature = "native", derive(ToSchema))]
pub struct Transaction {
    pub signature: TxSignature,
    pub params: TxParams,
}

#[derive(Clone, Serialize, Deserialize, Debug, Encode, Decode)]
pub struct TransactionZKVM {
    pub signature: TxSignature,
    pub params: TxParams,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq, Encode, Decode)]
#[cfg_attr(feature = "native", derive(ToSchema))]
pub struct SubmitProof {
    pub proof: Proof,
    pub nexus_hash: H256,
    pub state_root: H256,
    pub height: u32,
    pub app_id: AppAccountId,
    pub data: Option<H256>,
}

#[derive(Clone, Serialize, Deserialize, Debug, Encode, Decode, PartialEq, Eq)]
#[cfg_attr(feature = "native", derive(ToSchema))]
pub struct Proof(pub Vec<u8>);

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq, Encode, Decode)]
#[cfg_attr(feature = "native", derive(ToSchema))]
pub struct InitAccount {
    pub app_id: AppAccountId,
    pub statement: StatementDigest,
    pub start_nexus_hash: H256,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NexusRollupPI {
    pub nexus_hash: H256,
    pub state_root: H256,
    pub height: u32,
    pub start_nexus_hash: H256,
    pub app_id: AppAccountId,
    pub img_id: StatementDigest,
    pub rollup_hash: Option<H256>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, Encode, Decode)]
#[cfg_attr(feature = "native", derive(ToSchema))]
pub struct NexusHeader {
    pub parent_hash: H256,
    pub prev_state_root: H256,
    pub state_root: H256,
    pub tx_root: H256,
    pub avail_header_hash: H256,
    pub number: u32,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, Encode, Decode)]
pub struct TransactionResult {
    pub hash: H256,
    pub result: bool,
}

#[cfg(any(feature = "native"))]
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "native", derive(ToSchema))]
pub enum TransactionStatus {
    InPool,
    Failed,
    Successful,
}

#[cfg(any(feature = "native"))]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, ToSchema)]
pub struct NexusBlockWithTransactions {
    pub transactions: Vec<TransactionWithStatus>,
    pub header: NexusHeader,
}

#[cfg(any(feature = "native"))]
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct TransactionWithStatus {
    pub transaction: Transaction,
    pub status: TransactionStatus,
    pub block_hash: Option<H256>,
}

#[cfg(any(feature = "native"))]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, Encode, Decode)]
pub struct NexusBlock {
    pub header: NexusHeader,
    pub transactions: Vec<TransactionResult>,
}

#[cfg(any(feature = "native"))]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, Encode, Decode)]
pub struct NexusBlockWithPointers {
    pub block: NexusBlock,
    pub jmt_version: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StateUpdate {
    pub pre_state_root: H256,
    pub post_state_root: H256,
    pub pre_state: HashMap<[u8; 32], (Option<AccountState>, SparseMerkleProof<Sha256>)>,
}

//TODO: Store on hash list, instead of headers.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HeaderStore {
    pub inner: Vec<NexusHeader>,
    pub max_size: usize,
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
    pub app_lookup: DataLookup,
    pub commitment: KateCommitment,
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

impl HeaderStore {
    pub fn new(max_size: usize) -> Self {
        Self {
            inner: Vec::with_capacity(max_size),
            max_size,
        }
    }

    pub fn push_front(&mut self, header: &NexusHeader) -> () {
        if self.inner.len() == self.max_size {
            self.inner.remove(self.max_size - 1); // Remove the last element if size is at max
        }
        self.inner.insert(0, header.clone());
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn first(&self) -> Option<&NexusHeader> {
        if self.is_empty() {
            return None;
        }
        return Some(&self.inner[0]);
    }

    pub fn inner(&self) -> &Vec<NexusHeader> {
        &self.inner
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

impl NexusRollupPI {
    pub fn check_consistency(&self, img_id: &StatementDigest) -> Result<(), anyhow::Error> {
        if img_id != &self.img_id {
            Err(anyhow::anyhow!("The same img_id not used for recursion"))
        } else {
            Ok(())
        }
    }
}

#[cfg(any(feature = "native-risc0"))]
impl TryFrom<risc0_zkvm::Receipt> for Proof {
    type Error = Error;

    fn try_from(value: risc0_zkvm::Receipt) -> Result<Self, Self::Error> {
        Ok(Self(to_vec(&value)?))
    }
}

#[cfg(any(feature = "native-risc0"))]
impl TryInto<risc0_zkvm::Receipt> for Proof {
    type Error = Error;

    fn try_into(self) -> Result<risc0_zkvm::Receipt, Self::Error> {
        Ok(from_slice(&self.0)?)
    }
}

impl NexusTransaction for TransactionZKVM {
    fn hash(&self) -> H256 {
        let serialized = self.params.encode();
        let mut hasher = ShaHasher::new();

        hasher.0.update(&serialized);
        hasher.finish()
    }
}

#[cfg(any(feature = "native"))]
impl NexusTransaction for Transaction {
    fn hash(&self) -> H256 {
        let serialized = self.params.encode();
        let mut hasher = ShaHasher::new();

        hasher.0.update(&serialized);
        hasher.finish()
    }
}

impl NexusHeader {
    pub fn hash(&self) -> H256 {
        let serialized = self.encode();

        let mut hasher = Sha256::new();

        hasher.update(&serialized);
        let mut fixed_bytes = [0u8; 32];

        fixed_bytes.copy_from_slice(hasher.finalize().as_slice());
        let hash = H256::from(fixed_bytes);

        hash
    }
}

impl fmt::Display for AppAccountId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(&self.0))
    }
}

impl fmt::Debug for AppAccountId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "AppAccountId({})", hex::encode(&self.0))
    }
}
