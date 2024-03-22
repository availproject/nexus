pub mod adapter_zkvm;
pub mod proof_storage;
pub mod rollup;
pub mod state;
pub mod types;

use crate::proof_storage::ProofTrait;
use rollup::server;
use state::AdapterState;
use std::sync::Arc;
use tokio::sync::Mutex;
use types::{AdapterPrivateInputs, AdapterPublicInputs};

pub async fn run<P: ProofTrait + 'static + Send + Sync>(
    public_inputs: AdapterPublicInputs,
    private_inputs: AdapterPrivateInputs,
    vk: [u8; 32],
) -> Arc<Mutex<AdapterState<P>>> {
    let adapter_state: Arc<Mutex<AdapterState<P>>> = Arc::new(Mutex::new(AdapterState::new(
        public_inputs,
        private_inputs,
        vk,
    )));

    let state_copy: Arc<Mutex<AdapterState<P>>> = Arc::clone(&adapter_state);
    tokio::spawn(async move {
        let state = state_copy.lock().await;
        state.process_queue().await;
    });

    server(adapter_state).await;
}
