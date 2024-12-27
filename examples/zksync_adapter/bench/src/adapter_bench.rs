use nexus_core::types::{AccountState, AppAccountId, StatementDigest, H256 as NexusH256};
#[cfg(feature = "risc0")]
use nexus_core::zkvm::risczero::{RiscZeroProof as Proof, RiscZeroProver as Prover, ZKVM};
#[cfg(feature = "sp1")]
use nexus_core::zkvm::sp1::{Sp1Proof as Proof, Sp1Prover as Prover, SP1ZKVM as ZKVM};
use nexus_core::zkvm::ProverMode;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::time::Instant;
use zksync_core::types::L1BatchWithMetadata;
use zksync_core::STF;
#[cfg(feature = "risc0")]
use zksync_methods::{ZKSYNC_ADAPTER_ELF, ZKSYNC_ADAPTER_ID};

#[cfg(any(feature = "sp1"))]
use env_logger;

#[cfg(any(feature = "sp1"))]
use log;

use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::io::{self};

pub mod constants;

use crate::constants::{get_mock_l1_batch_with_metadata, get_mock_proof};
//@TODO : use mockproofs for bench

fn create_mock_data() -> (
    Option<Proof>,
    Option<(AppAccountId, AccountState)>,
    Vec<String>,
    L1BatchWithMetadata,
    Vec<u8>,
    Vec<[u8; 32]>,
    NexusH256,
) {
    let prev_adapter_proof = None;
    let init_account = Some((
        AppAccountId([1u8; 32]),
        AccountState {
            statement: StatementDigest([3u32; 8]),
            state_root: [1u8; 32],
            start_nexus_hash: [2u8; 32],
            last_proof_height: 0,
            height: 0,
        },
    ));
    let new_rollup_proof = get_mock_proof();
    let new_rollup_pi = get_mock_l1_batch_with_metadata();
    let pubdata_commitments = vec![0u8; 10];
    let versioned_hashes = vec![[0u8; 32]; 5];
    let nexus_hash = NexusH256::zero();

    (
        prev_adapter_proof,
        init_account,
        new_rollup_proof,
        new_rollup_pi,
        pubdata_commitments,
        versioned_hashes,
        nexus_hash,
    )
}

fn main() {
    #[cfg(any(feature = "sp1"))]
    env_logger::Builder::from_env("RUST_LOG")
        .filter_level(log::LevelFilter::Info)
        .init();

    let (
        prev_adapter_proof,
        init_account,
        new_rollup_proof,
        new_rollup_pi,
        pubdata_commitments,
        versioned_hashes,
        nexus_hash,
    ) = create_mock_data();

    #[cfg(feature = "sp1")]
    let ZKSYNC_ADAPTER_ELF: &[u8] =
        include_bytes!("../../methods/sp1-guest/elf/riscv32im-succinct-zkvm-elf");
    #[cfg(feature = "sp1")]
    let ZKSYNC_ADAPTER_ID = [0u32; 8];

    let img_id = ZKSYNC_ADAPTER_ID;
    let elf = ZKSYNC_ADAPTER_ELF.to_vec(); // Mock ELF data
    let prover_modes = vec![ProverMode::Compressed, ProverMode::Compressed];

    for i in 0..prover_modes.len() {
        let prover_mode = &prover_modes[i.clone()];
        let stf = STF::new(img_id, elf.clone(), prover_mode.clone());

        let start = Instant::now();

        let mut file = File::create("arguments_output.txt").unwrap();

    // Write each argument to the file
    writeln!(file, "prev_proof_with_pi: {:?}", prev_adapter_proof.clone());
    writeln!(file, "account_state: {:?}", init_account.clone());
    writeln!(file, "proof: {:?}", new_rollup_proof.clone());
    writeln!(file, "batch_metadata: {:?}", new_rollup_pi.clone());
    writeln!(file, "pubdata_commitments: {:?}", pubdata_commitments.clone());
    writeln!(file, "versioned_hashes: {:?}", versioned_hashes.clone());
    writeln!(file, "range[0]: {:?}", nexus_hash.clone());

        let recursive_proof = stf
            .create_recursive_proof::<Prover, Proof, ZKVM>(
                prev_adapter_proof.clone(),
                init_account.clone(),
                new_rollup_proof.clone(),
                new_rollup_pi.clone(),
                pubdata_commitments.clone(),
                versioned_hashes.clone(),
                nexus_hash,
            )
            .unwrap();

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
}
