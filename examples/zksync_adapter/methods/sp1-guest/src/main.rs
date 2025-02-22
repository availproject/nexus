#![no_main]
use zksync_core::prover::run;
use nexus_core::zkvm::sp1::SP1ZKVM;
sp1_zkvm::entrypoint!(main);

fn main() {
    println!("cycle-tracker-start: sp1-guest");
    run::<SP1ZKVM>();
    println!("cycle-tracker-end: sp1-guest");
}