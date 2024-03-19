use std::future::Future;

use crate::types::H256;
use crate::{
    agg_types::{AggregatedTransaction, SubmitProofTransaction},
    types::AppId,
};
use anyhow::{anyhow, Error};
#[cfg(any(feature = "native"))]
use avail_subxt::avail::PairSigner;

pub trait Leaf<K> {
    fn get_key(&self) -> K;
}

pub trait Aggregator {
    fn aggregate(
        &self,
        submit_proof_txs: Vec<SubmitProofTransaction>,
    ) -> Result<AggregatedTransaction, Error>;
}

#[cfg(any(feature = "native"))]
pub trait RollupAdapter<PI: RollupPublicInputs, P: Proof<PI>> {
    fn new(app_id: AppId, avail_signer: PairSigner) -> Self;
    fn start() -> impl Future<Output = Result<(), anyhow::Error>>;
    fn submit_blob(blob: Vec<u8>) -> impl Future<Output = Result<(), anyhow::Error>>;
    fn submit_proof_for_blob() -> impl Future<Output = Result<(), anyhow::Error>>;
}

pub trait Proof<PI> {
    fn verify(&self, vk: &[u8; 32], public_inputs: &PI) -> Result<(), Error>;
}

pub trait RollupPublicInputs {
    fn prev_state_root(&self) -> H256;
    fn post_state_root(&self) -> H256;
    fn blob_hash(&self) -> H256;
}
