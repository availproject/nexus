use serde::{de::DeserializeOwned, Deserialize, Serialize};

pub trait ZKVMProver<R: ZKProof> {
    fn new(elf: Vec<u8>) -> Self;
    fn add_input<T: Serialize>(&mut self, input: &T) -> Result<(), anyhow::Error>;
    fn add_proof_for_recursion(&mut self, proof: R) -> Result<(), anyhow::Error>;
    fn prove(&mut self) -> Result<R, anyhow::Error>;
}

pub trait ZKProof {
    fn verify(&self, img_id: [u8; 32]) -> Result<(), anyhow::Error>;
    fn public_inputs<V: DeserializeOwned>(&self) -> Result<V, anyhow::Error>;
}
