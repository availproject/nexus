use crate::rpc::routes;
use anyhow::{anyhow, Context, Error};
use avail_rust::da_commitments::DaCommitmentBuilder;
use avail_rust::error::ClientError;
use avail_rust::kate_recovery::commitments;
use avail_rust::prelude::SDK;
use avail_rust::prelude::*;
use avail_rust::rpc as avail_rpc;
use avail_rust::subxt::config::Header as HeaderTrait;
use avail_rust::transactions::DataAvailabilityCalls::SubmitDataWithCommitments;
pub use avail_rust::AvailHeader as Header;
use avail_rust::H256 as AvailH256;
use avail_rust::{Filter as AvailFilter, Keypair};
use jmt::storage::TreeUpdateBatch;
use kzg::verify_row_kzg;
use kzg::{compute_kzg_proof, compute_row_proof};
use nexus_core::db::SharedDB;
use nexus_core::metrics::{ExecutionMetrics, ProvingMetrics};
use nexus_core::types::BlobProof;
use nexus_core::types::NexusZKVMInputs;
#[cfg(any(feature = "risc0"))]
use nexus_core::zkvm::risczero::{RiscZeroProof as Proof, RiscZeroProver as Prover, ZKVM};
use nexus_core::{
    db::{BatchTransaction, NodeDB},
    mempool::Mempool,
    state::VmState,
    state_machine::StateMachine,
    traits::NexusTransaction,
    types::{
        AvailHeader, BlockStatus, HeaderStore, NexusBlock, NexusBlockWithPointers, NexusHeader, NexusRollupPI, Proof as NexusProof, Transaction,
        TransactionResult, TransactionStatus, TransactionWithStatus, TransactionZKVM, TxParams, H256,
    },
    zkvm::{
        traits::{ZKVMEnv, ZKVMProof, ZKVMProver},
        ProverMode,
    },
};
use serde_json;
use sp_runtime::MultiSigner;
use std::{collections::HashMap, mem, thread};
use tokio::fs;
use tracing::{debug, error, info, instrument, warn};

#[cfg(any(feature = "sp1"))]
use nexus_core::zkvm::sp1::{Sp1Proof as Proof, Sp1Prover as Prover, SP1ZKVM as ZKVM};

#[cfg(any(feature = "risc0"))]
use prover::{NEXUS_RUNTIME_ELF, NEXUS_RUNTIME_ID};
pub use relayer::{Relayer, SimpleRelayer};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use std::{env::args, fmt::Debug as DebugTrait};
use tokio::sync::{mpsc::UnboundedReceiver, watch, Mutex};
//use avail_rust::
use tokio::time::{sleep, Duration};
use warp::Filter;

//Kate imports
use kzg::kate::gridgen::AsBytes;
use kzg::kate::gridgen::EvaluationGrid;
use kzg::kate::Seed;
use kzg::kate_recovery::data::Cell;
use kzg::kate_recovery::matrix::Position;
use nexus_core::types::{Blob, CompactDataLookup, HeaderExtension};

type DataSubmissionWithCommitmentsCall = avail::data_availability::calls::types::SubmitDataWithCommitments;

pub mod instrumentation;
pub mod rpc;
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AvailToNexusPointer {
    number: u32,
    nexus_hash: H256,
}

pub fn setup_components(db_path: &str) -> (Arc<Mutex<NodeDB>>, Arc<Mutex<VmState>>) {
    // Construct the node_db path directly as a string
    let node_db_path: String = format!("{}/node_db", db_path);
    let node_db = NodeDB::from_path(&node_db_path);

    // Use the runtime_db path directly as a string
    let runtime_db_path = format!("{}/runtime_db", db_path);
    let state = Arc::new(Mutex::new(VmState::new(&runtime_db_path)));

    (Arc::new(Mutex::new(node_db)), state)
}

pub async fn submit_data(data: Vec<u8>, sdk: &SDK, signer: Keypair, app_id: u32) -> Result<(), anyhow::Error> {
    let commitments = DaCommitmentBuilder::new(data.clone())
        .build()
        .map_err(|e| anyhow!("Failed to build DA commitments: {:?}", e))?;

    let alice_address = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY";

    let nonce = account::nonce(&sdk.client, alice_address).await.unwrap();
    let options = Options::new().app_id(app_id).nonce(nonce);
    let tx = sdk.tx.data_availability.submit_data_with_commitments(data, commitments);
    let res = tx
        .execute_and_watch_inclusion(&signer, options)
        .await
        .map_err(|e| anyhow!("Failed to execute and watch transaction: {:?}", e))?;

    assert_eq!(
        res.is_successful(),
        Some(true),
        "Transactions must be successful"
    );

    info!(
        "Block Hash: {:?}, Block Number: {}, Tx Hash: {:?}, Tx Index: {}",
        res.block_hash, res.block_number, res.tx_hash, res.tx_index
    );

    // Decoding
    let decoded = res
        .decode_as::<DataSubmissionWithCommitmentsCall>()
        .await
        .map_err(|e| anyhow!("Failed to decode transaction response: {:?}", e))?;
    let Some(decoded) = decoded else {
        return Err(anyhow!("Failed to get Data Submission Call data"));
    };

    info!("Data Submission with commitments completed correctly");

    assert_eq!(
        res.is_successful(),
        Some(true),
        "Transactions must be successful"
    );

    Ok(())
}

#[instrument(level = "info", skip(mempool, shutdown_rx, sdk))]
pub async fn sequencer_handle(mempool: Mempool, mut shutdown_rx: watch::Receiver<bool>, sdk: SDK, app_id: u32) -> () {
    loop {
        tokio::time::sleep(Duration::from_secs(5)).await;
        info!("Starting batching process");

        if *shutdown_rx.borrow() {
            info!("Shutdown signal received. Stopping sequencer...");
            break;
        }

        let account = account::alice();

        let (txs, index) = mempool.get_current_txs().await;

        if txs.is_empty() {
            info!("No txs for this batch, skipping blob submission");

            continue;
        }
        let blob: &[u8] = &bincode::serialize(&txs).unwrap();

        let result = submit_data(blob.to_vec(), &sdk, account, app_id).await;

        match result {
            Ok(_) => info!("Data submitted successfully"),
            Err(e) => {
                error!(error = ?e, "Data submission failed");

                break;
            }
        };

        if let Some(i) = index {
            mempool.clear_upto_tx(i.clone()).await;
        };
    }
}

#[instrument(level = "info", skip(shared_db, shutdown_rx, prover_metrics))]
pub async fn prover_handle(
    shared_db: Arc<SharedDB>,
    mut shutdown_rx: watch::Receiver<bool>,
    start_block: u32,
    prover_mode: ProverMode,
    prover_metrics: ProvingMetrics,
) -> Result<(), Error> {
    info!("Starting prover engine in {:?} mode", prover_mode);

    let mut block_to_prove: u32 = start_block;

    loop {
        if *shutdown_rx.borrow() {
            info!("Shutdown signal received, stopping execution engine");
            break;
        }

        tokio::time::sleep(Duration::from_secs(1)).await;

        let (mut block_with_info, nexus_hash): (NexusBlockWithPointers, H256) = {
            match shared_db.get_block_with_number(block_to_prove as u64).await {
                Ok(Some(block)) => {
                    let nexus_hash = block.block.header.hash();
                    (block, nexus_hash)
                }
                Ok(None) => {
                    warn!("Block {} not found retrying in sometime.", block_to_prove);
                    continue;
                }
                Err(e) => {
                    error!(error = ?e, "Error getting block details for block number {}", block_to_prove);
                    continue;
                }
            }
        };

        if block_with_info.block_status == BlockStatus::ProofGenerationSuccessful {
            info!(
                "Block number {:?} already proved. Skipping.....",
                block_to_prove
            );
            continue;
        }

        info!("Proving Block : {:?}", block_to_prove);

        let proof_result = match prove_batch::<Prover, Proof, ZKVM>(
            block_with_info.zkvm_inputs.clone(),
            block_with_info.block.header.clone(),
            &prover_mode,
            prover_metrics.clone(),
        )
        .await
        {
            Ok(i) => {
                //TODO: remove zkvm_inputs to avoid state bloat.
                block_with_info.block_status = BlockStatus::ProofGenerationSuccessful;
                Some(i)
            }
            Err(e) => {
                error!(error = ?e, "Prover error when proving block {}", block_to_prove);
                block_with_info.block_status = BlockStatus::ProofGenerationFailed;

                //We do not exit the loop, and retry in the next iteration, as sometimes it could be a temporary issue
                None
            }
        };

        if proof_result.is_some() {
            // TODO : check if we need to store this actually. As it is not feched anywhere rn.
            // node_db.put(
            //     &[nexus_hash.as_slice(), b"-proof"].concat(),
            //     &proof_result.unwrap(),
            // )?;

            // Updating only when proof result is successful.
            // Updating the block status in shared DB to ProofGenerationSuccessful.
            shared_db
                .update_block_status_and_submit_proof(
                    block_to_prove as u64,
                    block_with_info.block.header.hash(),
                    bincode::serialize(&proof_result.unwrap().0)?,
                    BlockStatus::ProofGenerationSuccessful,
                )
                .await?;

            block_to_prove += 1;
        }
    }

    Ok(())
}

const MAX_RETRIES: u32 = 2;
const RETRY_DELAY_SECS: u64 = 5;

pub async fn get_latest_proven_block(shared_db: &Arc<SharedDB>) -> Result<u64, Error> {
    let mut retries = 0;
    loop {
        match shared_db.get_latest_proven_block().await? {
            Some(block) => {
                return Ok((block + 1).try_into().unwrap());
            }
            None => {
                if retries == MAX_RETRIES {
                    info!("Max retries reached. Starting Proving from block 0");
                    return Ok(0);
                }
                sleep(Duration::from_secs(RETRY_DELAY_SECS)).await;
                warn!("Didn't found any last proven block. Retrying.....");
                retries += 1;
                continue;
            }
        }
    }
}

#[instrument(level = "info", skip(relayer_mutex, node_db_mutex, shutdown_rx))]
pub async fn relayer_handle(
    relayer_mutex: Arc<Mutex<impl Relayer + Send + 'static>>,
    node_db_mutex: Arc<Mutex<NodeDB>>,
    mut shutdown_rx: watch::Receiver<bool>,
    avail_start_block: u32,
) -> () {
    let relayer = relayer_mutex.lock().await;
    let start_height: u32 = {
        let db_lock = node_db_mutex.lock().await;

        let avail_hash: Option<H256> = match db_lock.get::<HeaderStore>(b"previous_headers") {
            //Can do unwrap below as an empty store would not be stored.
            Ok(Some(i)) => Some(i.first().unwrap().avail_header_hash),
            Ok(None) => None,
            Err(_) => panic!("Could not access db"),
        };

        if let Some(hash) = avail_hash {
            let height = match db_lock.get::<AvailToNexusPointer>(hash.as_slice()) {
                Ok(Some(i)) => i.number,
                Ok(None) => panic!("Node DB error. Cannot find mapping to avail -> nexus block for already processed block"),
                Err(e) => {
                    error!(error = ?e, "Node DB error");
                    panic!("Node DB error. Cannot find mapping to avail -> nexus block")
                }
            } + 1;

            height
        } else {
            avail_start_block
        }
    };

    tokio::select! {
        _ = relayer.start(start_height) => {
            info!("Relayer start function exited");
        }
        _ = shutdown_rx.changed() => {
            if *shutdown_rx.borrow() {
                info!("Shutdown signal received. Stopping relayer handle...");
                relayer.stop();
            }
        }
    }

    info!("Exited relayer handle");
}

pub async fn execute_batch<Z: ZKVMProver<P>, P: ZKVMProof + Serialize + Clone + DebugTrait + TryFrom<NexusProof>, E: ZKVMEnv>(
    blobs: &Vec<Blob>,
    blob_proofs: &Vec<BlobProof>,
    state_machine: &mut StateMachine<P>,
    header: &AvailHeader,
    header_store: &mut HeaderStore,
    app_id: u32,
    execution_metrics: ExecutionMetrics,
) -> Result<
    (
        NexusZKVMInputs,
        NexusHeader,
        HashMap<H256, bool>,
        Option<TreeUpdateBatch>,
    ),
    Error,
>
where
    <P as TryFrom<NexusProof>>::Error: std::fmt::Debug,
{
    let mut txs: Vec<Transaction> = Vec::new();

    if blobs.len() != blob_proofs.len() {
        return Err(anyhow!("Blob and blob proof lengths do not match"));
    }

    for blob in blobs {
        let blob_txs: Vec<Transaction> = bincode::deserialize(&blob.get_data()).map_err(|e| anyhow!("blob deserialization error: {:?}", e))?;
        txs.extend(blob_txs);
    }

    let commitments: Vec<[u8; 48]> = {
        let (app_lookup, commitments): (CompactDataLookup, Vec<[u8; 48]>) = match &header.extension {
            HeaderExtension::V3(extension) => {
                let commitment_chunks: Vec<[u8; 48]> = extension
                    .commitment
                    .commitment
                    .chunks_exact(48)
                    .map(|chunk| {
                        let mut arr = [0u8; 48];
                        arr.copy_from_slice(chunk);
                        arr
                    })
                    .collect();
                (extension.app_lookup.clone(), commitment_chunks)
            }
            _ => return Err(anyhow!("Header extension not supported")),
        };

        let mut filtered_commitments: Vec<[u8; 48]> = Vec::new();

        // may not need to sort here by start.
        let mut sorted_indices: Vec<_> = app_lookup.index.iter().collect();
        sorted_indices.sort_by_key(|i| i.start);

        for (idx, current) in sorted_indices.iter().enumerate() {
            if current.app_id.0 == app_id {
                let start = current.start as usize;
                let end = if idx + 1 < sorted_indices.len() {
                    sorted_indices[idx + 1].start as usize
                } else {
                    commitments.len()
                };

                filtered_commitments.extend_from_slice(&commitments[start..end]);
            }
        }

        filtered_commitments
    };

    for (i, blob) in blobs.iter().enumerate() {
        let verification_result = verify_row_kzg(&blob.0, &commitments[i], &blob_proofs[i].0)?;
        if !verification_result {
            return Err(anyhow!("Verification result: {}", verification_result));
        }
    }

    let batch_execution_time_start = std::time::Instant::now();
    let (tree_update_batch, state_update, tx_result, nexus_header): (
        Option<jmt::storage::TreeUpdateBatch>,
        nexus_core::types::StateUpdate,
        HashMap<H256, bool>,
        NexusHeader,
    ) = state_machine.execute_batch(&header, header_store, &txs).await?;
    let batch_execution_time = batch_execution_time_start.elapsed();
    execution_metrics.batch_execution_time.record(batch_execution_time.as_secs(), &[]);

    //Creating zkvm_inputs before adding the new header to header store.
    let zkvm_inputs = NexusZKVMInputs {
        blobs: blobs.clone(),
        blob_proofs: blob_proofs.clone(),
        state_update,
        header: header.clone(),
        header_store: header_store.clone(),
        app_id,
    };

    header_store.push_front(&nexus_header);

    Ok((zkvm_inputs, nexus_header, tx_result, tree_update_batch))
}

pub async fn prove_batch<Z: ZKVMProver<P>, P: ZKVMProof + Serialize + Clone + DebugTrait + TryFrom<NexusProof>, E: ZKVMEnv>(
    zkvm_inputs: NexusZKVMInputs,
    nexus_header: NexusHeader,
    prover_mode: &ProverMode,
    prover_metrics: ProvingMetrics,
) -> Result<P, Error>
where
    <P as TryFrom<NexusProof>>::Error: std::fmt::Debug,
{
    let mut txs: Vec<Transaction> = Vec::new();

    if zkvm_inputs.blobs.len() != zkvm_inputs.blob_proofs.len() {
        return Err(anyhow!("Blob and blob proof lengths do not match"));
    }

    for blob in &zkvm_inputs.blobs {
        let blob_txs: Vec<Transaction> = bincode::deserialize(&blob.get_data()).map_err(|e| anyhow!("blob deserialization error: {:?}", e))?;
        txs.extend(blob_txs);
    }

    let batch_proving_time_start = std::time::Instant::now();
    let (proof, result) = {
        #[cfg(any(feature = "sp1"))]
        let NEXUS_RUNTIME_ELF: &[u8] = include_bytes!("../../../target/elf-compilation/riscv32im-succinct-zkvm-elf/release/nexus_runtime_sp1");

        let mut zkvm_prover = Z::new(NEXUS_RUNTIME_ELF.to_vec(), prover_mode.clone());

        for tx in &txs {
            if let TxParams::SubmitProof(submit_proof_tx) = &tx.params {
                //TODO: Remove transactions that error out from mempool
                let proof = submit_proof_tx.proof.clone();
                let mut receipt: P = P::try_from(proof).unwrap();

                zkvm_prover.add_proof_for_recursion(receipt).unwrap();
            }
        }

        zkvm_prover.add_input(&zkvm_inputs).unwrap();
        let mut proof = zkvm_prover.prove()?;

        let result: NexusHeader = proof.public_inputs()?;

        (proof, result)
    };
    let batch_proving_time = batch_proving_time_start.elapsed();
    prover_metrics.batch_proving_time.record(batch_proving_time.as_secs(), &[]);

    if result.hash() != nexus_header.hash() {
        return Err(anyhow!(
            "Header produced during proof generation is different from the provided header"
        ));
    }

    Ok(proof)
}

pub async fn get_blobs_for_block(sdk: &SDK, avail_block_hash: AvailH256, app_id: u32) -> Result<(Vec<Blob>, Vec<BlobProof>), anyhow::Error> {
    info!(
        "Getting block for app ID: {} and block hash: {:?}",
        app_id, &avail_block_hash
    );
    let block = match Block::new(&sdk.client, avail_block_hash.into())
        .await
        .map_err(|e| anyhow!("Error querying block: {:?}", e))
    {
        Ok(i) => i,
        Err(e) => {
            return Err(e);
        }
    };

    let header = block.block.header();

    debug!("app index: {:?}", header.extension);

    let blob_txs = block.transactions_static::<SubmitDataWithCommitments>(AvailFilter::new().app_id(app_id));

    let mut checked_blobs: Vec<Vec<u8>> = Vec::new();

    for blob_tx in blob_txs {
        checked_blobs.push(blob_tx.value.data.0.clone());
    }

    let mut blobs: Vec<Blob> = Vec::new();

    for (_, blob_data) in checked_blobs.iter().enumerate() {
        let grid = EvaluationGrid::from_data(blob_data.clone(), 1024, 1024, 256, Seed::default()).expect("Failed to create evaluation grid");

        let mut rows: Vec<Blob> = Vec::new();

        for row_num in 0..grid.dims().rows().get() {
            let row = grid
                .row(row_num.into())
                .ok_or_else(|| anyhow!("Row {} should exist in grid but was not found", row_num))?
                .into_iter()
                .map(|c| c.to_bytes().map_err(|e| anyhow!("Failed to convert cell to bytes: {:?}", e)))
                .collect::<Result<Vec<_>, _>>()?;

            rows.push(Blob(row));
        }

        blobs.extend(rows);
    }

    let proofs = blobs
        .iter()
        .map(|blob| -> Result<BlobProof, anyhow::Error> {
            let proof = compute_row_proof(&blob.0)?;
            Ok(BlobProof(proof))
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok((blobs, proofs))
}

#[instrument(
    level = "info",
    skip(node_db, shared_db, state_machine, prover_mode, shutdown_rx, state, receiver, sdk, execution_metrics)
)]
pub async fn execution_engine_handle(
    receiver: Arc<Mutex<UnboundedReceiver<Header>>>,
    node_db: Arc<Mutex<NodeDB>>,
    shared_db: Arc<SharedDB>,
    mut state_machine: StateMachine<Proof>,
    prover_mode: ProverMode,
    mut shutdown_rx: watch::Receiver<bool>,
    state: Arc<Mutex<VmState>>,
    sdk: SDK,
    app_id: u32,
    execution_metrics: ExecutionMetrics,
) -> Result<(), anyhow::Error> {
    info!("Starting execution engine in {:?} mode", prover_mode);
    const MAX_HEADERS: usize = 5;
    let mut header_array: Vec<Header> = Vec::new();

    loop {
        if *shutdown_rx.borrow() {
            info!("Shutdown signal received, stopping execution engine");
            break;
        }

        let header_opt = {
            let mut lock = receiver.lock().await;
            lock.try_recv().ok()
        };

        if let Some(header) = header_opt {
            info!("━━━━━━━━━━━━━━━━━ NEW BLOCK ━━━━━━━━━━━━━━━━━");
            info!(
                avail_block = header.number,
                avail_hash = %hex::encode(header.hash()),
                parent_hash = %hex::encode(header.parent_hash),
                "Received new AvailDA header"
            );

            header_array.push(header.clone());

            let mut old_headers: HeaderStore = {
                let db_lock = node_db.lock().await;
                match db_lock.get::<HeaderStore>(b"previous_headers") {
                    Ok(Some(i)) => {
                        debug!("Loaded {} previous headers", i.inner().len());
                        i
                    }
                    Ok(None) => {
                        debug!("Creating new header store");
                        HeaderStore::new(32)
                    }
                    Err(_) => {
                        error!("Failed to get previous headers from DB");
                        return Err(anyhow!(
                            "DB Call failed to get previous headers. Restart required."
                        ));
                    }
                }
            };

            //let (txs, index) = mempool.get_current_txs().await;

            let (blobs, proofs) = match get_blobs_for_block(
                &sdk,
                AvailH256::from(header.hash().as_fixed_bytes()),
                app_id,
            )
            .await
            {
                Ok(i) => i,
                Err(e) => {
                    error!(error = ?e, "Failed to get blobs for block");

                    continue;
                }
            };
            info!(
                avail_block = header.number,
                blob_count = blobs.len(),
                "📦 Starting batch processing"
            );

            debug!("🔄 Beginning batch execution");

            tokio::time::sleep(Duration::from_secs(5)).await;
            let batch_execution_start_time = std::time::Instant::now();
            match execute_batch::<Prover, Proof, ZKVM>(
                &blobs,
                &proofs,
                &mut state_machine,
                &AvailHeader::from(&header),
                &mut old_headers,
                app_id,
                execution_metrics.clone(),
            )
            .await
            {
                Ok((zkvm_inputs, result, tx_result, tree_update_batch)) => {
                    let updated_version = state.lock().await.get_version(false)?;
                    info!(
                        nexus_block = result.number,
                        batch_hash = %hex::encode(result.hash().as_slice()),
                        state_root = %hex::encode(result.state_root.as_slice()),
                        state_version = ?updated_version,
                        "✨ Batch execution completed"
                    );

                    info!("💾 Starting batch commit");

                    let mut txs: Vec<Transaction> = Vec::new();

                    for blob in blobs {
                        let blob_txs: Vec<Transaction> =
                            bincode::deserialize(&blob.get_data()).map_err(|e| anyhow!("blob deserialization error: {:?}", e))?;
                        txs.extend(blob_txs);
                    }

                    match save_batch_information(
                        &node_db,
                        &shared_db,
                        &mut state_machine,
                        ProcessedBatchInfo {
                            avail_header: &header,
                            header: &result,
                            txs_result: &tx_result,
                            tree_update_batch,
                            txs: &txs,
                            updated_header_store: &old_headers,
                            jmt_version: match updated_version {
                                Some(i) => i,
                                None => 0,
                            },
                            zkvm_inputs,
                        },
                    )
                    .await
                    {
                        Ok(_) => {
                            let successful_txs = tx_result.values().filter(|&&success| success).count();

                            let txs_length = tx_result.values().len();
                            info!(
                                nexus_block = result.number,
                                batch_hash = %hex::encode(result.hash().as_slice()),
                                state_root = %hex::encode(result.state_root.as_slice()),
                                total_txs = txs_length,
                                successful_txs = successful_txs,
                                failed_txs = txs_length - successful_txs,
                                "✅ Batch processing completed successfully"
                            );

                            execution_metrics.batch_number_counter.add(1, &[]);
                            execution_metrics.number_of_transactions_batch.record(txs_length as u64, &[]);
                        }
                        Err(e) => {
                            error!(error = ?e, "❌ Failed to commit batch");
                            return Err(e);
                        }
                    }
                }
                Err(e) => {
                    error!(error = ?e, "❌ Batch execution failed");
                    return Err(e);
                }
            }
            info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ \n");
            let batch_execution_time = batch_execution_start_time.elapsed();
            execution_metrics.total_batch_execution_time.record(batch_execution_time.as_secs(), &[]);
        } else {
            debug!("Waiting for new blocks");
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }

    info!("Execution engine stopped");
    Ok(())
}

#[instrument(level = "debug", skip(node_db, shared_db, state_machine, processed_batch_info))]
pub async fn save_batch_information<'a>(
    node_db: &Arc<Mutex<NodeDB>>,
    shared_db: &Arc<SharedDB>,
    state_machine: &mut StateMachine<Proof>,
    processed_batch_info: ProcessedBatchInfo<'a>,
) -> Result<(), Error> {
    debug!(
        nexus_block = processed_batch_info.header.number,
        "Starting batch commit"
    );

    if let Some(tree_update) = &processed_batch_info.tree_update_batch {
        debug!(
            nexus_block = processed_batch_info.header.number,
            node_count = tree_update.node_batch.nodes().len(),
            "Committing state updates"
        );

        state_machine
            .commit_state(
                &processed_batch_info.header.state_root,
                &tree_update.node_batch,
                processed_batch_info.header.number,
            )
            .await?;
    }

    debug!("Writing batch data to database");
    let nexus_hash = processed_batch_info.header.hash();
    let mut batch_transaction = BatchTransaction::new();

    batch_transaction.put(
        b"previous_headers",
        &processed_batch_info.updated_header_store,
    );
    batch_transaction.put(
        processed_batch_info.header.avail_header_hash.as_slice(),
        &AvailToNexusPointer {
            number: processed_batch_info.avail_header.number,
            nexus_hash: nexus_hash.clone(),
        },
    );

    let mut txs_result_vec: Vec<TransactionResult> = vec![];

    for (tx_hash, success) in processed_batch_info.txs_result.iter() {
        let db_lock = node_db.lock().await;
        let mut tx: TransactionWithStatus = match db_lock.get::<TransactionWithStatus>(tx_hash.as_slice())? {
            //Can do unwrap below as an empty store would not be stored.
            Some(i) => i,
            None => {
                let tx = processed_batch_info.txs.iter().find(|tx| tx.hash() == tx_hash.clone()).unwrap().clone();
                let mut tx_with_status = TransactionWithStatus {
                    transaction: tx.clone(),
                    status: TransactionStatus::InPool,
                    block_hash: None,
                };

                tx_with_status
            }
        };

        tx.block_hash = Some(nexus_hash.clone());
        tx.status = if success.clone() {
            TransactionStatus::Successful
        } else {
            TransactionStatus::Failed
        };

        batch_transaction.put(tx_hash.as_slice(), &tx);
        txs_result_vec.push(TransactionResult {
            hash: tx_hash.clone(),
            result: success.clone(),
        });
    }
    batch_transaction.put(nexus_hash.as_slice(), &processed_batch_info.header);

    let nexus_block_with_pointers = NexusBlockWithPointers {
        block: NexusBlock {
            header: processed_batch_info.header.clone(),
            transactions: txs_result_vec,
        },
        jmt_version: processed_batch_info.jmt_version,
        zkvm_inputs: processed_batch_info.zkvm_inputs,
        block_status: BlockStatus::ExecutionCompleted,
    };

    batch_transaction.put(
        &[nexus_hash.as_slice(), b"-block"].concat(),
        &nexus_block_with_pointers,
    );

    batch_transaction.put(
        &[processed_batch_info.header.number.to_be_bytes().as_slice(), b"-block"].concat(),
        &nexus_hash,
    );
    let db_lock = node_db.lock().await;
    db_lock.put_batch(batch_transaction)?;

    db_lock.set_current_root(&processed_batch_info.header.state_root).unwrap();

    // Inserting the block into storage db to be accessible by
    // other services.
    println!("Adding nexus block with pointers to postgresql");
    shared_db.insert_nexus_block_with_pointers(&nexus_block_with_pointers).await?;

    Ok(())
}

pub struct ProcessedBatchInfo<'a> {
    avail_header: &'a Header,
    header: &'a NexusHeader,
    txs_result: &'a HashMap<H256, bool>,
    tree_update_batch: Option<TreeUpdateBatch>,
    txs: &'a Vec<Transaction>,
    //mempool_index: &'a Option<usize>,
    updated_header_store: &'a HeaderStore,
    jmt_version: u64,
    zkvm_inputs: NexusZKVMInputs,
}

pub fn run_server(
    mempool: Mempool,
    node_db: Arc<Mutex<NodeDB>>,
    shared_db: Arc<SharedDB>,
    state: Arc<Mutex<VmState>>,
    mut shutdown_rx: watch::Receiver<bool>,
    port: u32,
) -> tokio::task::JoinHandle<()> {
    let routes = routes(mempool, node_db, shared_db, state.clone());
    let cors = warp::cors()
        .allow_any_origin()
        .allow_methods(vec!["POST"])
        .allow_headers(vec!["content-type"]);
    let routes = routes.with(cors);

    tokio::spawn(async move {
        let address = SocketAddr::from_str(format!("{}:{}", String::from("127.0.0.1"), port).as_str())
            .context("Unable to parse host address from config")
            .unwrap();

        info!("🌐 RPC Server running on: {:?}", &address);

        let (_, server) = warp::serve(routes).bind_with_graceful_shutdown(address, async move {
            shutdown_rx.changed().await.ok();
            info!("💤 Shutdown signal received. Stopping server...");
        });

        server.await;

        info!("✅ Exited server handle");
    })
}

pub async fn run_nexus(
    relayer_mutex: Arc<Mutex<impl Relayer + Send + 'static>>,
    node_db: Arc<Mutex<NodeDB>>,
    shared_db: Arc<SharedDB>,
    mut state_machine: StateMachine<Proof>,
    (prover_mode, server_port): (ProverMode, u32),
    state: Arc<Mutex<VmState>>,
    mut shutdown_rx: watch::Receiver<bool>,
    ws_url: String,
    app_id: u32,
    start_block: u32,
    avail_start_block: u32,
) -> Result<(), Error> {
    let mut shutdown_rx_1 = shutdown_rx.clone();
    let mut shutdown_rx_2 = shutdown_rx.clone();
    let mut shutdown_rx_3 = shutdown_rx.clone();
    let mut shutdown_rx_4 = shutdown_rx.clone();

    let ws_url_clone = ws_url.clone();

    let db_clone = node_db.clone();
    let db_clone_2 = node_db.clone();
    let db_clone_3 = node_db.clone();
    let state_2 = state.clone();

    let prover_mode_clone = prover_mode.clone();

    // Initialize the execution metrics
    let execution_metrics = ExecutionMetrics::init();

    let receiver = {
        let mut relayer = relayer_mutex.lock().await;

        relayer.receiver()
    };
    let mempool = Mempool::new(node_db.clone());
    let mempool_clone = mempool.clone();

    let server_handle = run_server(
        mempool,
        db_clone,
        shared_db.clone(),
        state,
        shutdown_rx,
        server_port,
    );
    let relayer_handle = tokio::spawn(async move {
        relayer_handle(
            relayer_mutex,
            db_clone_2,
            shutdown_rx_1.clone(),
            avail_start_block,
        )
        .await
    });

    // The prover handle picks up executed batches from the shared db and generates the proofs with zkvm inputs.
    // This initiation needs to be moved to a separate binary.
    // let prover_handle = tokio::spawn(async move { prover_handle(db_clone_3, shutdown_rx_4, start_block, prover_mode_clone).await });

    let execution_engine = tokio::spawn(async move {
        let sdk: SDK = SDK::new(&ws_url.clone()).await.expect("Failed to connect to Avail RPC");
        execution_engine_handle(
            receiver,
            node_db,
            shared_db,
            state_machine,
            prover_mode,
            shutdown_rx_2,
            state_2,
            sdk,
            app_id,
            execution_metrics,
        )
        .await
    });
    let sequencer_handle = tokio::spawn(async move {
        let sdk: SDK = SDK::new(&ws_url_clone.clone()).await.expect("Failed to connect to Avail RPC");
        sequencer_handle(mempool_clone, shutdown_rx_3, sdk, app_id).await
    });

    let result = tokio::try_join!(
        server_handle,
        execution_engine,
        relayer_handle,
        sequencer_handle,
        //   prover_handle,
    );

    match result {
        Ok((
            _,
            execution_engine_result,
            _,
            _,
            //    _
        )) => {
            info!("✅ Exited node gracefully");

            match execution_engine_result {
                Ok(()) => Ok(()),
                Err(e) => {
                    error!(error = ?e, "❌ Execution engine handle has error");
                    Err(e)
                }
            }
        }
        Err(e) => {
            error!(
                error = ?e,
                "❌ Exiting node with an error, should not have happened"
            );
            Err(anyhow!(e))
        }
    }
}
