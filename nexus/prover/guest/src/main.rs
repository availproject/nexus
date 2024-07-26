#![no_main]
use nexus_core::prover::run;
use nexus_core::zkvm::risczero::ZKVM;
risc0_zkvm::guest::entry!(main);

#[cfg(any(feature = "spone"))]
use nexus_core::zkvm::spone::SZKVM;
#[cfg(any(feature = "spone"))]
sp1_zkvm::entrypoint!(main);

fn main() {
    #[cfg(any(feature = "native"))]
    run::<ZKVM>();

    #[cfg(any(feature = "spone"))]
    run::<SZKVM>();
}
