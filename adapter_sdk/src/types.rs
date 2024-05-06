use crate::traits::Proof;
// use bounded_collections::BoundedVec;

// use codec::{Decode, Encode};

// use avail_core::DataProof;
pub use nexus_core::types::RollupPublicInputsV2 as AdapterPublicInputs;
use nexus_core::types::{AppId, AvailHeader, StatementDigest, H256};
use serde::{Deserialize, Serialize};





#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
// #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
// #[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct TxDataRoots {
	/// Global Merkle root
	pub data_root: H256,
	/// Merkle root hash of submitted data
	pub blob_root: H256,
	/// Merkle root of bridged data
	pub bridge_root: H256,
}

#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct DataProof {
	pub roots: TxDataRoots,
	/// Proof items (does not contain the leaf hash, nor the root obviously).
	///
	/// This vec contains all inner node hashes necessary to reconstruct the root hash given the
	/// leaf hash.
	pub proof: Vec<H256>,
	/// Number of leaves in the original tree.
	///
	/// This is needed to detect a case where we have an odd number of leaves that "get promoted"
	/// to upper layers.
	// #[codec(compact)]
	pub number_of_leaves: u32,
	/// Index of the leaf the proof is for (0-based).
	// #[codec(compact)]
	pub leaf_index: u32,
	/// Leaf content.
	pub leaf: H256,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdapterPrivateInputs {
    pub header: AvailHeader,
    pub app_id: AppId,
    pub blob: Option<(H256, DataProof)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollupPublicInputs {
    pub prev_state_root: H256,
    pub post_state_root: H256,
    pub blob_hash: H256,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollupProof<P: Proof> {
    pub proof: P,
    pub public_inputs: RollupPublicInputs,
}

pub struct AdapterConfig {
    pub app_id: AppId,
    pub elf: Vec<u8>,
    pub adapter_elf_id: StatementDigest,
    pub vk: [u8; 32],
    pub rollup_start_height: u32,
}
