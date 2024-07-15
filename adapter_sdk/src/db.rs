use crate::state::QueueItem;
use crate::traits::Proof;
use crate::types::AdapterPublicInputs;
use anyhow::Error;
use nexus_core::db::NodeDB;
use risc0_zkvm::Receipt;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::collections::VecDeque;
use std::marker::PhantomData;

pub struct DB<P>(NodeDB, PhantomData<P>);

impl<P: Proof + Clone + DeserializeOwned + Serialize> DB<P> {
    pub fn from_path(path: &str) -> Self {
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
