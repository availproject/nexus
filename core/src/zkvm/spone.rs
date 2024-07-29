

use anyhow::{Error, Ok};
use jmt::proof;
use sha2::Digest;
use sha2::Sha256;
use super::traits::{ZKProof, ZKVMProver};
use serde::{de::DeserializeOwned, Serialize, Deserialize};
use anyhow::anyhow;
#[cfg(any(feature = "spone"))]
use sp1_sdk::{utils, ProverClient, SP1PublicValues, SP1Stdin, SP1ProofWithPublicValues, SP1VerifyingKey, SP1ProvingKey};

use crate::types::Proof;
use super::traits::{ZKProof, ZKVMEnv};
#[cfg(any(feature = "spone"))]
pub struct SpOneProver {
    sp1_standard_input: SP1Stdin,
    sp1_client: ProverClient,
    proving_key: SP1ProvingKey,
    verification_key: SP1VerifyingKey,
    proofs: Vec<SpOneProof>,
    elf: Vec<u8>,    
}

#[cfg(any(feature = "spone"))]
impl ZKVMProver<SpOneProof> for SpOneProver {
    fn new(elf: Vec<u8>) -> Self {
        let stdin = SP1Stdin::new();
        let client = ProverClient::new();
        let (pk, vk) = client.setup(&elf);
        Self { stdin, client, pk, vk, elf }
    }

    fn add_input<T: serde::Serialize>(&mut self, input: &T) -> Result<(), anyhow::Error> {
        self.sp1_standard_input.write(input).map_err(|e| anyhow!(e))?;
        Ok(())
    }

    fn add_proof_for_recursion(&mut self, proof: SpOneProof) -> Result<(), anyhow::Error> {
        // self.env_builder.add_assumption(proof.0);
        // self.sp1_standard_input.write()
        // sp1_zkvm::lib::verify_proof(vkey, public_values_digest);
        self.proofs.push(proof);
        Ok(())
    }

    fn prove(&mut self) -> Result<SpOneProof, anyhow::Error> {

        //create an array of self.vk hashes
        let vkeys;
        for i in 0..self.proofs.len() {
            vkeys.push(self.vk.hash_u32());
        }
        self.stdin.write::<Vec<[u32; 8]>>(&vkeys);
        // let vkeys = self.proofs.iter().map(|self.vk| self.vk.hash_u32()).collect::<Vec<_>>();
        // let vkeys = inputs
        //     .iter()
        //     .map(|input| input.vk.hash_u32())
        //     .collect::<Vec<_>>();
        // stdin.write::<Vec<[u32; 8]>>(&vkeys);

        let public_values = self.proofs
                            .iter()
                            .map(|proof| proof.public_values)
                            .collect::<Vec<_>>();
        stdin.write::<Vec<Vec<u8>>>(&public_values);
        // // Write the public values.
        // let public_values = inputs
        //     .iter()
        //     .map(|input| input.proof.public_values.to_vec())
        //     .collect::<Vec<_>>();
        // stdin.write::<Vec<Vec<u8>>>(&public_values);

        for proof in self.proofs {
            let SP1Proof::Compressed(proof) = proof.proof;
            stdin.write_proof(proof.proof, proof.vk);
        }
        // // Write the proofs.
        // //
        // // Note: this data will not actually be read by the aggregation program, instead it will be
        // // witnessed by the prover during the recursive aggregation process inside SP1 itself.
        self.sp1_client
            .prove(&self, stdin)
            .plonk()
            .run()
            .expect("proving failed");
        // for input in inputs {
        //     let SP1Proof::Compressed(proof) = input.proof.proof else {
        //         panic!()
        //     };
        //     stdin.write_proof(proof, input.vk.vk);
        // }
        // let mut proof = self.sp1_client.prove(&self.proving_key, self.stdin).expect("proving failed");

        Ok(SpOneProof(proof, self.sp1_client))
    }
}
#[cfg(any(feature = "spone"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpOneProof(pub SP1ProofWithPublicValues, pub ProverClient, pub SP1ProvingKey);

#[cfg(any(feature = "spone"))]
impl ZKProof for SpOneProof {
    fn public_inputs<V: serde::de::DeserializeOwned>(&self) -> Result<V, anyhow::Error> {
        
    }

    fn verify(&self, img_id: [u8; 32]) -> Result<(), anyhow::Error> {

        // self.1.verify(self.0, self.2).expect("verification failed");
        
        // let client = ProverClient::new();
        // let public_values = SP1PublicValues::new();
        // let proof = self.0.clone();
        // let result = client.verify_proof(proof, public_values, img_id);
        // if result {
        //     Ok(())
        // } else {
        //     Err(anyhow!("Proof verification failed"))
        // }

        Ok(())

    }

    fn try_from(value: Proof) -> Result<Self, anyhow::Error> {
        let receipt: SP1ProofWithPublicValues = value.try_into()?;
        Ok(Self(receipt))
    }

    fn try_into(&self) -> Result<Proof, Error> {
        // let encoded
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

#[cfg(any(feature = "spone"))]
pub struct SZKVM();

#[cfg(any(feature = "spone"))]
impl ZKVMEnv for SZKVM{
    fn read_input<T: DeserializeOwned>() -> Result<T, anyhow::Error> {
        Ok(sp1_zkvm::io::read())
    }

    fn verify<T: Serialize>(img_id: [u32; 8], public_input: &T) -> Result<(), anyhow::Error> {
        let serialized_data = serialize_to_data(public_input)?;
        let byte_slice: &[u8] = serialized_data.as_ref();
        let public_values_digest = Sha256::digest(byte_slice);
        unsafe {
            sp1_zkvm::lib::syscall_verify_sp1_proof(&img_id, public_values_digest)
        }
       Ok(())
    }

    fn commit<T: Serialize>(data: &T) {
        let serialized_data = serialize_to_data(data).unwrap();
        let byte_slice: &[u8] = serialized_data.as_ref();
        sp1_zkvm::io::commit_slice(byte_slice);
    }
}