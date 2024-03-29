use std::future::Future;

use crate::types::H256;
use crate::{
    agg_types::{AggregatedTransaction, SubmitProofTransaction},
    types::AppId,
};
use anyhow::Error;
#[cfg(any(feature = "native"))]
use avail_subxt::avail::PairSigner;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

pub use sparse_merkle_tree::traits::Hasher;

pub trait Leaf<K> {
    fn get_key(&self) -> K;
}

pub trait Aggregator {
    fn aggregate(
        &self,
        submit_proof_txs: Vec<SubmitProofTransaction>,
    ) -> Result<AggregatedTransaction, Error>;
}
