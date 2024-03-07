use crate::agg_types::{AggregatedTransaction, SubmitProofTransaction};
use anyhow::{anyhow, Error};

pub trait Leaf<K> {
    fn get_key(&self) -> K;
}

pub trait Aggregator {
    fn aggregate(
        &self,
        submit_proof_txs: Vec<SubmitProofTransaction>,
    ) -> Result<AggregatedTransaction, Error>;
}
