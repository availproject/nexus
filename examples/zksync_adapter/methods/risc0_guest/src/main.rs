#![no_main]
use zksync_methods::zksync_prover::run;
use nexus_core::zkvm::risczero::ZKVM;
risc0_zkvm::guest::entry!(main);

fn main() {
    run::<ZKVM>();
}
