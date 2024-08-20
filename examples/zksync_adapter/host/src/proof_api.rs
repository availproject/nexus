use anyhow::anyhow;
use reqwest::Client;
use zksync_core::{L1BatchWithMetadata, MockProof};

pub struct ProofAPI {
    url: String,
    client: Client,
}

pub enum ProofAPIResponse {
    Pruned,
    Pending,
    Found((L1BatchWithMetadata, MockProof)),
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
            let metadata: L1BatchWithMetadata = response.json().await?;

            // Assuming you have a way to get MockProof; otherwise, return an appropriate variant
            let proof = MockProof(()); // Or fetch it from somewhere

            Ok(ProofAPIResponse::Found((metadata, proof)))
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
