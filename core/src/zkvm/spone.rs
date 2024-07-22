#[cfg(any(feature = "spone"))]
use sp1_sdk::{utils, ProverClient, SP1PublicValues, SP1Stdin, SP1ProofWithPublicValues};

use super::traits::ZKProof;

#[cfg(any(feature = "spone"))]
pub struct SpOneProver {
    sp1_standard_input: SP1Stdin,
    sp1_client: ProverClient,
    elf: Vec<u8>,    
}

#[cfg(any(feature = "spone"))]
impl ZKVMProver<SpOneProof> for SpOneProver {
    fn new(elf: Vec<u8>) -> Self {
        let stdin = SP1Stdin::new();
        let client = ProverClient::new();
        Self { stdin, client,  elf }
    }

    fn add_input(&mut self, input: &T) -> Result<(), anyhow::Error> {
        self.stdin.write(input).map_err(|e| anyhow!(e))?;
        Ok(())
    
    }

    fn add_proof_for_recursion(&mut self, proof: SpOneProof) -> Result<(), anyhow::Error> {
        // self.env_builder.add_assumption(proof.0);

        Ok(())
    }

    fn prove(&mut self) -> Result<SpOneProof, anyhow::Error> {
        let (pk, vk) = self.sp1_client.setup(&self.elf);
        let mut proof = self.sp1_client.prove(&pk, stdin).expect("proving failed");

        Ok(SpOneProof(proof, self.sp1_client))
    }
}


#[cfg(any(feature = "spone"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpOneProof(pub SP1ProofWithPublicValues, pub ProverClient);

#[cfg(any(feature = "spone"))]
impl ZKProof for SpOneProof {
    fn public_inputs<V: serde::de::DeserializeOwned>(&self) -> Result<V, anyhow::Error> {
        
    }

    fn verify(&self, img_id: [u8; 32]) -> Result<(), anyhow::Error> {

        self.1.verify(self.0, vk).expect("verification failed");
        // let client = ProverClient::new();
        // let public_values = SP1PublicValues::new();
        // let proof = self.0.clone();
        // let result = client.verify_proof(proof, public_values, img_id);
        // if result {
        //     Ok(())
        // } else {
        //     Err(anyhow!("Proof verification failed"))
        // }

    }
    
}
