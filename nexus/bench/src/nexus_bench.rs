use nexus_core::{
    db::NodeDB,
    state::vm_state::VmState,
    state_machine::StateMachine,
    types::{
        AvailHeader, DataLookup, Digest, DigestItem, Extension, HeaderStore, KateCommitment,
        NexusHeader, Transaction, V3Extension, H256,
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

//@TODO : use mockproofs for bench
fn create_mock_data() -> (
    Vec<Transaction>,
    StateMachine<ZKVM, Proof>,
    AvailHeader,
    HeaderStore,
) {
    let _node_db: NodeDB = NodeDB::from_path(&String::from("./db/node_db"));
    let mut db_options = Options::default();
    db_options.create_if_missing(true);

    let state = Arc::new(Mutex::new(VmState::new(&String::from("./db/runtime_db"))));
    let mut txs: Vec<Transaction> = Vec::new();
    let state_machine = StateMachine::<ZKVM, Proof>::new(state.clone());

    // Read all file names from the transactions directory
    let dir_path = "src/transactions";
    let mut files = fs::read_dir(dir_path)
        .unwrap()
        .filter_map(|entry| {
            entry.ok().and_then(|e| {
                let path = e.path();
                if path.is_file() && path.extension().unwrap_or_default() == "json" {
                    path.file_name()
                        .map(|name| name.to_string_lossy().into_owned())
                } else {
                    None
                }
            })
        })
        .collect::<Vec<_>>();

    // Sort files to ensure consistent ordering
    files.sort();

    let num_txns = 300; // desired number of transactions
    let files_len = files.len(); // actual number of files available

    for tx in 0..num_txns {
        // Use modulo to loop back to start when we run out of files
        let file_index = tx % files_len;
        let file_path = format!("{}/{}", dir_path, files[file_index]);
        let tx_file = File::open(&file_path).unwrap();
        let tx_reader = BufReader::new(tx_file);
        let tx: Transaction = from_reader(tx_reader).unwrap();
        txs.push(tx);
    }

    let avail_header = File::open("src/avail_header.json").unwrap();
    let avail_header_reader = BufReader::new(avail_header);
    let header: AvailHeader = from_reader(avail_header_reader).unwrap();

    let header_store = File::open("src/header_store.json").unwrap();
    let header_store_reader = BufReader::new(header_store);
    let header_store: HeaderStore = from_reader(header_store_reader).unwrap();

    (txs, state_machine, header, header_store)
}

fn main() {
    #[cfg(any(feature = "sp1"))]
    env_logger::Builder::from_env("RUST_LOG")
        .filter_level(log::LevelFilter::Info)
        .init();

    let prover_modes = vec![ProverMode::NoAggregation, ProverMode::Compressed];

    for mode in 0..prover_modes.len() {
        let (txs, mut state_machine, header, mut header_store) = create_mock_data();
        let prover_mode = &prover_modes[mode.clone()];

        let start = Instant::now();

        let rt = tokio::runtime::Runtime::new().unwrap();

        rt.block_on(async {
            let (proof, header, tx_result, tree_update_batch) =
                execute_batch::<Prover, Proof, ZKVM>(
                    &txs,
                    &mut state_machine,
                    &header,
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
        })
    }
}
