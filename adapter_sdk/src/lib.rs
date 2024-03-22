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
use types::AdapterPublicInputs;

pub async fn run<P: ProofTrait + 'static + Send + Sync>(public_inputs: AdapterPublicInputs) {
    let adapter_state: Arc<Mutex<AdapterState<P>>> =
        Arc::new(Mutex::new(AdapterState::new(public_inputs)));

    let state_copy: Arc<Mutex<AdapterState<P>>> = Arc::clone(&adapter_state);
    tokio::spawn(async move {
        let mut state = state_copy.lock().await;
        state.process_queue().await;
    });

    // Start the server with the state
    server(adapter_state).await;
}
