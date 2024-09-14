#![no_main]
use zksync-methods::zksync_prover::run;
sp1_zkvm::entrypoint!(main);

fn main() {
    // let previous_adapter_pi: AdapterPublicInputs = sp1_zkvm::io::read();
    // let new_rollup_proof: MockProof = sp1_zkvm::io::read();
    // let new_rollup_pi: L1BatchWithMetadata = sp1_zkvm::io::read();
    // let img_id: [u32; 8] = sp1_zkvm::io::read();
    // let new_batch: CommitBatchInfo = sp1_zkvm::io::read();
    // let nexus_hash: H256 = sp1_zkvm::io::read();

    // if new_rollup_pi.header.number.0 > 1 {
    //     let serialized = serialize(&previous_adapter_pi).unwrap();
        
    //     let mut previous_adapter_pi_bytes = [0u8; 32];
    //     previous_adapter_pi_bytes.copy_from_slice(&serialized[..32]);
    //     unsafe {
    //         sp1_zkvm::lib::syscall_verify_sp1_proof(&img_id, &previous_adapter_pi_bytes);
    //     }
    // }

    // let result = STF::verify_continuity_and_proof(
    //     previous_adapter_pi.clone(),
    //     new_rollup_proof,
    //     new_rollup_pi,
    //     new_batch,
    //     nexus_hash,
    // )
    // .unwrap();

    // sp1_zkvm::io::commit(&result);
    
}
