#![no_main]
use nexus_core::prover::run;
use nexus_core::zkvm::sp1::SP1ZKVM;
sp1_zkvm::entrypoint!(main);

fn main() {
    println!("cycle-tracker-start: zksync-adapter-sp1");
    run::<SP1ZKVM>();
    println!("cycle-tracker-end: zksync-adapter-sp1");
}
