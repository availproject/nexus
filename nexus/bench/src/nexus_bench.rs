use std::fs;
use std::path::PathBuf;
use std::env;
use std::time::Instant;
use std::sync::Arc;
use tokio::sync::Mutex;
use nexus_core::{
    types::{
        AvailHeader, DataLookup, Digest, Extension, H256, HeaderStore,
        KateCommitment, TransactionV2, V3Extension,
    },
    state_machine::{StateMachine, VmState},
    zkvm::ProverMode,
};

// #[cfg(any(feature = "risc0", feature = "sp1"))]
use host::execute_batch;

#[cfg(feature = "risc0")]
use nexus_core::zkvm::risczero::{RiscZeroProof as Proof, RiscZeroProver as Prover, ZKVM};
#[cfg(feature = "sp1")]
use nexus_core::zkvm::sp1::{Sp1Proof as Proof, Sp1Prover as Prover, SP1ZKVM as ZKVM};

#[cfg(any(feature = "sp1"))]
use env_logger;

#[cfg(any(feature = "sp1"))]
use log;

/*

    txs: &Vec<TransactionV2>,
    state_machine: &mut StateMachine<E, P>,
    header: &AvailHeader,
    header_store: &mut HeaderStore,
    
*/

fn dummy_extension() -> Extension {
    let app_lookup = DataLookup {
        size : 0,
        index: Vec::new(),
    }; 
    let commitment = KateCommitment{
       rows : 0,
       cols : 0,
       commitment: Vec::new(),
       data_root:H256::zero(),
    }; 

    Extension::V3(V3Extension {
        app_lookup,
        commitment,
    })
}

fn create_mock_data() -> (
    &'static Vec<TransactionV2>,
    &'static mut StateMachine<ZKVM,Proof>,
    &'static AvailHeader,
    &'static mut HeaderStore
){
   let txs : Vec<TransactionV2> = Vec::new();
   
//    let vm_state = VmState::default();
//    let state = Arc::new(Mutex::new(vm_state));
   
   let state_machine = StateMachine::<ZKVM,Proof>::new(state);
   
   let header = AvailHeader {
     parent_hash : H256::zero(),
     number : 0 ,
     state_root : H256::zero(),
     extrinsics_root : H256::zero(),
     digest : Digest{
        logs : Vec::new(),
     },
     extension : dummy_extension() ,
   };
   let header_store = HeaderStore {
    inner : Vec::new(),
    max_size : 0,
   };

   (
    &txs,
    &mut state_machine,
    &header,
    &mut header_store,
   )
}

fn main() {

    #[cfg(any(feature = "sp1"))]
    env_logger::Builder::from_env("RUST_LOG")
        .filter_level(log::LevelFilter::Info)
        .init();
    
    let (
        txs,
        state_machine,
        header,
        header_store
    ) = create_mock_data();
    
    let vec = vec![ProverMode::NoAggregation, ProverMode::Compressed];

    for i in 0..2 {
    
    let prover_mode = vec[i.clone()];
     
    let start = Instant::now();

    let (_,proof)= execute_batch::<Prover,Proof,ZKVM>(txs,state_machine,header,prover_mode);

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
