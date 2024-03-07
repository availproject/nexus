use crate::types::{RollupPublicInputsV2, SubmitProof, TxSignature};
use parity_scale_codec::{Decode, Encode};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug, Encode, Decode, PartialEq, Eq)]
pub struct SubmitProofTransaction {
    pub signature: TxSignature,
    pub params: SubmitProof,
    pub proof: Vec<u8>,
}

#[derive(Clone, Serialize, Deserialize, Debug, Encode, Decode, PartialEq, Eq)]
pub struct RuntimeTransaction {
    pub signature: TxSignature,
    pub params: SubmitProof,
}

pub struct AggregatedTransaction {
    pub submit_proof_txs: Vec<RuntimeTransaction>,
    pub aggregated_proof: Vec<u8>,
}

pub struct AggregatedPublicInput(Vec<RollupPublicInputsV2>);
