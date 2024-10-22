use serde::{Deserialize, Serialize};

pub mod traits;

#[cfg(any(feature = "native-risc0", feature = "zkvm-risc0"))]
pub mod risczero;

#[cfg(any(feature = "native-sp1", feature = "zkvm-sp1"))]
pub mod sp1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProverMode {
    Compressed,
    Groth16,
    NoAggregation,
    MockProof,
}
