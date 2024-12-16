use crate::{
    db::NodeDB,
    traits::NexusTransaction,
    types::{Transaction, TransactionStatus, TransactionWithStatus},
};
use anyhow::anyhow;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct Mempool {
    tx_list: Arc<Mutex<Vec<Transaction>>>,
    node_db: Arc<Mutex<NodeDB>>,
}

impl Mempool {
    pub fn new(node_db: Arc<Mutex<NodeDB>>) -> Self {
        Self {
            tx_list: Arc::new(Mutex::new(vec![])),
            node_db,
        }
    }

    pub async fn get_current_txs(&self) -> (Vec<Transaction>, Option<usize>) {
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

    pub async fn add_tx(&self, tx: Transaction) -> Result<(), anyhow::Error> {
        let mut node_db = self.node_db.lock().await;
        let tx_hash = tx.hash();
        match node_db.get::<TransactionWithStatus>(tx_hash.as_slice()) {
            Ok(Some(i)) => Err(anyhow!("Transaction already exists")),
            Ok(None) => {
                node_db.put(
                    tx_hash.as_slice(),
                    &TransactionWithStatus {
                        transaction: tx.clone(),
                        status: TransactionStatus::InPool,
                        block_hash: None,
                    },
                );
                let mut tx_list = self.tx_list.lock().await;

                tx_list.push(tx);

                println!("Added tx. Total txs for next batch : {}", tx_list.len());

                Ok(())
            }
            Err(e) => Err(anyhow!("Internal mempool error")),
        }
    }
}
