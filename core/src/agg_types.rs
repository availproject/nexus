use crate::types::{InitAccount, RollupPublicInputsV2, SubmitProof, TxSignature};
use parity_scale_codec::{Decode, Encode};
use bonsai_sdk::alpha::responses::Groth16Seal;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct SubmitProofTransaction {
    pub signature: TxSignature,
    pub params: SubmitProof,
    pub proof: Groth16Seal,
}

#[derive(Clone, Serialize, Deserialize, Debug, Encode, Decode, PartialEq, Eq)]
pub struct RuntimeTransaction {
    pub signature: TxSignature,
    pub params: SubmitProof,
}

#[derive(Clone, Serialize, Deserialize, Debug, Encode, Decode, PartialEq, Eq)]
pub struct AggregatedTransaction {
    pub submit_proof_txs: Vec<RuntimeTransaction>,
    pub aggregated_proof: Vec<u8>,
}

#[derive(Clone, Serialize, Deserialize, Debug, Encode, Decode, PartialEq, Eq)]
pub struct AggregatedPublicInput(Vec<RollupPublicInputsV2>);

#[derive(Clone, Serialize, Deserialize, Debug, Encode, Decode, PartialEq, Eq)]
pub struct InitTransaction {
    pub signature: TxSignature,
    pub params: InitAccount,
}
