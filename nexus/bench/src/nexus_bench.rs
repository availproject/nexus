use geth_methods::ADAPTER_ID;
use nexus_core::{
    db::NodeDB,
    state::vm_state::VmState,
    state_machine::StateMachine,
    types::{
        AppAccountId, AppId, AvailHeader, HeaderStore,
        InitAccount, NexusHeader, StatementDigest,
        Transaction, TxParams, TxSignature,
    },
    zkvm::ProverMode,
};
use nexus_host::execute_batch;
use rocksdb::Options;
use serde_json::from_reader;
use std::env;
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;

#[cfg(feature = "risc0")]
use nexus_core::zkvm::risczero::{RiscZeroProof as Proof, RiscZeroProver as Prover, ZKVM};
#[cfg(feature = "sp1")]
use nexus_core::zkvm::sp1::{Sp1Proof as Proof, Sp1Prover as Prover, SP1ZKVM as ZKVM};

#[cfg(any(feature = "sp1"))]
use env_logger;

#[cfg(any(feature = "sp1"))]
use log;

fn create_mock_data() -> (
    StateMachine<ZKVM, Proof>,
    Vec<AvailHeader>,
    HeaderStore,
) {
    let _node_db: NodeDB = NodeDB::from_path(&String::from("./db/node_db"));
    let mut db_options = Options::default();
    db_options.create_if_missing(true);

    let state = Arc::new(Mutex::new(VmState::new(&String::from("./db/runtime_db"))));
    let state_machine = StateMachine::<ZKVM, Proof>::new(state.clone());

    let avail_header = File::open("src/avail_header.json").unwrap();
    let avail_header_reader = BufReader::new(avail_header);
    let avail_headers: Vec<AvailHeader> = from_reader(avail_header_reader).unwrap();

    let header_store = File::open("src/header_store.json").unwrap();
    let header_store_reader = BufReader::new(header_store);
    let header_store: HeaderStore = from_reader(header_store_reader).unwrap();

    (state_machine, avail_headers, header_store)
}

async fn bench_init_account_transactions(header: NexusHeader, prover_mode: ProverMode, state_machine: &mut StateMachine<ZKVM, Proof>, avail_headers: Vec<AvailHeader>, mut header_store: HeaderStore) -> Proof {
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

    let (proof, header, _, _) = execute_batch::<Prover, Proof, ZKVM>(
        &init_account_transactions,
        state_machine,
        &avail_headers[1],
        &mut header_store,
        prover_mode.clone(),
    )
    .await
    .unwrap();   

    // store header into json file
    let json_string = serde_json::to_string_pretty(&header).unwrap();
    fs::write("src/headers/nexus_header.json", json_string).unwrap();

    proof
}

async fn bench_submit_proof_transactions(prover_mode: ProverMode, state_machine: &mut StateMachine<ZKVM, Proof>, avail_headers: Vec<AvailHeader>, mut header_store: HeaderStore) -> Proof {
    let mut submit_proof_transactions: Vec<Transaction> = Vec::new();
    for txn_index in 0..100 {
        let file_name = format!("src/submit_proof_transactions/submit_proof_transaction_{}.json", txn_index);
        let file_content = fs::read_to_string(file_name).unwrap();
        let tx: Transaction = serde_json::from_str(&file_content).unwrap();
        submit_proof_transactions.push(tx);
    }
    let (proof, _, _, _) = execute_batch::<Prover, Proof, ZKVM>(
        &submit_proof_transactions,
        state_machine,
        &avail_headers[1],
        &mut header_store,
        prover_mode.clone(), 
    )
    .await
    .unwrap();

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
async fn main() {
    #[cfg(any(feature = "sp1"))]
    env_logger::Builder::from_env("RUST_LOG")
        .filter_level(log::LevelFilter::Info)
        .init();

    let (mut state_machine, avail_headers, mut header_store) = create_mock_data();
    let mock_txs: Vec<Transaction> = Vec::new();
    let no_aggregation_prover_mode = ProverMode::NoAggregation;
    let compressed_prover_mode = ProverMode::Compressed;

    let (_, header, _, _) = execute_batch::<Prover, Proof, ZKVM>(
        &mock_txs,
        &mut state_machine,
        &avail_headers[0],
        &mut header_store,
        no_aggregation_prover_mode.clone(),
    )
    .await
    .unwrap();

    let init_account_time_start = Instant::now();
    // bench how much time it takes to with 100 init account transactions with no aggregation prover mode
    let mut proof = bench_init_account_transactions(header.clone(), no_aggregation_prover_mode.clone(), &mut state_machine, avail_headers.clone(), header_store.clone()).await;
    let init_account_transactions_duration = init_account_time_start.elapsed();
    println!("Proof generation time for Init account transactions with prover mode no aggregation took: {:?}", init_account_transactions_duration);

    let mut file_size = get_proof_size(proof);
    println!("Size of the Proof Binary: {} bytes", file_size);

    let submit_account_time_start = Instant::now();
    // bench how much time it take to work with 100 submit proof transactions with no aggregation prover mode
    proof = bench_submit_proof_transactions(no_aggregation_prover_mode.clone(),  &mut state_machine, avail_headers.clone(), header_store.clone()).await;
    let submit_account_transactions_duration = submit_account_time_start.elapsed();
    println!("Proof generation time for Submit account transactions with prover mode no aggregation took: {:?}", submit_account_transactions_duration);
    
    file_size = get_proof_size(proof);
    println!("Size of the Proof Binary: {} bytes", file_size);

    let init_account_time_start = Instant::now();
    // bench how much time it takes to with 100 init account transactions with compressed prover mode
    proof = bench_init_account_transactions(header.clone(), compressed_prover_mode.clone(), &mut state_machine, avail_headers.clone(), header_store.clone()).await;
    let init_account_transactions_duration = init_account_time_start.elapsed();
    println!("Proof generation time for Init account transactions with compressed prover mode took: {:?}", init_account_transactions_duration);

    file_size = get_proof_size(proof);
    println!("Size of the Proof Binary: {} bytes", file_size);

    let submit_account_time_start = Instant::now();
    // bench how much time it take to work with 100 submit proof transactions with compressed prover mode
    proof = bench_submit_proof_transactions(compressed_prover_mode.clone(), &mut state_machine, avail_headers.clone(), header_store.clone()).await;
    let submit_account_transactions_duration = submit_account_time_start.elapsed();
    println!("Proof generation took for Submit account transactions with compressed prover mode took: {:?}", submit_account_transactions_duration);

    file_size = get_proof_size(proof);
    println!("Size of the Proof Binary: {} bytes", file_size);
}
