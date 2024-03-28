#![no_main]
// If you want to try std support, also update the guest Cargo.toml file
// std support is experimental

use nexus_core::agg_types::AggregatedTransaction;
use nexus_core::agg_types::InitTransaction;
use nexus_core::types::AvailHeader;
use nexus_core::types::HeaderStore;
use nexus_core::types::StateUpdate;
use nexus_core::types::TransactionZKVM;
use nexus_core::zkvm_state_machine::ZKVMStateMachine;
use risc0_zkvm::guest::env;
risc0_zkvm::guest::entry!(main);

fn main() {
    let start = env::cycle_count();
    eprintln!("Start cycle {}", start);

    let txs: Vec<TransactionZKVM> = env::read();
    let touched_states: StateUpdate = env::read();
    let header: AvailHeader = env::read();
    let mut header_store: HeaderStore = env::read();

    let zkvm_state_machine = ZKVMStateMachine::new();
    let zkvm_result = zkvm_state_machine
        .execute_batch(&header, &mut header_store, &txs, touched_states)
        .expect("Should not have panicked.");

    let after_stf = env::cycle_count();
    eprintln!("after STF {}", after_stf);

    env::commit(&zkvm_result);
}
