use crate::types::Proof as NexusProof;
use serde::{de::DeserializeOwned, Serialize};

#[cfg(any(feature = "native"))]
pub trait ZKVMProver<R: ZKVMProof> {
    fn new(elf: Vec<u8>) -> Self;
    fn add_input<T: Serialize>(&mut self, input: &T) -> Result<(), anyhow::Error>;
    fn add_proof_for_recursion(&mut self, proof: R) -> Result<(), anyhow::Error>;
    fn prove(&mut self) -> Result<R, anyhow::Error>;
}

#[cfg(any(feature = "native"))]
pub trait ZKVMProof: Sized {
    fn verify(&self, img_id: [u8; 32]) -> Result<(), anyhow::Error>;
    fn public_inputs<V: DeserializeOwned>(&self) -> Result<V, anyhow::Error>;
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
