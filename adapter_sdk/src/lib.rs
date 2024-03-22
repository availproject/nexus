pub mod adapter_zkvm;
pub mod proof_storage;
pub mod rollup;
pub mod state;
pub mod types;

use nexus_core::traits::{Proof, RollupPublicInputs};
use state::AdapterState;
use std::sync::Arc;
use tokio::sync::Mutex;
use types::{AdapterPrivateInputs, AdapterPublicInputs};

pub async fn setup<PI: RollupPublicInputs, P: Proof<PI>>(
    public_inputs: AdapterPublicInputs,
    private_inputs: AdapterPrivateInputs,
    vk: [u8; 32],
) -> AdapterState<PI, P> {
    let adapter_state: AdapterState<PI, P> = AdapterState::new(public_inputs, private_inputs, vk);
    adapter_state
}
