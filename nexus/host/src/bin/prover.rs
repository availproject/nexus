use anyhow::anyhow;
use host::{prover_handle, setup_components};
use nexus_core::types::NexusBlockWithProveStatus;
use nexus_core::{types::ProveStatus, zkvm::ProverMode};
use std::{collections::HashMap, env::args, time::Duration};
use tokio::sync::watch;
use tokio::time::sleep;
use tracing::{error, info};

const MAX_RETRIES: u32 = 10;
const RETRY_DELAY_SECS: u64 = 5;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (node_db, _) = setup_components("./db");

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
        // If block status is not `Proved` we re run the block
        let latest_unproved_block: NexusBlockWithProveStatus = get_nexus_latest_unproved_block(nexus_rpc)
            .await
            .expect("Unable to get latest unproven block.");
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

pub async fn get_nexus_latest_unproved_block(nexus_url: String) -> Result<NexusBlockWithProveStatus, anyhow::Error> {
    let client = reqwest::Client::new();
    let mut params = HashMap::new();
    params.insert("prove_status", ProveStatus::NotProved);

    for attempt in 1..=MAX_RETRIES {
        match client.get(&format!("{}/block-prove-status", nexus_url)).query(&params).send().await {
            Ok(response) if response.status().is_success() => match response.json::<NexusBlockWithProveStatus>().await {
                Ok(block_info) => return Ok(block_info),
                Err(e) => {
                    eprintln!("Attempt {}: Failed to parse JSON: {:?}", attempt, e);
                }
            },
            Ok(response) => {
                eprintln!(
                    "Attempt {}: Non-success status: {}",
                    attempt,
                    response.status()
                );
            }
            Err(e) => {
                eprintln!("Attempt {}: Request error: {:?}", attempt, e);
            }
        }

        if attempt < MAX_RETRIES {
            eprintln!("Retrying in {} seconds...", RETRY_DELAY_SECS);
            sleep(Duration::from_secs(RETRY_DELAY_SECS)).await;
        }
    }

    Err(anyhow!(
        "Failed to fetch latest unproved block after {} attempts",
        MAX_RETRIES
    ))
}
