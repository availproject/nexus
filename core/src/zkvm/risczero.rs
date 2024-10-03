use crate::types::Proof;

use super::traits::ZKVMEnv;
#[cfg(any(feature = "native-risc0"))]
use super::traits::{ZKVMProof, ZKVMProver};
use anyhow::anyhow;
use anyhow::Error;
use risc0_zkvm::guest::env;
use risc0_zkvm::serde::to_vec;
#[cfg(any(feature = "native-risc0"))]
use risc0_zkvm::{default_prover, Executor, ExecutorEnv, ExecutorEnvBuilder, Prover};
use risc0_zkvm::{serde::from_slice, Receipt};
use serde::{Deserialize, Serialize};

#[cfg(any(feature = "native-risc0"))]
pub struct RiscZeroProver<'a> {
    env_builder: ExecutorEnvBuilder<'a>,
    elf: Vec<u8>,
}

#[cfg(any(feature = "native-risc0"))]
impl<'a> ZKVMProver<RiscZeroProof> for RiscZeroProver<'a> {
    fn new(elf: Vec<u8>) -> Self {
        let env_builder = ExecutorEnv::builder();
        Self { env_builder, elf }
    }

    fn add_input<T: serde::Serialize>(&mut self, input: &T) -> Result<(), anyhow::Error> {
        self.env_builder.write(input).map_err(|e| anyhow!(e))?;
        Ok(())
    }

    fn add_proof_for_recursion(&mut self, proof: RiscZeroProof) -> Result<(), anyhow::Error> {
        self.env_builder.add_assumption(proof.0);
        Ok(())
    }

    fn prove(&mut self) -> Result<RiscZeroProof, anyhow::Error> {
        let start_time = std::time::Instant::now(); // Start time measurement

        //let env_1: ExecutorEnv = self.env_builder.clone().build().map_err(|e| anyhow!(e))?;
        let env: ExecutorEnv = self.env_builder.build().map_err(|e| anyhow!(e))?;

        let prover = default_prover();
        // let stats = prover.execute(env, &self.elf).map_err(|e| anyhow!(e))?;
        // println!("Execution stats: {:?}", stats);
        let receipt = prover
            .prove(env, &self.elf)
            .map_err(|e| anyhow!("Error when proving: {:?}", e))?;

        let duration = start_time.elapsed(); // Calculate elapsed time
        println!("Prover stats: {:?}", receipt.stats);
        println!("Proof generation completed in: {:?}", duration); // Log the elapsed time

        Ok(RiscZeroProof(receipt.receipt))
    }
}

#[cfg(any(feature = "native-risc0"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiscZeroProof(pub Receipt);

#[cfg(any(feature = "native-risc0"))]
impl ZKVMProof for RiscZeroProof {
    fn public_inputs<V: serde::Serialize + serde::de::DeserializeOwned + Clone>(&self) -> Result<V, anyhow::Error> {
        from_slice(&self.0.journal.bytes).map_err(|e| anyhow!(e))
    }

    fn verify(&self, img_id: [u8; 32]) -> Result<(), anyhow::Error> {
        self.0.verify(img_id).map_err(|e| anyhow!(e))
    }
}

#[cfg(any(feature = "native-risc0"))]
impl TryInto<Proof> for RiscZeroProof {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<Proof, Self::Error> {
        let encoded_u32: Vec<u32> =
            to_vec(&self).map_err(|e| anyhow::anyhow!("Serialization error: {}", e))?;

        // Convert Vec<u32> to Vec<u8>
        let encoded_u8: Vec<u8> = encoded_u32
            .into_iter()
            .flat_map(|x| x.to_ne_bytes().to_vec())
            .collect();
        Ok(Proof(encoded_u8))
    }
}

#[cfg(any(feature = "native-risc0"))]
impl TryFrom<Proof> for RiscZeroProof {
    type Error = anyhow::Error;

    fn try_from(value: Proof) -> Result<Self, Self::Error> {
        let receipt: Receipt = from_slice(&value.0)?;

        Ok(Self(receipt))
    }
}

pub struct ZKVM();

impl ZKVMEnv for ZKVM {
    fn read_input<T: serde::de::DeserializeOwned>() -> Result<T, anyhow::Error> {
        Ok(env::read())
    }

    fn verify<T: serde::Serialize>(
        img_id: [u32; 8],
        public_inputs: &T,
    ) -> Result<(), anyhow::Error> {
        let public_input_vec = match to_vec(public_inputs) {
            Ok(i) => i,
            Err(_) => return Err(anyhow::anyhow!("Could not encode public inputs")),
        };

        env::verify(img_id, &public_input_vec).map_err(|e| anyhow::anyhow!(e))
    }

    fn commit<T: serde::Serialize>(data: &T) {
        env::commit(data);
    }
}
