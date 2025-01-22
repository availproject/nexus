use anyhow::anyhow;
use avail_rust::{
    avail::{
        self, runtime_types::pallet_nexus::nexus_h256::H256,
        runtime_types::pallet_nexus::pallet::NexusHeader,
    },
    block::{Block, DataSubmission},
    error::ClientError,
    transactions::{BalancesEvents, SystemEvents, Transaction},
    utils::account_id_from_str,
    Keypair,
    Nonce::BestBlockAndTxPool,
    Options, SecretUri, SDK,
};
use nexus_core::{
    db::NodeDB,
    types::{HeaderStore, PalletNexusHeader},
};
use prover::{NEXUS_RUNTIME_ELF, NEXUS_RUNTIME_ID};
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::{mpsc::UnboundedReceiver, watch, Mutex};
use tokio::time::Duration;
use tracing::info;

use nexus_core::zkvm::risczero::{RiscZeroProof, RiscZeroProver as Prover, ZKVM};

pub async fn run_proof_submitter(
    node_db: Arc<Mutex<NodeDB>>,
    mut shutdown_rx: watch::Receiver<bool>,
) -> Result<(), anyhow::Error> {
    let sdk = SDK::new("wss://da.nexus.avail.tools")
        .await
        .map_err(|e| anyhow!("Failed to initialize SDK"))?;

    // Input
    let secret_uri = SecretUri::from_str("//Alice")?;
    let account = Keypair::from_uri(&secret_uri)?;
    let options = Some(Options::new().nonce(BestBlockAndTxPool).app_id(0));

    if *shutdown_rx.borrow() {
        info!("Shutdown signal received, stopping execution engine");
        return Ok(());
    }
    let (online_client, rpc_client) = (&sdk.online_client, &sdk.rpc_client);

    let storage_query = avail::storage().nexus().image_id();

    let best_block_hash = Block::fetch_best_block_hash(rpc_client)
        .await
        .map_err(|e| anyhow!("Failed to fetch best block hash: {:?}", e))?;
    let storage = online_client.storage().at(best_block_hash);
    let result: [u32; 8] = storage
        .fetch(&storage_query)
        .await
        .map_err(|e| anyhow!("Failed to fetch best current image id: {:?}", e))?
        .map_or([0; 8], |val| val);

    if result == [0, 0, 0, 0, 0, 0, 0, 0] {
        println!("👨‍💻 Current image id is 0, submitting new image id");
        let payload = avail::tx().nexus().update_image_id(NEXUS_RUNTIME_ID);
        let transaction = Transaction::new(online_client.clone(), rpc_client.clone(), payload);

        let result = transaction
            .execute_wait_for_inclusion(&account, options.clone())
            .await
            .map_err(|e| anyhow!("Failed to execute transaction: {:?}", e))?;

        println!("{:?}", result);
    } else {
        println!("👨‍💻 Current image id is not 0, skipping");
    }
    loop {
        println!("👨‍💻 Checking for new headers");

        if *shutdown_rx.borrow() {
            info!("Shutdown signal received, stopping execution engine");
            return Ok(());
        }
        tokio::time::sleep(Duration::from_secs(2)).await;
        let db_lock = node_db.lock().await;

        let current_nexus_block = match db_lock.get::<HeaderStore>(b"previous_headers") {
            Ok(Some(headers)) => match headers.clone().first() {
                Some(header) => header.clone(),
                None => {
                    println!("No headers in db");

                    continue;
                }
            },
            _ => {
                println!("Error when retrieving headers from db, will try again.");

                continue;
            }
        };

        let proof = match db_lock
            .get::<RiscZeroProof>(&[current_nexus_block.hash().as_slice(), b"-proof"].concat())
        {
            Ok(Some(b)) => {
                // let pallet_header: PalletNexusHeader = match b.0.journal.decode() {
                //     Ok(header) => header,
                //     Err(e) => {
                //         println!("Failed to decode pallet header: {:?}", e);
                //         continue;
                //     }
                // };
                // Serialize the proof to bytes using serde
                match serde_json::to_vec(&b.0) {
                    Err(e) => {
                        println!("Failed to serialize proof: {:?}", e);
                        continue;
                    }
                    Ok(proof_bytes) => {
                        println!("Proof serialized successfully");
                        proof_bytes
                    }
                }
            }
            Ok(None) => {
                println!("Error when retrieving proof from db, will try again.");

                continue;
            }
            Err(_) => {
                println!("Error when retrieving proof from db, will try again.");

                continue;
            }
        };

        let storage_query = avail::storage().nexus().latest_state_root();

        let best_block_hash = Block::fetch_best_block_hash(rpc_client)
            .await
            .map_err(|e| anyhow!("Failed to fetch best block hash: {:?}", e))?;
        let storage = online_client.storage().at(best_block_hash);
        let result = match storage.fetch(&storage_query).await {
            Ok(i) => i,
            Err(e) => {
                println!("Failed to fetch latest state root: {:?}", e);
                continue;
            }
        };

        println!("Starting to update state root, {:?}", result);
        if let Some(state_root) = result {
            if state_root.number < current_nexus_block.number.into() {
                println!("New header not updated. Updating");
                let payload = avail::tx().nexus().update_state_root(
                    NexusHeader {
                        parent_hash: H256(current_nexus_block.parent_hash.as_fixed_slice().clone()),
                        prev_state_root: H256(
                            current_nexus_block.prev_state_root.as_fixed_slice().clone(),
                        ),
                        state_root: H256(current_nexus_block.state_root.as_fixed_slice().clone()),
                        avail_header_hash: H256(
                            current_nexus_block
                                .avail_header_hash
                                .as_fixed_slice()
                                .clone(),
                        ),

                        number: current_nexus_block.number.into(),
                        tx_root: H256(current_nexus_block.tx_root.as_fixed_slice().clone()),
                    },
                    proof,
                );
                let transaction =
                    Transaction::new(online_client.clone(), rpc_client.clone(), payload);

                let result = match transaction
                    .execute_wait_for_inclusion(&account, options.clone())
                    .await
                {
                    Ok(result) => result,
                    Err(e) => {
                        println!("Failed to execute transaction: {:?}", e);
                        continue;
                    }
                };

                println!("{:?}", result);
            }
        } else {
            println!("Initiating with first header");
            let payload = avail::tx().nexus().update_state_root(
                NexusHeader {
                    parent_hash: H256(current_nexus_block.parent_hash.as_fixed_slice().clone()),
                    prev_state_root: H256(
                        current_nexus_block.prev_state_root.as_fixed_slice().clone(),
                    ),
                    state_root: H256(current_nexus_block.state_root.as_fixed_slice().clone()),
                    avail_header_hash: H256(
                        current_nexus_block
                            .avail_header_hash
                            .as_fixed_slice()
                            .clone(),
                    ),

                    number: current_nexus_block.number.into(),
                    tx_root: H256(current_nexus_block.tx_root.as_fixed_slice().clone()),
                },
                proof,
            );
            let transaction = Transaction::new(online_client.clone(), rpc_client.clone(), payload);

            let result = transaction
                .execute_wait_for_inclusion(&account, options.clone())
                .await
                .map_err(|e| anyhow!("Failed to execute transaction: {:?}", e))?;

            println!("{:?}", result);
        }
    }
}
