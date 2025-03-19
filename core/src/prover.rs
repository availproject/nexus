use crate::types::AvailHeader;
use crate::types::Blob;
use crate::types::BlobProof;
use crate::types::HeaderStore;
use crate::types::StateUpdate;
use crate::zkvm::traits::ZKVMEnv;
use crate::zkvm_state_machine::ZKVMStateMachine;

pub fn run<Z: ZKVMEnv>() {
    // let start = env::cycle_count();
    // eprintln!("Start cycle {}", start);

    println!("Starting to read blobs input");
    let blobs: Vec<Blob> = Z::read_input::<Vec<Blob>>().unwrap();
    println!("Finished reading blobs input");

    println!("Starting to read blob proofs input");
    let blob_proofs: Vec<BlobProof> = Z::read_input::<Vec<BlobProof>>().unwrap();
    println!("Finished reading blob proofs input");

    println!("Starting to read state update input");
    let touched_states: StateUpdate = Z::read_input::<StateUpdate>().unwrap();
    println!("Finished reading state update input");

    println!("Starting to read header input");
    let header: AvailHeader = Z::read_input::<AvailHeader>().unwrap();
    println!("Finished reading header input");

    //TODO: Should be part of elf, instead of being passed as input
    println!("Starting to read app id input");
    let app_id: u32 = Z::read_input::<u32>().unwrap();
    println!("Finished reading app id input");

    println!("Starting to read header store input");
    let mut header_store: HeaderStore = Z::read_input::<HeaderStore>().unwrap();
    println!("Finished reading header store input");

    let zkvm_state_machine = ZKVMStateMachine::<Z>::new();
    let zkvm_result = zkvm_state_machine
        .execute_batch(
            &header,
            &mut header_store,
            &blobs,
            &blob_proofs,
            touched_states,
            app_id,
        )
        .expect("Should not have panicked.");

    // let after_stf = env::cycle_count();
    // eprintln!("after STF {}", after_stf);

    Z::commit(&zkvm_result);
}
