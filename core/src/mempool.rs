use crate::{agg_types::InitTransaction, types::TransactionV2, zkvm::traits::ZKVMProof};
use anyhow::{anyhow, Error};
use serde::Serialize;
use std::fmt::Debug as DebugTrait;
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

    pub async fn get_current_txs(&self) -> (Vec<TransactionV2>, Option<usize>) {
        let tx_list = self.tx_list.lock().await;

        (
            tx_list.clone(),
            match tx_list.len() {
                0 => None,
                i => Some(i),
            },
        )
    }

    pub async fn clear_upto_tx(&self, index: usize) -> () {
        let mut tx_list = self.tx_list.lock().await;
        println!("Clearing tx list {index} {}", tx_list.len());
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

        println!("Added tx. Total txs for next batch : {}", tx_list.len());
    }
}
