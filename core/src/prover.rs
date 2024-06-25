use crate::types::AvailHeader;
use crate::types::HeaderStore;
use crate::types::StateUpdate;
use crate::types::TransactionZKVM;
use crate::zkvm::traits::ZKVMEnv;
use crate::zkvm_state_machine::ZKVMStateMachine;

pub fn run<Z: ZKVMEnv>() {
    // let start = env::cycle_count();
    // eprintln!("Start cycle {}", start);

    let txs: Vec<TransactionZKVM> = Z::read_input().unwrap();
    let touched_states: StateUpdate = Z::read_input().unwrap();
    let header: AvailHeader = Z::read_input().unwrap();
    let mut header_store: HeaderStore = Z::read_input().unwrap();

    let zkvm_state_machine = ZKVMStateMachine::<Z>::new();
    let zkvm_result = zkvm_state_machine
        .execute_batch(&header, &mut header_store, &txs, touched_states)
        .expect("Should not have panicked.");

    // let after_stf = env::cycle_count();
    // eprintln!("after STF {}", after_stf);

    Z::commit(&zkvm_result);
}
