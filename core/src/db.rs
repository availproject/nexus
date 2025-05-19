use std::collections::HashMap;

use crate::traits::NexusTransaction;
use crate::types::BlockProof;
use crate::types::BlockStatus;
use crate::types::NexusBlockWithPointers;
use crate::types::NexusBlockWithPointersDbResponse;
use crate::types::TransactionWithStatus;
use crate::types::H256;
use anyhow::{anyhow, Error};
use rocksdb::{Options, WriteBatchWithTransaction, DB};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::{from_slice, to_vec};
use serde_json::{from_str, to_string};
use sqlx::{migrate::Migrator, postgres::PgPoolOptions, PgPool};
use tracing::{debug, error, info, instrument, span, Level};

pub struct NodeDB {
    db: DB,
}

pub struct BatchTransaction(pub WriteBatchWithTransaction<false>);

impl BatchTransaction {
    #[instrument(level = "debug")]
    pub fn new() -> Self {
        debug!("Creating new BatchTransaction");
        Self(rocksdb::WriteBatchWithTransaction::<false>::default())
    }

    #[instrument(level = "debug", skip(self, value), fields(key = ?hex::encode(serialized_key)))]
    pub fn put<V: Serialize>(&mut self, serialized_key: &[u8], value: &V) -> Result<(), Error> {
        debug!("Adding put operation to batch");
        self.0.put(serialized_key, to_vec(&value)?);
        debug!("Put operation added successfully");
        Ok(())
    }
}

impl NodeDB {
    #[instrument(level = "debug")]
    pub fn from_path(path: &str) -> Self {
        let mut db_options = Options::default();
        db_options.create_if_missing(true);

        debug!("Opening RocksDB at path: {}", path);
        let db = DB::open(&db_options, path).expect("unable to open rocks db.");
        info!("RocksDB opened successfully");

        NodeDB { db }
    }

    #[instrument(level = "debug", skip(db))]
    pub fn with_db(db: DB) -> Self {
        debug!("Creating NodeDB with existing DB instance");
        NodeDB { db }
    }

    #[instrument(level = "debug", skip(self))]
    pub fn db_asref(&self) -> &DB {
        debug!("Returning reference to internal DB");
        &self.db
    }

    #[instrument(level = "debug", skip(self), fields(key = ?hex::encode(serialized_key)))]
    pub fn get<V: DeserializeOwned>(&self, serialized_key: &[u8]) -> Result<Option<V>, Error> {
        debug!("Attempting to get value from DB");
        match self.db.get(serialized_key) {
            Err(e) => {
                error!("Error getting value: {}", e);
                Err(anyhow!("{}", e.to_string()))
            }
            Ok(None) => {
                debug!("No value found for key");
                Ok(None)
            }
            Ok(Some(i)) => {
                debug!("Value found, deserializing");
                Ok(Some(from_slice(&i)?))
            }
        }
    }

    #[instrument(level = "debug", skip(self, value), fields(key = ?hex::encode(serialized_key)))]
    pub fn put<V: Serialize>(&self, serialized_key: &[u8], value: &V) -> Result<(), Error> {
        debug!("Attempting to put value in DB");
        match self.db.put(serialized_key, to_vec(&value)?) {
            Err(e) => {
                error!("Error putting value: {}", e);
                Err(anyhow!("{}", e.to_string()))
            }
            _ => {
                debug!("Value put successfully");
                Ok(())
            }
        }
    }

    #[instrument(level = "debug", skip(self), fields(key = ?hex::encode(serialized_key)))]
    pub fn delete(&self, serialized_key: &[u8]) -> Result<(), Error> {
        debug!("Attempting to delete key from DB");
        match self.db.get(serialized_key) {
            Err(e) => {
                error!("Error checking key existence: {}", e);
                Err(anyhow!("{}", e.to_string()))
            }
            Ok(Some(_)) => match self.db.delete(serialized_key) {
                Err(e) => {
                    error!("Error deleting key: {}", e);
                    Err(anyhow!("{}", e.to_string()))
                }
                _ => {
                    debug!("Key deleted successfully");
                    Ok(())
                }
            },
            Ok(None) => {
                debug!("Key not found, nothing to delete");
                Ok(())
            }
        }
    }

    #[instrument(level = "debug", skip(self, batch_tx))]
    pub fn put_batch(&self, batch_tx: BatchTransaction) -> Result<(), Error> {
        debug!("Attempting to write batch to DB");
        self.db
            .write(batch_tx.0)
            .map_err(|e| {
                error!("Failed to write batch: {}", e);
                anyhow!("Failed to write batch: {}", e)
            })
            .map(|_| {
                debug!("Batch written successfully");
            })
    }

    #[instrument(level = "debug", skip(self))]
    pub fn get_current_root(&self) -> Result<Option<H256>, Error> {
        debug!("Attempting to get current root");
        self.get(b"current-root")
    }

    #[instrument(level = "debug", skip(self))]
    pub fn set_current_root(&self, root: &H256) -> Result<(), Error> {
        debug!("Attempting to set current root");
        self.put(b"current-root", root)
    }
}

pub struct SharedDB {
    db: PgPool,
}

// static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

impl SharedDB {
    pub async fn init(db_url: String) -> anyhow::Result<Self> {
        let pool = PgPoolOptions::new().max_connections(5).connect(&db_url).await?;

        // Run migration
        // MIGRATOR.run(&pool).await.expect("Migration failed");
        info!("Postgres DB opened successfully");
        Ok(Self { db: pool })
    }

    pub async fn get_latest_proven_block(&self) -> anyhow::Result<Option<i64>> {
        let block_number = sqlx::query_scalar!(
            r#"
            SELECT block_number FROM nexus_blocks
            WHERE block_status = $1
            ORDER BY block_number DESC
            LIMIT 1
            "#,
            BlockStatus::ProofGenerationSuccessful.to_string()
        )
        .fetch_optional(&self.db)
        .await?;

        Ok(block_number)
    }

    pub async fn insert_nexus_block_with_pointers(&self, data: &NexusBlockWithPointers) -> anyhow::Result<()> {
        let header_hash = data.block.header.hash();
        let block_number = data.block.header.number;
        let res = sqlx::query!(
            r#"
            INSERT INTO nexus_blocks (block_hash, block_number, block, jmt_version, zkvm_inputs, block_status)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
            header_hash.as_slice(),
            block_number as i64,
            to_string(&data.block)?,
            data.jmt_version as i64,
            bincode::serialize(&data.zkvm_inputs)?, // Using bincode here because serde doesn't work.
            data.block_status.to_string()
        )
        .execute(&self.db)
        .await
        .map_err(|e| anyhow!("Failed to insert nexus block with pointers: {}", e))?;
        Ok(())
    }

    pub async fn insert_transaction(&self, tx: &TransactionWithStatus) -> anyhow::Result<()> {
        let transaction_hash = tx.transaction.hash();
        //TODO: Move the prepare command to host and .sql
        sqlx::query!(
            r#"
            INSERT INTO transaction_with_status (transaction_hash, transaction, status, block_hash)
            VALUES ($1, $2, $3, $4)
            "#,
            transaction_hash.as_slice(),
            to_string(&tx.transaction)?,
            tx.status.to_string(),
            tx.block_hash.map(|h| h.as_slice().to_vec())
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }

    pub async fn get_block_with_number(&self, block_number: u64) -> anyhow::Result<Option<NexusBlockWithPointers>> {
        let block = sqlx::query_as!(
            NexusBlockWithPointersDbResponse,
            r#"
            SELECT 
                block_hash, 
                block_number, 
                block, 
                jmt_version, 
                zkvm_inputs, 
                block_status
            FROM nexus_blocks
            WHERE block_number = $1
            "#,
            block_number as i64
        )
        .fetch_optional(&self.db)
        .await?;

        match block {
            Some(block) => Ok(Some(NexusBlockWithPointers {
                block: from_str(&block.block)?,
                jmt_version: block.jmt_version as u64,
                zkvm_inputs: bincode::deserialize(&block.zkvm_inputs)?,
                block_status: BlockStatus::from_string(block.block_status),
            })),
            None => Ok(None),
        }
    }

    pub async fn update_block_status(&self, block_number: u64, new_status: BlockStatus) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
            UPDATE nexus_blocks
            SET block_status = $1
            WHERE block_number = $2
            "#,
            new_status.to_string(),
            block_number as i64
        )
        .execute(&self.db)
        .await?;
        Ok(())
    }

    pub async fn insert_proof(&self, block_number: u64, block_hash: H256, proof: Vec<u8>) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO proofs (block_hash, block_number, proof)
            VALUES ($1, $2, $3)
            "#,
            block_hash.as_slice(),
            block_number as i64,
            proof.as_slice()
        )
        .execute(&self.db)
        .await?;
        Ok(())
    }

    pub async fn get_block_proof_by_number(&self, block_number: u64) -> anyhow::Result<Option<BlockProof>> {
        let block_proof = sqlx::query_as!(
            BlockProof,
            r#"
            SELECT * FROM proofs
            WHERE block_number = $1
            "#,
            block_number as i64
        )
        .fetch_optional(&self.db)
        .await?;
        Ok(block_proof)
    }
}
