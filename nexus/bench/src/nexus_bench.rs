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
    
    let db_path = "./db/node_db";
    let runtime_db_path = "./db/runtime_db";

    // Remove the database directory if it exists
    if fs::metadata(db_path).is_ok() {
        fs::remove_dir_all(db_path).expect("Failed to remove existing node_db directory");
    }

    // Create a new RocksDB instance
    let _node_db: NodeDB = NodeDB::from_path(&String::from(db_path));
    let mut db_options = Options::default();
    db_options.create_if_missing(true);

    // Remove runtime_db directory if it exists
    if fs::metadata(runtime_db_path).is_ok() {
        fs::remove_dir_all(runtime_db_path).expect("Failed to remove existing runtime_db directory");
    }

    let state = Arc::new(Mutex::new(VmState::new(&String::from(runtime_db_path))));
    let mut txs: Vec<Transaction> = Vec::new();
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
                    start_nexus_hash: header_1.hash(),
                }),
            };
            let json_string = serde_json::to_string_pretty(&tx).unwrap();
            let file_name = format!("src/init_account_transactions/init_account_txn_{}.json", txn_index);
            fs::write(file_name, json_string).unwrap();
            match nexus_api.send_tx(tx).await {
                Ok(i) => {
                    println!("Completed init account for app id {:?}", txn_index);
                }
                Err(e) => {
                    println!("Error when iniating account: {:?}", e);
                    continue;
                }
            }
        }

        let (_, header, _, _) = execute_batch::<Prover, Proof, ZKVM>(
            &mock_txs,
            &mut state_machine,
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

    for mode in 0..prover_modes.len() {
        let (_, mut state_machine, avail_headers, mut header_store) = create_mock_data();
        let prover_mode = &prover_modes[mode.clone()];

        {
            let start = Instant::now();
    
            let (proof, _, _, _) = execute_batch::<Prover, Proof, ZKVM>(
                &init_account_transactions,
                &mut state_machine,
                &avail_headers[0],
                &mut header_store,
                prover_mode.clone(),
            )
            .await
            .unwrap();
    
            let duration = start.elapsed();
            println!("Proof generation took: {:?}", duration);
    
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
            println!("Size of the binary file: {} bytes", file_size);
        }

        {
            let start = Instant::now();
    
            let (proof, header, tx_result, tree_update_batch) = execute_batch::<Prover, Proof, ZKVM>(
                &submit_proof_transactions,
                &mut state_machine,
                &avail_headers[1],
                &mut header_store,
                prover_mode.clone(),
            )
            .await
            .unwrap();
    
            let duration = start.elapsed();
            println!("Proof generation took: {:?}", duration);
    
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
            println!("Size of the binary file: {} bytes", file_size);
        }
    }
}
