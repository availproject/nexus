use nexus_core::zkvm::risczero::ZKVM;
use risc0_zkvm::guest::env;
use ethereum_adapter_core::prover::run;

fn main() {
    let before_cycle = env::cycle_count();
    run::<ZKVM>();
    let after_cycle = env::cycle_count();
    println!("Proving took {} cycles", after_cycle - before_cycle);
}