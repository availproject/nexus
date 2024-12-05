pub use avail_subxt::Header;
use nexus_core::{state_machine::StateMachine, zkvm::ProverMode};

#[cfg(any(feature = "risc0"))]
use nexus_core::zkvm::risczero::{RiscZeroProof as Proof, RiscZeroProver as Prover, ZKVM};

use host::{run_nexus, setup_components};
#[cfg(any(feature = "sp1"))]
use nexus_core::zkvm::sp1::{Sp1Proof as Proof, Sp1Prover as Prover, SP1ZKVM as ZKVM};
pub use relayer::{Relayer, SimpleRelayer};
use std::env::args;
use std::sync::Arc;
use tokio::sync::{watch, Mutex};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = args().collect();
    let dev_flag = args.iter().any(|arg| arg == "--dev");
    if dev_flag {
        println!("⚠️ Running in dev mode. Proofs are not valid");
    }

    let prover_mode = if dev_flag {
        ProverMode::MockProof
    } else {
        ProverMode::Compressed
    };

    let (node_db, state) = setup_components("./db");
    let mut state_machine = StateMachine::<ZKVM, Proof>::new(state.clone());

    let relayer_mutex = Arc::new(Mutex::new(SimpleRelayer::new()));
    // Shared shutdown signal using a watch channel
    let (shutdown_tx, shutdown_rx) = watch::channel(false);

    // Create a Tokio runtime
    let rt = tokio::runtime::Runtime::new().unwrap();

    // Run the tasks inside the runtime
    rt.block_on(async {
        // Spawn a task to handle the Ctrl+C signal
        let shutdown_task = tokio::spawn(async move {
            if let Err(err) = tokio::signal::ctrl_c().await {
                eprintln!("Failed to listen for shutdown signal: {:?}", err);
            } else {
                println!("Received shutdown signal. Sending notification...");
                let _ = shutdown_tx.send(true); // Notify other components
            }
        });

        // Spawn the main Nexus logic
        let nexus_task = tokio::spawn(async move {
            run_nexus(
                relayer_mutex,
                node_db,
                state_machine,
                (prover_mode, 7000),
                state,
                shutdown_rx,
            )
            .await;
        });

        // Wait for both tasks to complete
        if let Err(e) = tokio::try_join!(shutdown_task, nexus_task) {
            eprintln!("Error during shutdown or Nexus operation: {:?}", e);
        }
    });

    println!("Nexus has shut down gracefully.");
    Ok(())
}
