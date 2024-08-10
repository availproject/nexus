

use anyhow::{Error, Ok};
use jmt::proof;
use sha2::Digest;
use sha2::Sha256;
use super::traits::{ZKVMProof, ZKVMProver};
use serde::{de::DeserializeOwned, Serialize, Deserialize};
use serde_json::to_vec;
use anyhow::anyhow;
#[cfg(any(feature = "sp1"))]
use sp1_sdk::{utils, ProverClient, SP1PublicValues, SP1Stdin, SP1ProofWithPublicValues, SP1VerifyingKey, SP1ProvingKey};
use crate::types::Proof;
use super::traits::ZKVMEnv;
#[cfg(any(feature = "sp1"))]
pub struct Sp1Prover {
    sp1_standard_input: SP1Stdin,
    sp1_client: ProverClient,
    elf: Vec<u8>,    
}

#[cfg(any(feature = "sp1"))]
impl ZKVMProver<Sp1Proof> for Sp1Prover {
    fn new(elf: Vec<u8>) -> Self {
        Self { sp1_standard_input: SP1Stdin::new(), sp1_client: ProverClient::new(), elf }
    }

    fn add_input<T: serde::Serialize>(&mut self, input: &T) -> Result<(), anyhow::Error> {
        self.sp1_standard_input.write(input);
        Ok(())
    }

    fn add_proof_for_recursion(&mut self, proof: SpOneProof) -> Result<(), anyhow::Error> {
        // self.env_builder.add_assumption(proof.0);
        // self.sp1_standard_input.write()
        // sp1_zkvm::lib::verify_proof(vkey, public_values_digest);
        // self.proofs.push(proof);
        unimplemented!();
        Ok(())
    }

    fn prove(&mut self) -> Result<Sp1Proof, anyhow::Error> {
        let (pk, vk) = self.sp1_client.setup(&self.elf);
        let proof = self.sp1_client.prove(&pk, self.sp1_standard_input.clone()).run().expect("proof generation failed");
        Ok(SpOneProof(proof))
    }
}
#[cfg(any(feature = "sp1"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sp1Proof(pub SP1ProofWithPublicValues);

#[cfg(any(feature = "sp1"))]
impl ZKVMProof for Sp1Proof {
    fn public_inputs<V: serde::de::DeserializeOwned>(&self) -> Result<V, anyhow::Error> {
        let json = serde_json::to_string(&self.0.public_values)?;
        let deserialized = serde_json::from_str(&json)?;
        Ok(deserialized)
    }

    fn verify(&self, img_id: [u8; 32]) -> Result<(), anyhow::Error> {
        panic!("Not implemented since sp1 proof doesn't contain verify method similar to Risczero https://docs.rs/risc0-zkvm/1.0.5/risc0_zkvm/struct.Receipt.html#method.verify");
    }

    fn try_from(value: Proof) -> Result<Self, anyhow::Error> {
        let receipt: SP1ProofWithPublicValues = value.try_into()?;
        Ok(Self(receipt))
    }

    fn try_into(&self) -> Result<Proof, Error> {
        let encoded_u8: Vec<u8> = to_vec(&self)
            .map_err(|e| anyhow::anyhow!("Serialization error: {}", e))?;
        Ok(Proof(encoded_u8))
    }
}

#[cfg(any(feature = "sp1"))]
impl TryFrom<Proof> for SP1ProofWithPublicValues {
    type Error = anyhow::Error;

    fn try_from(value: Proof) -> Result<Self, Self::Error> {
        let receipt: SP1ProofWithPublicValues = value.try_into()?;
        Ok(receipt)
    }
}

#[cfg(any(feature = "sp1"))]
impl TryInto<Proof> for SP1ProofWithPublicValues {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<Proof, Self::Error> {
        let encoded_u8: Vec<u8> = to_vec(&self)
            .map_err(|e| anyhow::anyhow!("Serialization error: {}", e))?;
        Ok(Proof(encoded_u8))
    }
}

struct SerializedData(Vec<u8>);

impl AsRef<[u8]> for SerializedData {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

fn serialize_to_data<T: Serialize>(input: &T) -> Result<SerializedData, Error> {
    let serialized = bincode::serialize(input)?;
    Ok(SerializedData(serialized))
}

#[cfg(any(feature = "sp1"))]
pub struct SP1ZKVM();

#[cfg(any(feature = "sp1"))]
impl ZKVMEnv for SP1ZKVM {
    fn read_input<T: DeserializeOwned>() -> Result<T, anyhow::Error> {
        Ok(sp1_zkvm::io::read::<T>())
    }

    fn verify<T: Serialize>(img_id: [u32; 8], public_input: &T) -> Result<(), anyhow::Error> {
        let serialized_data = serialize_to_data(public_input)?;
        let byte_slice: &[u8] = serialized_data.as_ref();
        let public_values_digest = Sha256::digest(byte_slice);
        // unsafe {
        //     sp1_zkvm::lib::syscall_verify_sp1_proof(&img_id, public_values_digest)
        // }
       Ok(())
    }

    fn commit<T: Serialize>(data: &T) {
        let serialized_data = serialize_to_data(data).unwrap();
        let byte_slice: &[u8] = serialized_data.as_ref();
        sp1_zkvm::io::commit_slice(byte_slice);
    }
}
