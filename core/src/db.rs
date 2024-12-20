use std::collections::HashMap;

use crate::types::H256;
use anyhow::{anyhow, Error};
use rocksdb::{Options, WriteBatchWithTransaction, DB};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::{from_slice, to_vec};
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
