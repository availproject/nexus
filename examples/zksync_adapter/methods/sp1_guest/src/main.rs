#![no_main]
use adapter_sdk::types::AdapterPublicInputs;
use nexus_core::types::H256;
use zksync_core::types::{CommitBatchInfo, L1BatchWithMetadata};
use zksync_core::{MockProof, STF};

sp1_zkvm::entrypoint!(main);

fn main() {
    let previous_adapter_pi: AdapterPublicInputs = sp1_zkvm::io::read();
    let new_rollup_proof: MockProof = sp1_zkvm::io::read();
    let new_rollup_pi: L1BatchWithMetadata = sp1_zkvm::io::read();
    let img_id: [u32; 8] = sp1_zkvm::io::read();
    let new_batch: CommitBatchInfo = sp1_zkvm::io::read();
    let nexus_hash: H256 = sp1_zkvm::io::read();

    if new_rollup_pi.header.number.0 > 1 {
        sp1_zkvm::lib::syscall_verify_sp1_proof(img_id, &to_vec(&previous_adapter_pi).unwrap());
    }

    let result = STF::verify_continuity_and_proof(
        previous_adapter_pi.clone(),
        new_rollup_proof,
        new_rollup_pi,
        new_batch,
        nexus_hash,
    )
    .unwrap();

    sp1_zkvm::io::commit(&result);
    
}
