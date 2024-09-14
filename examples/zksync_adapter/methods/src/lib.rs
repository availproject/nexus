pub mod zksync_prover;

// #[cfg(any(feature = "risc0"))]
include!(concat!(env!("OUT_DIR"), "/methods.rs"));
