use crate::state::QueueItem;
use crate::traits::Proof;
use crate::types::AdapterPublicInputs;
use anyhow::{anyhow, Context, Error};
use nexus_core::db::NodeDB;
use risc0_zkvm::Receipt;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use sp_core::H256;
use std::collections::VecDeque;
use std::marker::PhantomData;

pub struct DB<P>(NodeDB, PhantomData<P>);

impl<P: Proof + Clone + DeserializeOwned + Serialize> DB<P> {
    pub fn from_path(path: String) -> Self {
        Self(NodeDB::from_path(path), PhantomData)
    }

    pub(crate) fn get_last_known_queue(&self) -> Result<VecDeque<QueueItem<P>>, Error> {
        let result = self.0.get::<Vec<QueueItem<P>>>(b"blob_queue")?;

        Ok(match result {
            Some(i) => VecDeque::from_iter(i.into_iter()),
            None => VecDeque::new(),
        })
    }

    pub(crate) fn store_last_known_queue(
        &self,
        queue: &VecDeque<QueueItem<P>>,
    ) -> Result<(), Error> {
        let vec: Vec<QueueItem<P>> = queue.into_iter().map(|f| f.clone()).collect();

        self.0.put(b"blob_queue", &vec)
    }

    pub(crate) fn get_last_proof(
        &self,
    ) -> Result<Option<(Receipt, AdapterPublicInputs, u32)>, Error> {
        Ok(self
            .0
            .get::<(Receipt, AdapterPublicInputs, u32)>(b"last_proof")?)
    }

    pub(crate) fn store_last_proof(
        &self,
        proof: &(Receipt, AdapterPublicInputs, u32),
    ) -> Result<(), Error> {
        self.0.put(b"last_proof", &proof)
    }
}

pub struct HashDB(NodeDB);

#[derive(Serialize, Deserialize)]
pub struct InclusionData {
    block_hash: H256,
    pub transaction_index: u32,
}

impl InclusionData {
    pub(crate) fn new(block_hash: H256, transaction_index: u32) -> Self {
        InclusionData {
            block_hash,
            transaction_index,
        }
    }
}

impl HashDB {
    pub fn from_path(path: String) -> Self {
        Self(NodeDB::from_path(path))
    }

    pub(crate) fn put(&self, key: H256, value: InclusionData) -> Result<(), Error> {
        self.0.put(key.as_bytes(), &value);
        Ok(())
    }

    pub(crate) fn get(&self, key: H256) -> Result<InclusionData, Error> {
        let inclusion_proof: Result<Option<InclusionData>, Error> = self.0.get(key.as_bytes());
        match inclusion_proof {
            Ok(value) => Ok(value.unwrap()),
            Err(err) => Err(anyhow!("No entry")),
        }
    }
}
