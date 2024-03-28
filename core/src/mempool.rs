use crate::{agg_types::InitTransaction, types::TransactionV2};
use anyhow::{anyhow, Error};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone, Debug)]
pub struct Mempool {
    tx_list: Arc<Mutex<Vec<TransactionV2>>>,
}

impl Mempool {
    pub fn new() -> Self {
        Self {
            tx_list: Arc::new(Mutex::new(vec![])),
        }
    }

    pub async fn get_current_txs(&self) -> (Vec<TransactionV2>, usize) {
        let tx_list = self.tx_list.lock().await;

        (tx_list.clone(), tx_list.len())
    }

    pub async fn clear_upto_tx(&self, index: usize) -> () {
        let mut tx_list = self.tx_list.lock().await;

        // Clear transactions up to the specified index
        if index < tx_list.len() {
            tx_list.drain(0..=index);
        } else {
            // Handle case where index exceeds the length of tx_list
            tx_list.clear();
        }
    }

    pub async fn add_tx(&self, tx: TransactionV2) {
        let mut tx_list = self.tx_list.lock().await;

        tx_list.push(tx);
    }
}
