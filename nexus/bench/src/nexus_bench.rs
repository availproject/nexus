use nexus_core::{
    db::NodeDB,
    state::vm_state::VmState,
    state_machine::StateMachine,
    types::{
        AvailHeader, DataLookup, Digest, DigestItem, Extension, HeaderStore, KateCommitment,
        NexusHeader, TransactionV2, V3Extension, H256,
    },
    zkvm::ProverMode,
};
use nexus_host::execute_batch;
use rocksdb::Options;
use std::env;
use std::fs;
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
    Vec<TransactionV2>,
    StateMachine<ZKVM, Proof>,
    AvailHeader,
    HeaderStore,
) {
    let _node_db: NodeDB = NodeDB::from_path(&String::from("./db/node_db"));
    let mut db_options = Options::default();
    db_options.create_if_missing(true);

    let state = Arc::new(Mutex::new(VmState::new(&String::from("./db/runtime_db"))));
    let txs: Vec<TransactionV2> = Vec::new();
    let state_machine = StateMachine::<ZKVM, Proof>::new(state.clone());
    
    let file = File::open("avail_header.json")?;
    let header = AvailHeader = from_reader(file)?;

    let file2 = File::open("header_store.json")?;
    let header_store = HeaderStore = from_reader(file2)?;

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
        let prover_mode = &vec[i.clone()];

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
