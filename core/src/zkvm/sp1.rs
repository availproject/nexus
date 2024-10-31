use super::traits::ZKVMEnv;
#[cfg(any(feature = "native-sp1"))]
use super::traits::{ZKVMProof, ZKVMProver};
use super::ProverMode;
use crate::types::Proof;
use anyhow::anyhow;
use anyhow::Error;
use bincode;
use jmt::proof;
use serde::Deserializer;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::{from_slice, to_vec};
use sha2::Digest;
use sha2::Sha256;
#[cfg(any(feature = "native-sp1"))]
use sp1_sdk::{
    utils, ProverClient, SP1Proof, SP1ProofWithPublicValues, SP1Prover, SP1ProvingKey,
    SP1PublicValues, SP1Stdin, SP1VerifyingKey,HashableKey,
};
use std::borrow::Cow;

#[cfg(any(feature = "native-sp1"))]
pub struct Sp1Prover {
    sp1_standard_input: SP1Stdin,
    sp1_client: ProverClient,
    elf: Vec<u8>,
    prover_mode: ProverMode,
}

#[cfg(any(feature = "native-sp1"))]
impl ZKVMProver<Sp1Proof> for Sp1Prover {
    fn new(elf: Vec<u8>, prover_mode: ProverMode) -> Self {
        let mut sp1_standard_input = SP1Stdin::new();
        let sp1_client = ProverClient::new();
        Self {
            sp1_standard_input,
            sp1_client,
            elf,
            prover_mode,
        }
    }

    fn add_input<T: serde::Serialize>(&mut self, input: &T) -> Result<(), anyhow::Error> {
        //! TODO: need to do error handling
        self.sp1_standard_input.write(input);
        Ok(())
    }

    fn add_proof_for_recursion(&mut self, proof: Sp1Proof) -> Result<(), anyhow::Error> {
        let (pk, vk) = self.sp1_client.setup(&self.elf);

        self.sp1_standard_input
            .write::<[u32; 8]>(&vk.clone().hash_u32());

        let real_proof = match proof {
            Sp1Proof::Real(i) => i,
            Sp1Proof::Mock(i) => return Err(Error::msg("Expected a Real Proof , found a mock proof")),
        };

        self.sp1_standard_input
            .write::<Vec<u8>>(&real_proof.public_values.clone().to_vec());

        let SP1Proof::Compressed(p) = real_proof.proof else {
            return Err(Error::msg("Proof Compression failed"))
        };
        self.sp1_standard_input.write_proof(p, vk.vk.clone());
        Ok(())
    }

    fn prove(&mut self) -> Result<Sp1Proof, anyhow::Error> {
        let mut sp1_input = self.sp1_standard_input.clone();

        let proof = match &self.prover_mode {
            ProverMode::MockProof => {
                let (output, stats) = self.sp1_client.execute(&self.elf, sp1_input).run().unwrap();

                Sp1Proof::Mock(output)
            }
            ProverMode::Compressed => {
                let (agg_pk, agg_vk) = self.sp1_client.setup(&self.elf);
                Sp1Proof::Real(
                    self.sp1_client
                        .prove(&agg_pk, sp1_input)
                        .compressed()
                        .run()
                        .expect("proof generation failed"),
                )
            }
            _ => {
                let (pk, vk) = self.sp1_client.setup(&self.elf);
                Sp1Proof::Real(
                    self.sp1_client
                        .prove(&pk, sp1_input)
                        .run()
                        .expect("proof generation failed"),
                )
            }
        };

        Ok(proof)
    }
}
// #[cfg(any(feature = "native-sp1"))]
// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct Sp1Proof(pub SP1ProofWithPublicValues);

#[cfg(any(feature = "native-sp1"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Sp1Proof {
    Mock(SP1PublicValues),
    Real(SP1ProofWithPublicValues),
}

#[cfg(any(feature = "native-sp1"))]
impl ZKVMProof for Sp1Proof {
    fn public_inputs<V: serde::Serialize + serde::de::DeserializeOwned + Clone>(
        &mut self,
    ) -> Result<V, anyhow::Error> {
        let public_value = match &self {
            Sp1Proof::Mock(i) => i.clone().read::<V>(),
            Sp1Proof::Real(i) => i.public_values.clone().read::<V>(),
        };

        Ok(public_value.clone())
    }

    // fn verify(&self, img_id: [u8; 32]) -> Result<(), anyhow::Error> {
    //     unimplemented!("Not implemented since sp1 proof doesn't contain verify method similar to Risczero https://docs.rs/risc0-zkvm/1.0.5/risc0_zkvm/struct.Receipt.html#method.verify");
    // }

    fn verify(&self, img_id: Option<[u8; 32]>, elf: Option<Vec<u8>>) -> Result<(), anyhow::Error> {
        match &self {
            Sp1Proof::Mock(i) => Ok(()),
            Sp1Proof::Real(i) => {
                let elf = match elf {
                    Some(elf) => elf,
                    None => return Err(anyhow!("ELF is required")),
                };
                let sp1_client = ProverClient::new();

                let (_, vk) = sp1_client.setup(&elf);
                sp1_client.verify(&i, &vk)?;
                Ok(())
            }
        }
    }

    fn compress(&mut self) -> Result<(), anyhow::Error> {
        match self {
          Sp1Proof::Real(i) => {
            if let Some(groth16_proof) = i.proof.clone().try_as_groth_16() {
                i.proof = SP1Proof::Groth16(groth16_proof);
            } else {
                return Err(anyhow::anyhow!("Proof is not of type Groth16"));
            }
          },
          Sp1Proof::Mock(i) => {},
        }
        Ok(())
    }
}

#[cfg(any(feature = "native-sp1"))]
impl TryFrom<Proof> for Sp1Proof {
    type Error = anyhow::Error;

    fn try_from(value: Proof) -> Result<Self, Self::Error> {
        let receipt: Sp1Proof = from_slice(&value.0)?;
        Ok(receipt)
    }
}

// #[cfg(any(feature = "native-sp1"))]
// impl TryInto<Proof> for SP1ProofWithPublicValues {
//     type Error = anyhow::Error;

//     fn try_into(self) -> Result<Proof, Self::Error> {
//         let encoded_u8: Vec<u8> =
//             to_vec(&self).map_err(|e| anyhow::anyhow!("Serialization error: {}", e))?;
//         Ok(Proof(encoded_u8))
//     }
// }

#[cfg(any(feature = "native-sp1"))]
impl TryInto<Proof> for Sp1Proof {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<Proof, Self::Error> {
        let encoded_u8: Vec<u8> =
            to_vec(&self).map_err(|e| anyhow::anyhow!("Serialization error: {}", e))?;
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

pub struct SP1ZKVM();

impl ZKVMEnv for SP1ZKVM {
    fn read_input<T: DeserializeOwned>() -> Result<T, anyhow::Error> {
        Ok(sp1_zkvm::io::read::<T>())
    }

    fn verify<T: Serialize>(img_id: [u32; 8], public_input: &T) -> Result<(), anyhow::Error> {
        let serialized_data = serialize_to_data(public_input)?;
        let byte_slice: &[u8] = serialized_data.as_ref();
        let public_values_digest = Sha256::digest(byte_slice);
        let mut digest_array = [0u8; 32];
        digest_array.copy_from_slice(&public_values_digest);
        sp1_zkvm::lib::verify::verify_sp1_proof(&img_id, &digest_array);
        
        Ok(())
    }

    fn commit<T: Serialize>(data: &T) {
        let serialized_data = serialize_to_data(data).unwrap();
        let byte_slice: &[u8] = serialized_data.as_ref();
        sp1_zkvm::io::commit_slice(byte_slice);
    }
}

#[cfg(any(feature = "native-sp1"))]
pub trait ProofConversion: std::convert::From<Sp1Proof> {}

#[cfg(any(feature = "native-sp1"))]
impl ProofConversion for Sp1Proof {}
