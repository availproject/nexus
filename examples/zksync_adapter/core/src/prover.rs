use crate::types::{CommitBatchInfo, L1BatchWithMetadata};
use crate::STF;
use adapter_sdk::types::AdapterPublicInputs;
use nexus_core::types::H256;
use nexus_core::zkvm::traits::ZKVMEnv;
use nexus_core::zkvm::ProverMode;

pub fn run<Z: ZKVMEnv>() {
    let previous_adapter_pi: AdapterPublicInputs = Z::read_input::<AdapterPublicInputs>().unwrap();
    let new_rollup_proof: Vec<String> = Z::read_input::<Vec<String>>().unwrap();
    let new_rollup_pi: L1BatchWithMetadata = Z::read_input::<L1BatchWithMetadata>().unwrap();
    let img_id: [u32; 8] = Z::read_input::<[u32; 8]>().unwrap();
    let new_batch: CommitBatchInfo = Z::read_input::<CommitBatchInfo>().unwrap();
    let pubdata_commitments: Vec<u8> = Z::read_input::<Vec<u8>>().unwrap();
    let versioned_hashes: Vec<[u8; 32]> = Z::read_input::<Vec<[u8; 32]>>().unwrap();
    let nexus_hash: H256 = Z::read_input::<H256>().unwrap();
    let prover_mode: ProverMode = Z::read_input::<ProverMode>().unwrap();

    if new_rollup_pi.header.number.0 > 1 {
        let vec = (&previous_adapter_pi).rollup_hash.unwrap().to_keyed_vec(&[]);
        Z::verify(img_id, &vec).unwrap();
    }

    let result = STF::verify_continuity_and_proof(
        previous_adapter_pi.clone(),
        new_rollup_proof,
        new_rollup_pi,
        new_batch,
        pubdata_commitments,
        versioned_hashes,
        nexus_hash,
        prover_mode,
    )
    .expect("Should not have panicked.");

    Z::commit(&result);
}
