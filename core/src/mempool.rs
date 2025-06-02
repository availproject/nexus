use crate::{
    db::NodeDB,
    metrics::MempoolMetrics,
    traits::NexusTransaction,
    types::{Transaction, TransactionStatus, TransactionWithStatus},
};
use anyhow::anyhow;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, error, event, info, instrument, span, warn, Level};

#[derive(Clone)]
pub struct Mempool {
    tx_list: Arc<Mutex<Vec<Transaction>>>,
    node_db: Arc<Mutex<NodeDB>>,
    mempool_metrics: MempoolMetrics,
}

impl Mempool {
    #[instrument(level = "debug", skip(node_db))]
    pub fn new(node_db: Arc<Mutex<NodeDB>>) -> Self {
        debug!("Creating new Mempool");
        Self {
            tx_list: Arc::new(Mutex::new(vec![])),
            node_db,
            mempool_metrics: MempoolMetrics::init(),
        }
    }

    #[instrument(level = "debug", skip(self))]
    pub async fn get_current_txs(&self) -> (Vec<Transaction>, Option<usize>) {
        debug!("Getting current transactions from mempool");
        let tx_list = self.tx_list.lock().await;

        (
            tx_list.clone(),
            match tx_list.len() {
                0 => None,
                i => Some(i),
            },
        )
    }

    #[instrument(level = "debug", skip(self))]
    pub async fn clear_upto_tx(&self, index: usize) -> () {
        debug!("Clearing transactions up to index {} from mempool", index);
        let (_, len) = self.get_current_txs().await;
        let mut tx_list = self.tx_list.lock().await;
        // Clear transactions up to the specified index
        if index < tx_list.len() {
            tx_list.drain(0..=index);
        } else {
            // Handle case where index exceeds the length of tx_list
            tx_list.clear();
        }
        if let Some(len) = len {
            self.mempool_metrics.mempool_txn_count_gauge.record((len - index) as u64, &[]);
            self.mempool_metrics.mempool_txn_count_histogram.record((len - index) as u64, &[]);
        }
    }

    #[instrument(level = "debug", skip(self))]
    pub async fn add_tx(&self, tx: Transaction) -> Result<(), anyhow::Error> {
        debug!("Adding transaction to mempool");
        let mut node_db = self.node_db.lock().await;
        let tx_hash = tx.hash();
        match node_db.get::<TransactionWithStatus>(tx_hash.as_slice()) {
            Ok(Some(i)) => {
                error!("Transaction already exists in mempool");
                Err(anyhow!("Transaction already exists"))
            }
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

                self.mempool_metrics.mempool_txn_count_gauge.record(tx_list.len() as u64, &[]);
                self.mempool_metrics.mempool_txn_count_histogram.record(tx_list.len() as u64, &[]);

                info!("Transaction successfully added to mempool");
                Ok(())
            }
            Err(e) => {
                error!("Internal mempool error: {}", e);
                Err(anyhow!("Internal mempool error"))
            }
        }
    }
}
