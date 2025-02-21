use anyhow::anyhow;
use num_bigint::BigUint;
use primitive_types::U256;
use reqwest::Client;
use zksync_core::{
    L1BatchWithMetadata, ProofWithCommitmentAndL1BatchMetaData, ProofWithL1BatchMetaData, Token,
};

pub struct ProofAPI {
    url: String,
    client: Client,
}

pub enum ProofAPIResponse {
    Pruned,
    Pending,
    Found((ProofWithCommitmentAndL1BatchMetaData, Vec<String>)),
}

fn u256_to_string(uint: &U256) -> String {
    let mut bytes = [0u8; 32];
    uint.to_big_endian(&mut bytes);
    BigUint::from_bytes_be(&bytes).to_string()
}

fn serialized_proof_bigint_strings_array(token: &Token) -> Vec<String> {
    fn process_token(token: &Token) -> Vec<String> {
        match token {
            Token::Array(arr) => arr.iter().flat_map(process_token).collect(),
            Token::FixedArray(arr) => arr.iter().flat_map(process_token).collect(),
            Token::Tuple(tuple) => tuple.iter().flat_map(process_token).collect(),
            Token::Uint(uint) => vec![u256_to_string(uint)],
            _ => vec![], // Ignore other token types
        }
    }

    process_token(token)
}

impl ProofAPI {
    pub async fn get_proof_for_l1_batch(
        &self,
        l1_batch_number: u32,
    ) -> Result<ProofAPIResponse, anyhow::Error> {
        // Construct the API URL
        let request_url = format!("{}/metadata?l1BatchNumber={}", self.url, l1_batch_number);

        // Send the GET request
        let response = self.client.get(&request_url).send().await?;

        // Check if the status is 200 OK
        if response.status().is_success() {
            // Parse the JSON response into L1BatchWithMetadata
            let proof_with_commitment_and_l1_batch_meta_data: ProofWithCommitmentAndL1BatchMetaData = response.json().await?;

            let tokens = proof_with_commitment_and_l1_batch_meta_data
                .clone()
                .proof_with_l1_batch_metadata
                .bytes;
            let proof = serialized_proof_bigint_strings_array(&tokens);
            let pubdata_commitments = proof_with_commitment_and_l1_batch_meta_data
                .clone()
                .pubdata_commitments;

            // Assuming you have a way to get MockProof; otherwise, return an appropriate variant
            Ok(ProofAPIResponse::Found((
                proof_with_commitment_and_l1_batch_meta_data,
                proof,
            )))
        } else {
            // Handle different status codes as needed
            match response.status().as_u16() {
                404 => Ok(ProofAPIResponse::Pending),
                _ => Err(anyhow!("Unexpected response status: {}", response.status())),
            }
        }
    }

    pub fn new(url: &str) -> Self {
        Self {
            url: String::from(url),
            client: Client::new(),
        }
    }
}
