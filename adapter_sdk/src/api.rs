use std::collections::HashMap;

use anyhow::anyhow;
use nexus_core::types::{AccountState, NexusHeader, TransactionV2, H256};

#[derive(Debug, Clone)]
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

    pub async fn get_range(&self) -> Result<Vec<H256>, anyhow::Error> {
        let response = reqwest::get("http://127.0.0.1:7000/range").await?;

        // Check if the request was successful
        if !response.status().is_success() {
            println!(
                "⛔️ Request to nexus failed with status {}. Nexus must be down",
                response.status()
            );

            return Err(anyhow!(
                "GET request failed with status: {}",
                response.status()
            ));
        }

        let range: Vec<H256> = response.json().await?;

        Ok(range)
    }

    pub async fn get_account_state(
        &self,
        app_account_id: &H256,
    ) -> Result<AccountState, anyhow::Error> {
        let app_account_id = hex::encode(app_account_id.as_slice());
        let mut params = HashMap::new();
        params.insert("app_account_id".to_string(), app_account_id.to_string());

        let response = self
            .client
            .get(&format!("{}/account", self.url))
            .query(&params)
            .send()
            .await?;

        if response.status().is_success() {
            let account: AccountState = response.json().await?;

            Ok(account)
        } else {
            Err(anyhow!(
                "Request failed with status code: {}",
                response.status()
            ))
        }
    }
}
