use adapter_sdk::types::AdapterPublicInputs;
use nexus_core::types::H256;
use zksync_core::types::{CommitBatchInfo, L1BatchWithMetadata};
use zksync_core::{MockProof, STF};
use nexus_core::zkvm::traits::ZKVMEnv;

pub fn run<Z: ZKVMEnv>() {
     
    let previous_adapter_pi: AdapterPublicInputs = Z::read_input::<AdapterPublicInputs>().unwrap();
    let new_rollup_proof: MockProof = Z::read_input::<MockProof>().unwrap();
    let new_rollup_pi: L1BatchWithMetadata = Z::read_input::<L1BatchWithMetadata>().unwrap();
    let img_id: [u32; 8] = Z::read_input::<[u32; 8]>().unwrap();
    let new_batch: CommitBatchInfo = Z::read_input::<CommitBatchInfo>().unwrap();
    let nexus_hash: H256 = Z::read_input::<H256>().unwrap();
    if new_rollup_pi.header.number.0 > 1 {
        Z::verify(img_id, &to_vec(&previous_adapter_pi).unwrap()).unwrap();
    }

    let result = STF::verify_continuity_and_proof(
        previous_adapter_pi.clone(),
        new_rollup_proof,
        new_rollup_pi,
        new_batch,
        nexus_hash,
    );

    Z::commit(&result);
    
}