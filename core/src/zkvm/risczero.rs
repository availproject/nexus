use anyhow::anyhow;
#[cfg(any(feature = "native"))]
use risc0_zkvm::{default_prover, ExecutorEnv, ExecutorEnvBuilder};
use risc0_zkvm::{serde::from_slice, Receipt};

use super::traits::{ZKProof, ZKVMProver};

#[cfg(any(feature = "native"))]
pub struct RiscZeroProver<'a> {
    env_builder: ExecutorEnvBuilder<'a>,
    elf: Vec<u8>,
}

#[cfg(any(feature = "native"))]
impl<'a> ZKVMProver<Proof> for RiscZeroProver<'a> {
    fn new(elf: Vec<u8>) -> Self {
        let env_builder = ExecutorEnv::builder();

        Self { env_builder, elf }
    }

    fn add_input<T: serde::Serialize>(&mut self, input: &T) -> Result<(), anyhow::Error> {
        self.env_builder.write(input).map_err(|e| anyhow!(e))?;

        Ok(())
    }

    fn add_proof_for_recursion(&mut self, proof: Proof) -> Result<(), anyhow::Error> {
        self.env_builder.add_assumption(proof.0);

        Ok(())
    }

    fn prove(&mut self) -> Result<Proof, anyhow::Error> {
        let env: ExecutorEnv = self.env_builder.build().map_err(|e| anyhow!(e))?;

        let prover = default_prover();

        let receipt = prover.prove(env, &self.elf).map_err(|e| anyhow!(e))?;

        Ok(Proof(receipt))
    }
}

#[cfg(any(feature = "native"))]
pub struct Proof(pub Receipt);

#[cfg(any(feature = "native"))]
impl ZKProof for Proof {
    fn public_inputs<V: serde::de::DeserializeOwned>(&self) -> Result<V, anyhow::Error> {
        from_slice(&self.0.journal.bytes).map_err(|e| anyhow!(e))
    }

    fn verify(&self, img_id: [u8; 32]) -> Result<(), anyhow::Error> {
        self.0.verify(img_id).map_err(|e| anyhow!(e))
    }
}
