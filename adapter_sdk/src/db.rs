use crate::state::QueueItem;
use crate::traits::RollupProof;
use crate::types::AdapterPublicInputs;
use anyhow::Error;
use nexus_core::db::NodeDB;
use nexus_core::zkvm::traits::ZKProof;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::collections::VecDeque;
use std::marker::PhantomData;

pub struct DB<P, ZP>(NodeDB, PhantomData<P>, PhantomData<ZP>);

impl<
        P: RollupProof + Clone + DeserializeOwned + Serialize,
        ZP: ZKProof + DeserializeOwned + Serialize + Clone,
    > DB<P, ZP>
{
    pub fn from_path(path: String) -> Self {
        Self(NodeDB::from_path(path), PhantomData, PhantomData)
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

    pub(crate) fn get_last_proof(&self) -> Result<Option<(ZP, AdapterPublicInputs, u32)>, Error> {
        Ok(self
            .0
            .get::<(ZP, AdapterPublicInputs, u32)>(b"last_proof")?)
    }

    pub(crate) fn store_last_proof(
        &self,
        proof: &(ZP, AdapterPublicInputs, u32),
    ) -> Result<(), Error> {
        self.0.put(b"last_proof", &proof)
    }
}
