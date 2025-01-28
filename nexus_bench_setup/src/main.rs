use adapter_sdk::{api::NexusAPI, types::AdapterConfig};
use anyhow::Error;
use geth_methods::{ADAPTER_ELF, ADAPTER_ID};
use nexus_core::db::NodeDB;
use nexus_core::state::vm_state::VmState;
use nexus_core::state_machine::StateMachine;
use nexus_core::types::{
    AccountWithProof, AppAccountId, AppId, AvailHeader, HeaderStore, InitAccount,
    NexusHeader, NexusRollupPI, StatementDigest, SubmitProof, Transaction, TxParams,
    TxSignature, H256,
};
use nexus_core::zkvm::ProverMode;
use nexus_host::execute_batch;
use risc0_zkvm::{default_prover, ExecutorEnv};
use rocksdb::Options;
use serde::{Deserialize, Serialize};
use serde_json::from_reader;
use std::env;
use std::env::args;
use std::fs;
use std::fs::File;
use std::io;
use std::io::BufReader;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::path::{Path, PathBuf};

#[cfg(feature = "risc0")]
use nexus_core::zkvm::risczero::{RiscZeroProof as Proof, RiscZeroProver as Prover, ZKVM};
#[cfg(feature = "sp1")]
use nexus_core::zkvm::sp1::{Sp1Proof as Proof, Sp1Prover as Prover, SP1ZKVM as ZKVM};

#[cfg(any(feature = "sp1"))]
use env_logger;

#[cfg(any(feature = "sp1"))]
use log;

#[derive(Clone, Serialize, Deserialize)]
struct AdapterStateData {
    last_height: u32,
    adapter_config: AdapterConfig,
}

fn create_mock_data(
    prover_mode: ProverMode,
) -> (StateMachine<ZKVM, Proof>, Vec<AvailHeader>, HeaderStore) {
    let db_path = "./db";
    let prover_mode_string = match prover_mode {
        ProverMode::NoAggregation => "no_aggregation",
        ProverMode::Compressed => "compressed",
        ProverMode::Groth16 => "groth16",
        ProverMode::MockProof => "mock_proof",
    };

    let node_db_path = format!("./db/node_db_{}", prover_mode_string);
    let runtime_db_path = format!("./db/runtime_db_{}", prover_mode_string);

    if fs::metadata(db_path).is_ok() {
        fs::remove_dir_all(db_path).expect("Failed to remove existing node_db directory");
    }

    let _node_db: NodeDB = NodeDB::from_path(&String::from(node_db_path));
    let mut db_options = Options::default();
    db_options.create_if_missing(true);

    let state = Arc::new(Mutex::new(VmState::new(&String::from(runtime_db_path))));
    let state_machine = StateMachine::<ZKVM, Proof>::new(state.clone());

    let avail_header = File::open("mock_data/avail_header.json").unwrap();
    let avail_header_reader = BufReader::new(avail_header);
    let avail_headers: Vec<AvailHeader> = from_reader(avail_header_reader).unwrap();

    let header_store: HeaderStore = HeaderStore::new(23);

    (state_machine, avail_headers, header_store)
}

async fn generate_init_account_transactions(
    header: NexusHeader,
    prover_mode: ProverMode,
    state_machine: &mut StateMachine<ZKVM, Proof>,
    avail_headers: Vec<AvailHeader>,
    header_store: &mut HeaderStore,
) -> NexusHeader {
    let mut init_account_transactions: Vec<Transaction> = Vec::new();

    for txn_index in 0..100 {
        let tx = Transaction {
            signature: TxSignature([0u8; 64]),
            params: TxParams::InitAccount(InitAccount {
                app_id: AppAccountId::from(AppId(txn_index as u32)),
                statement: StatementDigest(ADAPTER_ID),
                start_nexus_hash: header.hash(),
            }),
        };
        init_account_transactions.push(tx);
    }

    let json = serde_json::to_string_pretty(&init_account_transactions).unwrap();
    fs::write("mock_data/init_account_txns.json", json).unwrap();  

    let (_, header, _, _) = execute_batch::<Prover, Proof, ZKVM>(
        &init_account_transactions,
        state_machine,
        &avail_headers[1],
        header_store,
        prover_mode.clone(),
    )
    .await
    .unwrap();

    header
}

async fn generate_submit_proof_transactions(
    prover_mode: ProverMode,
    header: NexusHeader,
) {
    let nexus_api = NexusAPI::new(&"http://127.0.0.1:7001");
    let mut submit_proof_transactions = Vec::<Transaction>::new();
    for txn_index in 0..100 {
        let adapter_config = AdapterConfig {
            app_id: AppId(txn_index),
            elf: ADAPTER_ELF.to_vec(),
            adapter_elf_id: StatementDigest(ADAPTER_ID),
            vk: [0u8; 32],
            rollup_start_height: 606460,
            prover_mode: prover_mode.clone(),
            avail_url: String::from("wss://turing-rpc.avail.so:443/ws"),
        };

        // Retrieve or initialize the adapter state data from the database
        let adapter_state_data = AdapterStateData {
            last_height: 0,
            adapter_config,
        };

        // Main loop to fetch headers and run adapter
        let mut start_nexus_hash = None;

        let app_account_id = AppAccountId::from(adapter_state_data.adapter_config.app_id.clone());
        let account_with_proof: AccountWithProof =
            match nexus_api.get_account_state(&app_account_id.as_h256()).await {
                Ok(i) => i,
                Err(e) => {
                    println!("{:?}", e);
                    continue;
                }
            };

        let height: u32 = 30; // random height
        let public_inputs = NexusRollupPI {
            nexus_hash: header.hash(),
            state_root: H256::zero(),
            height,
            start_nexus_hash: start_nexus_hash
                .unwrap_or_else(|| H256::from(account_with_proof.account.start_nexus_hash)),
            app_id: app_account_id.clone(),
            img_id: StatementDigest(ADAPTER_ID),
            rollup_hash: Some(H256::zero()),
        };

        let mut env_builder = ExecutorEnv::builder();
        let env = env_builder.write(&public_inputs).unwrap().build().unwrap();
        let prover = default_prover();
        let prove_info = match prover.prove(env, ADAPTER_ELF) {
            Ok(i) => i,
            Err(e) => {
                println!("Unable to generate proof due to error: {:?}", e);
                continue;
            }
        };

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
                data: None,
            }),
        };

        submit_proof_transactions.push(tx.clone());
    }

    let json = serde_json::to_string_pretty(&submit_proof_transactions).unwrap();
    fs::write("mock_data/submit_proof_txns.json", json).unwrap();
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
            format!("Source directory not found: {:?}", src_absolute)
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
        
        // Check if it's a JSON file
        if path.extension().and_then(|ext| ext.to_str()) == Some("json") {
            let file_name = entry.file_name();
            let dst_path = dst_absolute.join(&file_name);
            
            // Get file size before copying
            let file_size = entry.metadata()?.len();
            
            // Copy the file
            match fs::copy(&path, &dst_path) {
                Ok(_) => {
                    files_copied += 1;
                    total_size += file_size;
                },
                Err(e) => eprintln!("Error copying {:?}: {}", file_name, e),
            }
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let args: Vec<String> = args().collect();

    if args.len() <= 1 {
        if args.len() < 1 {
            eprintln!("Usage: cargo run -- <prover_mode [compressed, no_aggregation]>");
            return Ok(());
        }

        if !["compressed", "no_aggregation"].contains(&args[0].as_str()) {
            eprintln!("Usage: cargo run -- <prover_mode [compressed, no_aggregation]>");
            return Ok(());
        }
    }

    let mut prover_mode: ProverMode = ProverMode::Compressed;
    let (mut state_machine, avail_headers, mut header_store) =
        create_mock_data(prover_mode.clone());
    let mock_txs: Vec<Transaction> = Vec::new();

    prover_mode = match args[1].as_str() {
        "compressed" => ProverMode::Compressed,
        "no_aggregation" => ProverMode::NoAggregation,
        "groth16" => ProverMode::Groth16,
        _ => {
            eprintln!("Usage: cargo run -- <prover_mode [compressed, no_aggregation, groth16]>");
            return Ok(());
        }
    };

    let (_, header, _, _) = execute_batch::<Prover, Proof, ZKVM>(
        &mock_txs,
        &mut state_machine,
        &avail_headers[0],
        &mut header_store,
        prover_mode.clone(),
    )
    .await
    .unwrap();

    let header = generate_init_account_transactions(
        header.clone(),
        prover_mode.clone(),
        &mut state_machine,
        avail_headers.clone(),
        &mut header_store,
    ).await;

    generate_submit_proof_transactions(
        prover_mode.clone(),
        header
    ).await;

    let src_dir = Path::new("mock_data");
    let dst_dir = Path::new("../nexus/bench/mock_data");

    match move_mock_data(src_dir, dst_dir) {
        Ok(_) => println!("Mock data generated and moved successfully"),
        Err(e) => eprintln!("Error copying mock data: {}", e),
    }

    return Ok(());
}
