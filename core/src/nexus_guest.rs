use crate::types::AvailHeader;
use crate::types::Blob;
use crate::types::BlobProof;
use crate::types::HeaderStore;
use crate::types::NexusZKVMInputs;
use crate::types::StateUpdate;
use crate::zkvm::traits::ZKVMEnv;
use crate::zkvm_state_machine::ZKVMStateMachine;

pub fn run<Z: ZKVMEnv>() {
    // let start = env::cycle_count();
    // eprintln!("Start cycle {}", start);

    println!("Starting to read zkvm inputs");
    let NexusZKVMInputs {
        blobs,
        blob_proofs,
        state_update,
        header,
        mut header_store,
        app_id,
    } = Z::read_input::<NexusZKVMInputs>().unwrap();
    println!("Finished reading zkvm inputs");

    let zkvm_state_machine = ZKVMStateMachine::<Z>::new();
    let zkvm_result = zkvm_state_machine
        .execute_batch(
            &header,
            &mut header_store,
            &blobs,
            &blob_proofs,
            state_update,
            app_id,
        )
        .expect("Should not have panicked.");

    // let after_stf = env::cycle_count();
    // eprintln!("after STF {}", after_stf);

    println!("Committing zkvm result");
    Z::commit(&zkvm_result);
}
