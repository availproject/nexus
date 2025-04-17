use anyhow::anyhow;
use host::{prover_handle, setup_components};
use nexus_core::types::NexusBlockWithProveStatus;
use nexus_core::{types::ProveStatus, zkvm::ProverMode};
use std::{collections::HashMap, env::args, thread, time::Duration};
use tokio::sync::watch;
use tracing::{error, info};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (node_db, state) = setup_components("./db");

    let args: Vec<String> = args().collect();
    let dev_flag = args.iter().any(|arg| arg == "--dev");
    if dev_flag {
        info!("⚠️  Running in dev mode - proofs are not valid");
    }
    let prover_mode = if dev_flag { ProverMode::MockProof } else { ProverMode::Compressed };

    let nexus_rpc = args
        .clone()
        .iter()
        .find(|arg| arg.starts_with("--nexus-rpc="))
        .map(|arg| arg.trim_start_matches("--avail-rpc="))
        .unwrap_or("http://localhost:7000")
        .to_string();

    // Create a Tokio runtime
    let rt = tokio::runtime::Runtime::new().unwrap();
    let (shutdown_tx, shutdown_rx) = watch::channel(false);

    rt.block_on(async {
        // Spawn a task to handle the Ctrl+C signal
        let shutdown_task = tokio::spawn(async move {
            if let Err(err) = tokio::signal::ctrl_c().await {
                error!("Failed to listen for shutdown signal: {:?}", err);
            } else {
                info!("Received shutdown signal, initiating graceful shutdown");
                let _ = shutdown_tx.send(true); // Notify other components
            }
        });

        // Check in db for the latest block which is not proven :
        // If block status is not `ProofGenerationSuccessful`
        // We re run that block
        let latest_unproved_block: NexusBlockWithProveStatus = get_latest_unproved_block_with_retry(nexus_rpc).await;
        let nexus_block_number = match latest_unproved_block.header {
            Some(header) => header.number,
            // Code will never reach here as it will panic during the retries
            None => panic!("Unable to get the nexus block number."),
        };

        info!("Starting prover engine");
        let prover_task = tokio::spawn(async move { prover_handle(node_db, shutdown_rx, nexus_block_number, prover_mode).await });

        // Wait for both tasks to complete
        if let Err(e) = tokio::try_join!(shutdown_task, prover_task) {
            error!("Error during execution: {:?}", e);
        }
    });

    Ok(())
}

async fn get_nexus_latest_unproved_block(nexus_url: String) -> Result<NexusBlockWithProveStatus, anyhow::Error> {
    let client = reqwest::Client::new();
    let mut params = HashMap::new();
    params.insert(String::from("prove_status"), &ProveStatus::NotProved);

    let response = client.get(&format!("{}/block-prove-status", nexus_url)).query(&params).send().await?;
    if response.status().is_success() {
        let block_info: NexusBlockWithProveStatus = response.json().await?;
        Ok(block_info)
    } else {
        Err(anyhow!(
            "Request failed with status code: {}, url: {}",
            response.status(),
            &format!("{}/block-prove-status", nexus_url)
        ))
    }
}

async fn get_latest_unproved_block_with_retry(nexus_url: String) -> NexusBlockWithProveStatus {
    let mut attempts = 0;
    let max_attempts = 10;

    while attempts < max_attempts {
        match get_nexus_latest_unproved_block(nexus_url.clone()).await {
            Ok(block) => {
                if block.header.is_some() {
                    return block;
                } else {
                    panic!("No block yet available to prove.");
                }
            }
            Err(e) => {
                eprintln!(
                    "Attempt {} failed: {:?}. Retrying in 5 seconds...",
                    attempts + 1,
                    e
                );
                attempts += 1;
                thread::sleep(Duration::from_secs(5));
            }
        }
    }

    panic!(
        "Failed to get latest unproved block after {} attempts",
        max_attempts
    );
}
