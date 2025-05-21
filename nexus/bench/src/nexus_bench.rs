use anyhow::Error;
use nexus_core::{
    db::NodeDB,
    state::vm_state::VmState,
    state_machine::StateMachine,
    types::{
        AppAccountId, AppId, AvailHeader, HeaderStore, InitAccount, NexusHeader, NexusZKVMInputs, StatementDigest, Transaction, TxParams, TxSignature,
    },
    zkvm::ProverMode,
};
use nexus_host::{execute_batch, prove_batch};
use rocksdb::Options;
use serde_json::from_reader;
use std::env::args;
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use std::{any, env};
use tokio::sync::Mutex;

#[cfg(feature = "risc0")]
use nexus_core::zkvm::risczero::{RiscZeroProof as Proof, RiscZeroProver as Prover, ZKVM};
#[cfg(feature = "sp1")]
use nexus_core::zkvm::sp1::{Sp1Proof as Proof, Sp1Prover as Prover, SP1ZKVM as ZKVM};

#[cfg(any(feature = "sp1"))]
use env_logger;

#[cfg(any(feature = "sp1"))]
use log;

fn create_mock_data(prover_mode: ProverMode) -> (StateMachine<Proof>, Vec<AvailHeader>, HeaderStore) {
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
    let state_machine = StateMachine::<Proof>::new(state.clone());

    let avail_header = File::open("../mock_data/avail_header.json").unwrap();
    let avail_header_reader = BufReader::new(avail_header);
    let avail_headers: Vec<AvailHeader> = from_reader(avail_header_reader).unwrap();

    let header_store: HeaderStore = HeaderStore::new(23);

    (state_machine, avail_headers, header_store)
}

async fn bench_init_account_transactions(prover_mode: ProverMode) -> Proof {
    let suffix = if cfg!(feature = "risc0") { "risc0" } else { "sp1" };
    let file_path = format!("./mock_data/zkvm_inputs_2_{}.bin", suffix);
    let bytes = std::fs::read(&file_path).expect(&format!("Failed to read file: {}", file_path));
    let (zkvm_inputs_2, nexus_header_2): (NexusZKVMInputs, NexusHeader) = bincode::deserialize(&bytes).expect("Failed to decode zkvm_inputs_2");

    let proof = prove_batch::<Prover, Proof, ZKVM>(zkvm_inputs_2, nexus_header_2, &prover_mode)
        .await
        .expect("Failed to prove zkvm_inputs_2");

    proof
}

async fn bench_submit_proof_transactions(prover_mode: ProverMode) -> Proof {
    let suffix = if cfg!(feature = "risc0") { "risc0" } else { "sp1" };
    let file_path = format!("./mock_data/zkvm_inputs_3_{}.bin", suffix);
    let bytes = std::fs::read(&file_path).expect(&format!("Failed to read file: {}", file_path));
    let (zkvm_inputs_3, nexus_header_3): (NexusZKVMInputs, NexusHeader) = bincode::deserialize(&bytes).expect("Failed to decode zkvm_inputs_3");

    let proof = prove_batch::<Prover, Proof, ZKVM>(zkvm_inputs_3, nexus_header_3, &prover_mode)
        .await
        .expect("Failed to prove zkvm_inputs_3");

    proof
}

fn get_proof_size(proof: Proof) -> u64 {
    let current_dir = env::current_dir().unwrap();
    let mut out_sr_path = PathBuf::from(current_dir);
    #[cfg(feature = "risc0")]
    out_sr_path.push("succinct_receipt_risc0.bin");

    #[cfg(feature = "sp1")]
    out_sr_path.push("succinct_receipt_sp1.bin");
    let serialized_data = bincode::serialize(&proof).unwrap();
    let _ = fs::write(out_sr_path.clone(), serialized_data).unwrap();
    let metadata = fs::metadata(&out_sr_path).unwrap();
    let file_size = metadata.len();
    file_size
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    #[cfg(any(feature = "sp1"))]
    env_logger::Builder::from_env("RUST_LOG").filter_level(log::LevelFilter::Info).init();

    let suffix = if cfg!(feature = "risc0") { "risc0" } else { "sp1" };

    let prover_mode_param = env::var("PROVER_MODE").unwrap_or_else(|_| "default".to_string());

    if !["compressed", "no_aggregation", "groth16", "mock_proof"].contains(&prover_mode_param.as_str()) {
        eprintln!("Usage: PROVER_MODE=<compressed, no_aggregation> cargo bench");
        return Ok(());
    }

    let mut prover_mode = ProverMode::Compressed;
    prover_mode = match prover_mode_param.as_str() {
        "compressed" => ProverMode::Compressed,
        "no_aggregation" => ProverMode::NoAggregation,
        "groth16" => ProverMode::Groth16,
        "mock_proof" => ProverMode::MockProof,
        _ => {
            eprintln!("Usage: PROVER_MODE=<compressed, no_aggregation> cargo bench");
            return Ok(());
        }
    };

    let file_path = format!("./mock_data/zkvm_inputs_1_{}.bin", suffix);
    let bytes = std::fs::read(&file_path).expect(&format!("Failed to read file: {}", file_path));
    let (zkvm_inputs_1, nexus_header_1): (NexusZKVMInputs, NexusHeader) = bincode::deserialize(&bytes).expect("Failed to decode zkvm_inputs_1");
    let proof = prove_batch::<Prover, Proof, ZKVM>(zkvm_inputs_1, nexus_header_1, &prover_mode)
        .await
        .expect("Failed to prove zkvm_inputs_1");

    // let init_account_time_start = Instant::now();

    // let proof = bench_init_account_transactions(prover_mode.clone()).await;

    // let init_account_transactions_duration = init_account_time_start.elapsed();
    // println!(
    //     "Proof generation time for Init account transactions with prover mode {:?} took: {:?}",
    //     prover_mode_param, init_account_transactions_duration
    // );

    // let file_size = get_proof_size(proof);
    // println!("Size of the Proof Binary: {} bytes", file_size);

    let submit_account_time_start = Instant::now();

    let proof = bench_submit_proof_transactions(prover_mode.clone()).await;

    let submit_account_transactions_duration = submit_account_time_start.elapsed();
    println!(
        "Proof generation time for Submit account transactions with prover mode {:?} took: {:?}",
        prover_mode_param, submit_account_transactions_duration
    );

    let file_size = get_proof_size(proof);
    println!("Size of the Proof Binary: {} bytes", file_size);

    return Ok(());
}
