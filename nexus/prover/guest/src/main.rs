#![no_main]
// If you want to try std support, also update the guest Cargo.toml file
// std support is experimental

use nexus_core::agg_types::AggregatedTransaction;
use nexus_core::agg_types::InitTransaction;
use nexus_core::simple_zkvm_state_machine::ZKVMStateMachine;
use nexus_core::types::AvailHeader;
use nexus_core::types::HeaderStore;
use nexus_core::types::SimpleStateUpdate;
use nexus_core::types::StateUpdate;
use nexus_core::types::TransactionV2;
use risc0_zkvm::guest::env;
risc0_zkvm::guest::entry!(main);

fn main() {
    let start = env::cycle_count();
    eprintln!("Start cycle {}", start);

    let txs: Vec<InitTransaction> = env::read();
    let aggregated_tx: AggregatedTransaction = env::read();
    let touched_states: SimpleStateUpdate = env::read();

    let zkvm_state_machine = ZKVMStateMachine::new();
    let zkvm_result = zkvm_state_machine
        .execute_batch(&txs, aggregated_tx, touched_states)
        .expect("Should not have panicked.");

    let after_stf = env::cycle_count();
    eprintln!("after STF {}", after_stf);

    env::commit(&zkvm_result);
}
