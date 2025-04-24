#![no_main]
use nexus_core::nexus_guest::run;
use nexus_core::zkvm::risczero::ZKVM;
risc0_zkvm::guest::entry!(main);

fn main() {
    run::<ZKVM>();
}
