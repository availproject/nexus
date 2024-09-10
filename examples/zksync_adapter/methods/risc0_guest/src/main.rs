#![no_main]
use adapter_sdk::types::AdapterPublicInputs;
use nexus_core::types::H256;
use risc0_zkvm::guest::env;
use risc0_zkvm::serde::to_vec;
use zksync_core::types::{CommitBatchInfo, L1BatchWithMetadata};
use zksync_core::{MockProof, STF};

risc0_zkvm::guest::entry!(main);

fn main() {
    let previous_adapter_pi: AdapterPublicInputs = env::read();
    let new_rollup_proof: MockProof = env::read();
    let new_rollup_pi: L1BatchWithMetadata = env::read();
    let img_id: [u32; 8] = env::read();
    let new_batch: CommitBatchInfo = env::read();
    let nexus_hash: H256 = env::read();
    if new_rollup_pi.header.number.0 > 1 {
        env::verify(img_id, &to_vec(&previous_adapter_pi).unwrap()).unwrap();
    }

    let result = STF::verify_continuity_and_proof(
        previous_adapter_pi.clone(),
        new_rollup_proof,
        new_rollup_pi,
        new_batch,
        nexus_hash,
    )
    .unwrap();

    env::commit(&result);
}
