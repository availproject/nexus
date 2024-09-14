#![no_main]
use zksync_methods::zksync_prover::run;
use nexus_core::zkvm::sp1::SP1ZKVM;
sp1_zkvm::entrypoint!(main);

fn main() {
    run::<SP1ZKVM>();
}
