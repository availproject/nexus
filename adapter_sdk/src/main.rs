pub mod rollup;
pub mod state;
pub mod types;

use rollup::server;
use state::AdapterState;
use types::AdapterPublicInputs;

#[tokio::main]
async fn main() {
    let public_inputs = AdapterPublicInputs { /* TODO */ };
    let adapter_state = AdapterState::new(public_inputs);

    tokio::spawn(async move {
        adapter_state.process_queue().await;
    });

    // Start the server with the state
    server(adapter_state).await;
}
