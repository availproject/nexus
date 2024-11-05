#[cfg(feature = "risc0")]
use nexus_core::zkvm::risczero::{RiscZeroProof as Proof, RiscZeroProver as Prover, ZKVM};
#[cfg(feature = "sp1")]
use nexus_core::zkvm::sp1::{Sp1Proof as Proof, Sp1Prover as Prover, SP1ZKVM as ZKVM};

#[cfg(any(feature = "sp1"))]
use env_logger;

#[cfg(any(feature = "sp1"))]
use log;


fn create_mock_data() -> {

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
        header_store,
        prover_mode,
    ) = create_mock_data();

    execute_batch::<Prover,Proof,ZKVM>()

    let duration = start.elapsed();
    println!("Proof generation took: {:?}", duration);

    let current_dir = env::current_dir().unwrap();
    let mut out_sr_path = PathBuf::from(current_dir);
    #[cfg(feature = "risc0")]
    out_sr_path.push("succinct_receipt_risc0.bin");

    #[cfg(feature = "sp1")]
    out_sr_path.push("succinct_receipt_sp1.bin");
    let serialized_data = bincode::serialize(&recursive_proof).unwrap();
    let _ = fs::write(out_sr_path.clone(), serialized_data).unwrap();

    let metadata = fs::metadata(&out_sr_path).unwrap();
    let file_size = metadata.len();
    println!("Size of the binary file: {} bytes", file_size);
}
