use anyhow::anyhow;
use nexus_core::types::{NexusHeader, TransactionV2, H256};

pub struct NexusAPI {
    url: String,
    client: reqwest::Client,
}

impl NexusAPI {
    pub fn new(url: &str) -> Self {
        Self {
            url: String::from(url),
            client: reqwest::Client::new(),
        }
    }

    pub async fn send_tx(&self, tx: TransactionV2) -> Result<String, anyhow::Error> {
        let url_with_tx = format!("{}/tx", self.url);

        let response = self.client.post(url_with_tx).json(&tx).send().await?;

        // Check if the request was successful
        if response.status().is_success() {
            Ok(match response.text().await {
                Ok(i) => i,
                Err(e) => return Err(anyhow!(e)),
            })
        } else {
            Err(anyhow!(
                "Post transaction call failed with status code: {}",
                response.status()
            ))
        }
    }

    pub async fn get_header(&self, hash: &H256) -> Result<NexusHeader, anyhow::Error> {
        let hash_hex = hex::encode(hash.as_slice());

        let url_with_hash = format!("{}/header?hash={}", self.url, hash_hex);

        let response = self.client.get(&url_with_hash).send().await?;

        if response.status().is_success() {
            let nexus_header: NexusHeader = response.json().await?;

            Ok(nexus_header)
        } else {
            Err(anyhow!(
                "Request failed with status code: {}",
                response.status()
            ))
        }
    }
}
