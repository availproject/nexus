#![no_main]
// If you want to try std support, also update the guest Cargo.toml file
// std support is experimental

use nexus_core::types::AvailHeader;
use nexus_core::types::HeaderStore;
use nexus_core::types::StateUpdate;
use nexus_core::types::TransactionV2;
use nexus_core::zkvm_state_machine::ZKVMStateMachine;
use risc0_zkvm::guest::env;
risc0_zkvm::guest::entry!(main);

fn main() {
    let start = env::cycle_count();
    eprintln!("Start cycle {}", start);

    let avail_header: AvailHeader = env::read();
    let before_headers = env::cycle_count();
    let mut old_headers: HeaderStore = env::read();
    let after_headers = env::cycle_count();
    eprintln!(
        "cycle count to read {} headers: {}",
        old_headers.inner().len(),
        after_headers - before_headers
    );

    let txs: Vec<TransactionV2> = env::read();
    let touched_states: StateUpdate = env::read();

    let zkvm_state_machine = ZKVMStateMachine::new();
    let zkvm_result = zkvm_state_machine
        .execute_batch(&avail_header, &mut old_headers, &txs, touched_states)
        .expect("Should not have panicked.");

    let after_stf = env::cycle_count();
    eprintln!("after STF {}", after_stf);

    env::commit(&zkvm_result);
}
