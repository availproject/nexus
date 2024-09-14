#![no_main]
use zksync-methods::zksync_prover::run;
sp1_zkvm::entrypoint!(main);

fn main() {
    run::<ZKVM>();
    
}
