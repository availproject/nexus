use anyhow::{anyhow, Context, Error};
pub use avail_subxt::Header;
use jmt::storage::TreeUpdateBatch;
use nexus_core::{
    db::{BatchTransaction, NodeDB},
    mempool::Mempool,
    state::VmState,
    state_machine::StateMachine,
    traits::NexusTransaction,
    types::{
        AvailHeader, HeaderStore, NexusBlock, NexusBlockWithPointers, NexusHeader,
        Proof as NexusProof, Transaction, TransactionResult, TransactionStatus,
        TransactionWithStatus, TransactionZKVM, TxParams, H256,
    },
    zkvm::{
        traits::{ZKVMEnv, ZKVMProof, ZKVMProver},
        ProverMode,
    },
};
use serde_json;
use std::{collections::HashMap, mem, thread};
use tokio::fs;
use tracing::{debug, error, info, instrument};

use crate::rpc::routes;
use avail_subxt::config::Header as HeaderTrait;
#[cfg(any(feature = "risc0"))]
use nexus_core::zkvm::risczero::{RiscZeroProof as Proof, RiscZeroProver as Prover, ZKVM};

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
use tokio::time::{sleep, Duration};
use warp::Filter;

pub mod rpc;
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AvailToNexusPointer {
    number: u32,
    nexus_hash: H256,
}

pub fn setup_components(db_path: &str) -> (Arc<Mutex<NodeDB>>, Arc<Mutex<VmState>>) {
    // Construct the node_db path directly as a string
    let node_db_path = format!("{}/node_db", db_path);
    let node_db = NodeDB::from_path(&node_db_path);

    // Use the runtime_db path directly as a string
    let runtime_db_path = format!("{}/runtime_db", db_path);
    let state = Arc::new(Mutex::new(VmState::new(&runtime_db_path)));

    (Arc::new(Mutex::new(node_db)), state)
}

pub async fn relayer_handle(
    relayer_mutex: Arc<Mutex<impl Relayer + Send + 'static>>,
    node_db_mutex: Arc<Mutex<NodeDB>>,
    mut shutdown_rx: watch::Receiver<bool>,
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
              },
          } + 1;

            height
        } else {
            10000
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

async fn execute_batch<
    Z: ZKVMProver<P>,
    P: ZKVMProof + Serialize + Clone + DebugTrait + TryFrom<NexusProof>,
    E: ZKVMEnv,
>(
    txs: &Vec<Transaction>,
    state_machine: &mut StateMachine<E, P>,
    header: &AvailHeader,
    header_store: &mut HeaderStore,
    prover_mode: ProverMode,
) -> Result<(P, NexusHeader, HashMap<H256, bool>, Option<TreeUpdateBatch>), Error>
where
    <P as TryFrom<NexusProof>>::Error: std::fmt::Debug,
{
    let (tree_update_batch, state_update, tx_result): (
        Option<jmt::storage::TreeUpdateBatch>,
        nexus_core::types::StateUpdate,
        HashMap<H256, bool>,
    ) = state_machine
        .execute_batch(&header, header_store, &txs)
        .await?;

    let (proof, result) = {
        #[cfg(any(feature = "sp1"))]
        let NEXUS_RUNTIME_ELF: &[u8] =
            include_bytes!("../../prover/sp1-guest/elf/riscv32im-succinct-zkvm-elf");

        let mut zkvm_prover = Z::new(NEXUS_RUNTIME_ELF.to_vec(), prover_mode);

        let zkvm_txs: Result<Vec<TransactionZKVM>, anyhow::Error> = txs
            .iter()
            .map(|tx| {
                if let TxParams::SubmitProof(submit_proof_tx) = &tx.params {
                    //TODO: Remove transactions that error out from mempool
                    let proof = submit_proof_tx.proof.clone();
                    let receipt: P = P::try_from(proof).unwrap();
                    zkvm_prover.add_proof_for_recursion(receipt).unwrap();
                }

                Ok(TransactionZKVM {
                    signature: tx.signature.clone(),
                    params: tx.params.clone(),
                })
            })
            .collect();

        let zkvm_txs = zkvm_txs?;

        zkvm_prover.add_input(&zkvm_txs).unwrap();
        zkvm_prover.add_input(&state_update).unwrap();
        zkvm_prover.add_input(&header).unwrap();
        zkvm_prover.add_input(&header_store).unwrap();
        let mut proof = zkvm_prover.prove()?;

        let result: NexusHeader = proof.public_inputs()?;
        (proof, result)
    };

    header_store.push_front(&result);

    Ok((proof, result, tx_result, tree_update_batch))
}

#[instrument(
    level = "info",
    skip(
        node_db,
        mempool,
        state_machine,
        prover_mode,
        shutdown_rx,
        state,
        receiver
    )
)]
pub async fn execution_engine_handle(
    receiver: Arc<Mutex<UnboundedReceiver<Header>>>,
    node_db: Arc<Mutex<NodeDB>>,
    mempool: Mempool,
    mut state_machine: StateMachine<ZKVM, Proof>,
    prover_mode: ProverMode,
    mut shutdown_rx: watch::Receiver<bool>,
    state: Arc<Mutex<VmState>>,
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
            debug!(
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

            let (txs, index) = mempool.get_current_txs().await;
            info!(
                avail_block = header.number,
                tx_count = txs.len(),
                mempool_index = index.unwrap_or(0),
                "📦 Starting batch processing"
            );

            debug!("🔄 Beginning batch execution");
            match execute_batch::<Prover, Proof, ZKVM>(
                &txs,
                &mut state_machine,
                &AvailHeader::from(&header),
                &mut old_headers,
                prover_mode.clone(),
            )
            .await
            {
                Ok((_, result, tx_result, tree_update_batch)) => {
                    let updated_version = state.lock().await.get_version(false)?;
                    info!(
                        nexus_block = result.number,
                        batch_hash = %hex::encode(result.hash().as_slice()),
                        state_root = %hex::encode(result.state_root.as_slice()),
                        state_version = ?updated_version,
                        "✨ Batch execution completed"
                    );

                    info!("💾 Starting batch commit");
                    match save_batch_information(
                        &node_db,
                        &mempool,
                        &mut state_machine,
                        ProcessedBatchInfo {
                            avail_header: &header,
                            header: &result,
                            txs_result: &tx_result,
                            tree_update_batch,
                            txs: &txs,
                            mempool_index: &index,
                            updated_header_store: &old_headers,
                            jmt_version: match updated_version {
                                Some(i) => i,
                                None => 0,
                            },
                        },
                    )
                    .await
                    {
                        Ok(_) => {
                            let successful_txs =
                                tx_result.values().filter(|&&success| success).count();
                            info!(
                                nexus_block = result.number,
                                batch_hash = %hex::encode(result.hash().as_slice()),
                                state_root = %hex::encode(result.state_root.as_slice()),
                                total_txs = txs.len(),
                                successful_txs = successful_txs,
                                failed_txs = txs.len() - successful_txs,
                                "✅ Batch processing completed successfully"
                            );
                        }
                        Err(e) => error!(error = ?e, "❌ Failed to commit batch"),
                    }
                }
                Err(e) => {
                    error!(error = ?e, "❌ Batch execution failed");
                    return Err(e);
                }
            }
            info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ \n");
        } else {
            debug!("Waiting for new blocks");
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }

    info!("Execution engine stopped");
    Ok(())
}

#[instrument(
    level = "debug",
    skip(node_db, mempool, state_machine, processed_batch_info)
)]
pub async fn save_batch_information<'a>(
    node_db: &Arc<Mutex<NodeDB>>,
    mempool: &Mempool,
    state_machine: &mut StateMachine<ZKVM, Proof>,
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
        let mut tx: TransactionWithStatus =
            match db_lock.get::<TransactionWithStatus>(tx_hash.as_slice())? {
                Some(i) => i,
                None => return Err(anyhow!("Tx not in db to modify.")),
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
    batch_transaction.put(
        &[nexus_hash.as_slice(), b"-block"].concat(),
        &NexusBlockWithPointers {
            block: NexusBlock {
                header: processed_batch_info.header.clone(),
                transactions: txs_result_vec,
            },
            jmt_version: processed_batch_info.jmt_version,
        },
    );
    batch_transaction.put(
        &[
            processed_batch_info.header.number.to_be_bytes().as_slice(),
            b"-block",
        ]
        .concat(),
        &nexus_hash,
    );
    let db_lock = node_db.lock().await;
    db_lock.put_batch(batch_transaction)?;

    db_lock
        .set_current_root(&processed_batch_info.header.state_root)
        .unwrap();
    if let Some(i) = processed_batch_info.mempool_index {
        mempool.clear_upto_tx(i.clone()).await;
    };

    Ok(())
}

pub struct ProcessedBatchInfo<'a> {
    avail_header: &'a Header,
    header: &'a NexusHeader,
    txs_result: &'a HashMap<H256, bool>,
    tree_update_batch: Option<TreeUpdateBatch>,
    txs: &'a Vec<Transaction>,
    mempool_index: &'a Option<usize>,
    updated_header_store: &'a HeaderStore,
    jmt_version: u64,
}

pub fn run_server(
    mempool: Mempool,
    node_db: Arc<Mutex<NodeDB>>,
    state: Arc<Mutex<VmState>>,
    mut shutdown_rx: watch::Receiver<bool>,
    port: u32,
) -> tokio::task::JoinHandle<()> {
    let routes = routes(mempool, node_db, state.clone());
    let cors = warp::cors()
        .allow_any_origin()
        .allow_methods(vec!["POST"])
        .allow_headers(vec!["content-type"]);
    let routes = routes.with(cors);

    tokio::spawn(async move {
        let address =
            SocketAddr::from_str(format!("{}:{}", String::from("127.0.0.1"), port).as_str())
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
    mut state_machine: StateMachine<ZKVM, Proof>,
    (prover_mode, server_port): (ProverMode, u32),
    state: Arc<Mutex<VmState>>,
    mut shutdown_rx: watch::Receiver<bool>,
) -> Result<(), Error> {
    let mut shutdown_rx_1 = shutdown_rx.clone();
    let mut shutdown_rx_2 = shutdown_rx.clone();
    let db_clone = node_db.clone();
    let db_clone_2 = node_db.clone();
    let state_2 = state.clone();

    let receiver = {
        let mut relayer = relayer_mutex.lock().await;

        relayer.receiver()
    };
    let mempool = Mempool::new(node_db.clone());
    let mempool_clone = mempool.clone();
    let relayer_handle = tokio::spawn(async move {
        relayer_handle(relayer_mutex, db_clone_2, shutdown_rx_1.clone()).await
    });

    let execution_engine = tokio::spawn(async move {
        execution_engine_handle(
            receiver,
            node_db,
            mempool_clone,
            state_machine,
            prover_mode,
            shutdown_rx_2.clone(),
            state_2.clone(),
        )
        .await
    });

    let server_handle = run_server(mempool, db_clone, state, shutdown_rx, server_port);

    let result = tokio::try_join!(server_handle, execution_engine, relayer_handle);

    match result {
        Ok((_, execution_engine_result, _)) => {
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
