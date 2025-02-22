use crate::types::Proof as NexusProof;
use serde::{de::DeserializeOwned, Serialize};

use super::ProverMode;

#[cfg(any(feature = "native"))]
pub trait ZKVMProver<R: ZKVMProof> {
    fn new(elf: Vec<u8>, prover_mode: ProverMode) -> Self;
    fn add_input<T: Serialize>(&mut self, input: &T) -> Result<(), anyhow::Error>;
    fn add_proof_for_recursion(&mut self, proof: R) -> Result<(), anyhow::Error>;
    fn prove(&mut self) -> Result<R, anyhow::Error>;
}

#[cfg(any(feature = "native"))]
pub trait ZKVMProof: Sized {
    fn verify(
        &self,
        img_id: Option<[u8; 32]>,
        elf: Option<Vec<u8>>,
        proof_mode: ProverMode,
    ) -> Result<(), anyhow::Error>;
    fn public_inputs<V: Serialize + DeserializeOwned + Clone>(
        &mut self,
    ) -> Result<V, anyhow::Error>;
    fn compress(&mut self) -> Result<Self, anyhow::Error>;
}

// pub trait ZKProof {
//     fn verify(&self, img_id: [u8; 32]) -> Result<(), anyhow::Error>;
//     fn public_inputs<V: DeserializeOwned>(&self) -> Result<V, anyhow::Error>;
// }

pub trait ZKVMEnv {
    fn verify<T: Serialize>(img_id: [u32; 8], public_inputs: &T) -> Result<(), anyhow::Error>;
    fn read_input<T: DeserializeOwned>() -> Result<T, anyhow::Error>;
    fn commit<T: Serialize>(data: &T);
}
