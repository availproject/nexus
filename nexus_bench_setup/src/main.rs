use adapter_sdk::{api::NexusAPI, types::AdapterConfig};
use anyhow::anyhow;
use anyhow::Error;
use bincode;
use mock_elf::{MOCK_GUEST_RISC0_ELF, MOCK_GUEST_RISC0_ID};
use nexus_core::db::NodeDB;
use nexus_core::state::vm_state::VmState;
use nexus_core::state_machine::StateMachine;
use nexus_core::types::{
    AccountWithProof, AppAccountId, AppId, AvailHeader, Blob, BlobProof, CompactDataLookup, DataLookupItem, Digest, HeaderExtension, HeaderStore,
    InitAccount, KateCommitment, NexusHeader, NexusRollupPI, StatementDigest, SubmitProof, Transaction, TxParams, TxSignature, V3Extension, H256,
};
use nexus_core::zkvm::ProverMode;
use nexus_host::execute_batch;
use risc0_zkvm::{default_prover, ExecutorEnv};
use rocksdb::Options;
use serde::{Deserialize, Serialize};
use serde_json::from_reader;
use std::env;
use std::fs;
use std::fs::File;
use std::io;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;

#[cfg(feature = "risc0")]
use nexus_core::zkvm::risczero::{RiscZeroProof as Proof, RiscZeroProver as Prover, ZKVM};
#[cfg(feature = "sp1")]
use nexus_core::zkvm::sp1::{Sp1Proof as Proof, Sp1Prover as Prover, SP1ZKVM as ZKVM};

#[cfg(any(feature = "sp1"))]
use env_logger;

#[cfg(any(feature = "sp1"))]
use log;

use kate::gridgen::AsBytes;
use kate::gridgen::EvaluationGrid;
use kate::{couscous, Seed};
use kzg::compute_row_proof;
use rand::Rng;
use sp_runtime::traits::BlakeTwo256;

#[derive(Clone, Serialize, Deserialize)]
struct AdapterStateData {
    last_height: u32,
    adapter_config: AdapterConfig,
}

fn create_mock_data(prover_mode: ProverMode) -> (StateMachine<Proof>, HeaderStore) {
    let db_path = "./db";

    let runtime_db_path = String::from("./db/runtime_db");

    if fs::metadata(db_path).is_ok() {
        fs::remove_dir_all(db_path).expect("Failed to remove existing node_db directory");
    }

    let mut db_options = Options::default();
    db_options.create_if_missing(true);

    let state = Arc::new(Mutex::new(VmState::new(&String::from(runtime_db_path))));
    let state_machine = StateMachine::<Proof>::new(state.clone());

    let header_store: HeaderStore = HeaderStore::new(23);

    (state_machine, header_store)
}

async fn generate_init_account_transactions(header: NexusHeader) -> Vec<Transaction> {
    let mut init_account_transactions: Vec<Transaction> = Vec::new();

    for txn_index in 0..40 {
        let tx = Transaction {
            signature: TxSignature([0u8; 64]),
            params: TxParams::InitAccount(InitAccount {
                app_id: AppAccountId::from(AppId(txn_index as u32)),
                statement: StatementDigest(MOCK_GUEST_RISC0_ID),
                start_nexus_hash: header.hash(),
            }),
        };
        init_account_transactions.push(tx);
    }

    init_account_transactions
}

async fn generate_submit_proof_transactions(prover_mode: ProverMode, header: NexusHeader, start_nexus_hash: H256) -> Vec<Transaction> {
    let mut submit_proof_transactions = Vec::<Transaction>::new();
    let prover = default_prover();
    //Benchmarking 15 transactions as any more will take create a new row on AvailDA.
    for txn_index in 0..15 {
        let app_account_id = AppAccountId::from(AppId(txn_index));

        let height: u32 = 30; // random height
        let public_inputs = NexusRollupPI {
            nexus_hash: header.hash(),
            state_root: H256::zero(),
            height,
            start_nexus_hash: start_nexus_hash.clone(),
            app_id: app_account_id.clone(),
            img_id: StatementDigest(MOCK_GUEST_RISC0_ID),
            rollup_hash: Some(H256::zero()),
        };
        let public_inputs_serialized = bincode::serialize(&public_inputs).expect("Failed to serialize public inputs");

        let mock_prover_env = ExecutorEnv::builder()
            .write(&public_inputs_serialized.len())
            .expect("Error writing to mock prover env")
            .write_slice(&public_inputs_serialized)
            .build()
            .expect("Error building mock prover env");

        println!("Starting proof gen for txn {}", txn_index);

        let prove_info = prover.prove(mock_prover_env, &MOCK_GUEST_RISC0_ELF).expect("Error when proving");

        let deserialized_pi: NexusRollupPI = bincode::deserialize(&prove_info.receipt.journal.bytes).expect("Failed to deserialize public inputs");

        let recursive_proof = Proof(prove_info.receipt);

        let tx = Transaction {
            signature: TxSignature([0u8; 64]),
            params: TxParams::SubmitProof(SubmitProof {
                app_id: app_account_id.clone(),
                nexus_hash: header.hash(),
                state_root: public_inputs.state_root.clone(),
                proof: match recursive_proof.clone().try_into() {
                    Ok(i) => i,
                    Err(e) => {
                        println!("Unable to serialise proof: {:?}", e);
                        continue;
                    }
                },
                height: public_inputs.height,
                data: Some(H256::zero()),
            }),
        };

        submit_proof_transactions.push(tx.clone());
    }

    submit_proof_transactions
}

fn get_absolute_path(path: &Path) -> io::Result<PathBuf> {
    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        Ok(env::current_dir()?.join(path))
    }
}

fn move_mock_data(src_dir: &Path, dst_dir: &Path) -> io::Result<()> {
    // Convert to absolute paths
    let src_absolute = get_absolute_path(src_dir)?;
    let dst_absolute = get_absolute_path(dst_dir)?;

    // Verify source directory exists
    if !src_absolute.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Source directory not found: {:?}", src_absolute),
        ));
    }

    // Create the destination directory if it doesn't exist
    if !dst_absolute.exists() {
        fs::create_dir_all(&dst_absolute)?;
    }

    let mut files_copied = 0;
    let mut total_size = 0;

    // Read all entries in the source directory
    for entry in fs::read_dir(&src_absolute)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|ext| ext.to_str()) == Some("bin") {
            let file_name = entry.file_name();
            let dst_path = dst_absolute.join(&file_name);

            // Get file size before copying
            let file_size = entry.metadata()?.len();

            // Copy the file
            match fs::copy(&path, &dst_path) {
                Ok(_) => {
                    files_copied += 1;
                    total_size += file_size;
                }
                Err(e) => eprintln!("Error copying {:?}: {}", file_name, e),
            }
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let app_id: u32 = 200;

    let mut prover_mode: ProverMode = ProverMode::Compressed;
    let (mut state_machine, mut header_store) = create_mock_data(prover_mode.clone());
    let mock_txs: Vec<Transaction> = Vec::new();

    let (avail_header_1, blobs, blobs_proof) = generate_header_and_proofs(&mock_txs, app_id, H256::zero());

    let (zkvm_inputs_1, header_1, _, tree_update_batch_opt) = execute_batch::<Prover, Proof, ZKVM>(
        &blobs,
        &blobs_proof,
        &mut state_machine,
        &avail_header_1,
        &mut header_store,
        app_id,
    )
    .await
    .unwrap();

    if let Some(tree_update_batch) = tree_update_batch_opt {
        state_machine
            .commit_state(
                &header_1.state_root,
                &tree_update_batch.node_batch,
                header_1.number,
            )
            .await
            .unwrap();
    }

    let init_txs = generate_init_account_transactions(header_1.clone()).await;

    let (avail_header_2, blobs, blobs_proof) = generate_header_and_proofs(&init_txs, app_id, avail_header_1.hash().clone());
    let (zkvm_inputs_2, header_2, _, tree_update_batch_opt) = execute_batch::<Prover, Proof, ZKVM>(
        &blobs,
        &blobs_proof,
        &mut state_machine,
        &avail_header_2,
        &mut header_store,
        app_id,
    )
    .await
    .unwrap();

    if let Some(tree_update_batch) = tree_update_batch_opt {
        state_machine
            .commit_state(
                &header_2.state_root,
                &tree_update_batch.node_batch,
                header_2.number,
            )
            .await
            .unwrap();
    }

    let submit_proof_txs = generate_submit_proof_transactions(prover_mode.clone(), header_2.clone(), header_1.hash()).await;
    let (avail_header_3, blobs, blobs_proof) = generate_header_and_proofs(&submit_proof_txs, app_id, avail_header_2.hash().clone());
    let (zkvm_inputs_3, header_3, _, tree_update_batch_opt) = execute_batch::<Prover, Proof, ZKVM>(
        &blobs,
        &blobs_proof,
        &mut state_machine,
        &avail_header_3,
        &mut header_store,
        app_id,
    )
    .await
    .unwrap();

    let suffix = if cfg!(feature = "risc0") { "risc0" } else { "sp1" };
    let zkvm_inputs_1_path = format!("mock_data/zkvm_inputs_1_{}.bin", suffix);
    std::fs::create_dir_all("mock_data").unwrap();
    std::fs::write(
        &zkvm_inputs_1_path,
        bincode::serialize(&(zkvm_inputs_1, header_1)).unwrap(),
    )
    .unwrap();
    println!("Saved zkvm_inputs_1 to {}", zkvm_inputs_1_path);

    let zkvm_inputs_2_path = format!("mock_data/zkvm_inputs_2_{}.bin", suffix);
    std::fs::write(
        &zkvm_inputs_2_path,
        bincode::serialize(&(zkvm_inputs_2, header_2)).unwrap(),
    )
    .unwrap();
    println!("Saved zkvm_inputs_2 to {}", zkvm_inputs_2_path);

    let zkvm_inputs_3_path = format!("mock_data/zkvm_inputs_3_{}.bin", suffix);
    std::fs::write(
        &zkvm_inputs_3_path,
        bincode::serialize(&(zkvm_inputs_3, header_3)).unwrap(),
    )
    .unwrap();
    println!("Saved zkvm_inputs_3 to {}", zkvm_inputs_3_path);

    let src_dir = Path::new("mock_data");
    let dst_dir = Path::new("../nexus/bench/mock_data");

    match move_mock_data(src_dir, dst_dir) {
        Ok(_) => println!("Mock data generated and moved successfully"),
        Err(e) => eprintln!("Error copying mock data: {}", e),
    }

    return Ok(());
}

fn generate_header_and_proofs(txs: &Vec<Transaction>, app_id: u32, parent_hash: H256) -> (AvailHeader, Vec<Blob>, Vec<BlobProof>) {
    let mut rng = rand::thread_rng();
    let tx_size: usize = 31 * 1024;
    let block_number: u32 = 0;
    let data_for_block: Vec<u8> = match txs.len() {
        0 => (0..tx_size).map(|_| rng.random::<u8>()).collect(),
        _ => {
            let mut blob: Vec<u8> = bincode::serialize(txs).unwrap();
            if (blob.len() > tx_size) {
                panic!(
                    "Serialized blob will extend to more than one row as {} > {}",
                    blob.len(),
                    tx_size
                );
            }
            blob
        }
    };

    let grid = EvaluationGrid::from_data(data_for_block, 1024, 1024, 1024, Seed::default()).expect("Failed to create evaluation grid");

    let poly_grid = grid.make_polynomial_grid().expect("Make polynomial grid failed");

    let public_params = couscous::multiproof_params();
    let extended_grid = poly_grid.commitments(&public_params).expect("Failed to generate commitments");

    let commitments: Vec<u8> = extended_grid.iter().flat_map(|c| c.to_bytes().ok()).flatten().collect();

    let mut nexus_blobs: Vec<Blob> = Vec::new();

    if txs.len() != 0 {
        for row_num in 0..grid.dims().rows().get() {
            let row = grid
                .row(row_num.into())
                .expect(&format!(
                    "Row {} should exist in grid but was not found",
                    row_num
                ))
                .into_iter()
                .map(|c| c.to_bytes().map_err(|e| anyhow!("Failed to convert cell to bytes: {:?}", e)))
                .collect::<Result<Vec<_>, _>>()
                .expect("Failed to convert row to bytes");

            nexus_blobs.push(Blob(row));
        }
    }

    let dimension = grid.dims();

    let data_lookup = if txs.len() == 0 {
        CompactDataLookup {
            size: dimension.rows().get() as u32,
            index: vec![DataLookupItem { app_id: AppId(1000), start: 0 }],
            rows_per_tx: vec![dimension.rows().get()],
        }
    } else {
        CompactDataLookup {
            size: dimension.rows().get() as u32,
            index: vec![DataLookupItem { app_id: AppId(app_id), start: 0 }],
            rows_per_tx: vec![dimension.rows().get()],
        }
    };

    let header: AvailHeader = AvailHeader {
        parent_hash,
        number: block_number,
        state_root: H256::zero(),
        extrinsics_root: H256::zero(),
        digest: Digest::default(),
        extension: HeaderExtension::V3(V3Extension {
            app_lookup: data_lookup,
            commitment: KateCommitment {
                rows: dimension.rows().get(),
                cols: dimension.cols().get(),
                commitment: commitments,
                data_root: H256::zero(),
            },
        }),
    };

    let proofs = nexus_blobs
        .iter()
        .map(|blob| -> Result<BlobProof, anyhow::Error> {
            let proof = compute_row_proof(&blob.0)?;
            Ok(BlobProof(proof))
        })
        .collect::<Result<Vec<_>, _>>()
        .expect("Could not generate row proofs");

    (header, nexus_blobs, proofs)
}
