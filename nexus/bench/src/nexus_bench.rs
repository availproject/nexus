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
    let txs: Vec<Transaction> = Vec::new();
    let state_machine = StateMachine::<ZKVM, Proof>::new(state.clone());

    let file = File::open("src/avail_header.json").unwrap();
    let reader = BufReader::new(file);
    let header: AvailHeader = from_reader(reader).unwrap();

    let file2 = File::open("src/header_store.json").unwrap();
    let reader2 = BufReader::new(file2);
    let header_store: HeaderStore = from_reader(reader2).unwrap();

    (txs, state_machine, header, header_store)
}

fn main() {
    #[cfg(any(feature = "sp1"))]
    env_logger::Builder::from_env("RUST_LOG")
        .filter_level(log::LevelFilter::Info)
        .init();

    let prover_modes = vec![ProverMode::NoAggregation, ProverMode::Compressed];

    for i in 0..prover_modes.len() {
        let (txs, mut state_machine, header, mut header_store) = create_mock_data();
        let prover_mode = &prover_modes[i.clone()];

        let start = Instant::now();

        let rt = tokio::runtime::Runtime::new().unwrap();

        rt.block_on(async {
            let (proof, header) = execute_batch::<Prover, Proof, ZKVM>(
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
