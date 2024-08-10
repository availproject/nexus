#![no_main]
use nexus_core::prover::run;

#[cfg(any(feature = "risc0"))]
use nexus_core::zkvm::risczero::ZKVM;
risc0_zkvm::guest::entry!(main);

#[cfg(any(feature = "sp1"))]
use nexus_core::zkvm::sp1::SP1ZKVM;
#[cfg(any(feature = "sp1"))]
sp1_zkvm::entrypoint!(main);

fn main() {
    #[cfg(any(feature = "risc0"))]
    run::<ZKVM>();

    #[cfg(any(feature = "sp1"))]
    run::<SP1ZKVM>();
}
