use crate::types::ProofInputs;
use nexus_core::types::Proof as NexusProof;
use nexus_core::types::{AppAccountId, NexusRollupPI, H256};
use nexus_core::utils::hasher::Digest;
use nexus_core::zkvm::traits::ZKVMEnv;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tracing::debug;
use tree_hash::TreeHash;

#[cfg(any(feature = "native"))]
use nexus_core::zkvm::traits::{ZKVMProof, ZKVMProver};

pub mod prover;
pub mod types;

#[derive(Serialize, Deserialize)]
pub struct ProverInputs {
    proof_inputs: ProofInputs,
    prev_pi_option: Option<NexusRollupPI>,
    app_id_option: Option<AppAccountId>,
    guest_image_id: [u32; 8],
    journal_bytes: Option<Vec<u8>>,
    start_nexus_hash: H256,
}

#[cfg(any(feature = "native", feature = "risc0", feature = "sp1"))]
pub fn create_proof<Z: ZKVMProver<P>, P: ZKVMProof + Serialize + Clone + TryFrom<NexusProof>>(
    prev_proof: Option<P>,
    proof_inputs: ProofInputs,
    prev_pi_option: Option<NexusRollupPI>,
    app_id_option: Option<AppAccountId>,
    guest_image_id: [u32; 8],
    journal_bytes: Option<Vec<u8>>,
    start_nexus_hash: H256,
    prover: Arc<Mutex<Z>>,
) -> Result<P, anyhow::Error>
where
    <P as TryFrom<NexusProof>>::Error: std::fmt::Debug,
{
    // TODO : replace with prover.lock()?
    let mut prover = prover.lock().expect("Unable to lock prover");

    // If previous proof available, add it in assumption
    if let Some(proof) = prev_proof {
        prover.add_proof_for_recursion(proof)?;
    }

    // Adding inputs
    let inputs_vec: Vec<u8> = serde_cbor::to_vec(&ProverInputs {
        proof_inputs,
        prev_pi_option,
        app_id_option,
        guest_image_id,
        journal_bytes,
        start_nexus_hash,
    })
    .expect("Failed to serialize inputs");
    debug!("Serialized inputs: {}", inputs_vec.len());

    prover.add_input(&inputs_vec)?;

    // Run the prover
    prover.prove()
}
