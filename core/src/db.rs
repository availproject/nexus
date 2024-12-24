use std::collections::HashMap;

use anyhow::{anyhow, Error};
use rocksdb::{Options, WriteBatchWithTransaction, DB};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::{from_slice, to_vec};
use sparse_merkle_tree::H256;

//Wrapper class to RocksDB which is used as backing storage.
pub struct NodeDB {
    db: DB,
}

// TODO: Check if Batch updates are atomic with TRANSACTION false
pub struct BatchTransaction(pub WriteBatchWithTransaction<false>);

impl BatchTransaction {
    pub fn new() -> Self {
        Self(rocksdb::WriteBatchWithTransaction::<false>::default())
    }
    pub fn put<V: Serialize>(&mut self, serialized_key: &[u8], value: &V) -> Result<(), Error> {
        self.0.put(serialized_key, to_vec(&value)?);

        Ok(())
    }
}

impl NodeDB {
    pub fn from_path(path: &str) -> Self {
        let mut db_options = Options::default();
        db_options.create_if_missing(true);

        let db = DB::open(&db_options, path).expect("unable to open rocks db.");

        NodeDB { db }
    }

    pub fn with_db(db: DB) -> Self {
        NodeDB { db }
    }

    pub fn db_asref(&self) -> &DB {
        &self.db
    }

    pub fn get<V: DeserializeOwned>(&self, serialized_key: &[u8]) -> Result<Option<V>, Error> {
        match self.db.get(serialized_key) {
            Err(e) => Err(anyhow!("{}", e.to_string())),
            Ok(None) => Ok(None),
            Ok(Some(i)) => Ok(Some(from_slice(&i)?)),
        }
    }

    pub fn put<V: Serialize>(&self, serialized_key: &[u8], value: &V) -> Result<(), Error> {
        match self.db.put(serialized_key, to_vec(&value)?) {
            Err(e) => Err(anyhow!("{}", e.to_string())),
            _ => Ok(()),
        }
    }

    pub fn delete(&self, serialized_key: &[u8]) -> Result<(), Error> {
        match self.db.get(serialized_key) {
            Err(e) => Err(anyhow!("{}", e.to_string())),
            Ok(Some(_)) => match self.db.delete(serialized_key) {
                Err(e) => Err(anyhow!("{}", e.to_string())),
                _ => Ok(()),
            },
            Ok(None) => Ok(()),
        }
    }

    pub fn put_batch(&self, batch_tx: BatchTransaction) -> Result<(), Error> {
        self.db
            .write(batch_tx.0)
            .map_err(|e| anyhow!("Failed to write batch: {}", e))
    }

    pub fn get_current_root(&self) -> Result<Option<H256>, Error> {
        self.get(b"current-root")
    }

    pub fn set_current_root(&self, root: &H256) -> Result<(), Error> {
        self.put(b"current-root", root)
    }
}
