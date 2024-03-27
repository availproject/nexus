pub mod adapter_zkvm;
pub mod service;
pub mod state;
pub mod types;
use nexus_core::traits::{Proof, RollupPublicInputs};
use risc0_zkp::core::digest::Digest;
use state::AdapterState;
use std::sync::Arc;
use tokio::sync::Mutex;
use types::{AdapterPrivateInputs, AdapterPublicInputs};

pub async fn setup<PI: RollupPublicInputs, P: Proof<PI>>(
    public_inputs: AdapterPublicInputs,
    private_inputs: AdapterPrivateInputs,
    vk: [u8; 32],
    zkvm_elf: &[u8],
    zkvm_id: impl Into<Digest>,
) {
    // let adapter_state: AdapterState<PI, P> =
    //     AdapterState::new(public_inputs, private_inputs, vk, zkvm_elf, zkvm_id);
}
