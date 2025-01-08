use crate::types::AvailHeader;
use crate::types::HeaderStore;
use crate::types::StateUpdate;
use crate::types::TransactionZKVM;
use crate::zkvm::traits::ZKVMEnv;
use crate::zkvm_state_machine::ZKVMStateMachine;

pub fn run<Z: ZKVMEnv>() {
    // let start = env::cycle_count();
    // eprintln!("Start cycle {}", start);

    let txs: Vec<TransactionZKVM> = Z::read_input::<Vec<TransactionZKVM>>().unwrap();
    let touched_states: StateUpdate = Z::read_input::<StateUpdate>().unwrap();
    let header: AvailHeader = Z::read_input::<AvailHeader>().unwrap();
    let mut header_store: HeaderStore = Z::read_input::<HeaderStore>().unwrap();
    let img_id: [u32; 8] = Z::read_input::<[u32; 8]>().unwrap();

    let zkvm_state_machine = ZKVMStateMachine::<Z>::new();
    let zkvm_result = zkvm_state_machine
        .execute_batch(&header, &mut header_store, &txs, touched_states)
        .expect("Should not have panicked.");

        if header_store.inner.clone().len() != 0 {
            Z::verify(img_id, &header_store.inner.first().unwrap()).unwrap();
            // = 0 only for the case of genesis
            if header_store.inner.first().unwrap().avail_header_hash != header.parent_hash {
                panic!("match not found");
            }
        }

    // let after_stf = env::cycle_count();
    // eprintln!("after STF {}", after_stf);

    Z::commit(&zkvm_result);
}
