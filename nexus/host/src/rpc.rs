use core::convert::Infallible;
use nexus_core::agg_types::{AggregatedTransaction, InitTransaction};
use nexus_core::mempool::{self, Mempool};
use nexus_core::simple_state_machine::StateMachine;
use nexus_core::types::TransactionV2;
use std::sync::Mutex;
use warp::{reply::Reply, Filter, Rejection};

use crate::BatchesToAggregate;

pub fn routes(
    mempool: Mempool,
    batches_to_aggregate: BatchesToAggregate,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    let mempool_clone = mempool.clone();

    let buy_nft = warp::post()
        .and(warp::path("tx"))
        .and(warp::any().map(move || mempool_clone.clone()))
        .and(warp::body::json())
        .and_then(submit_init_tx);

    let submit_batch = warp::post()
        .and(warp::path("tx"))
        .and(warp::any().map(move || (mempool.clone(), batches_to_aggregate.clone())))
        .and(warp::body::json())
        .and_then(submit_batch);

    buy_nft
}

pub async fn submit_init_tx(mempool: Mempool, tx: InitTransaction) -> Result<String, Infallible> {
    mempool.add_tx(tx).await;

    Ok(String::from("Added tx"))
}

pub async fn submit_batch(
    mempool: (Mempool, BatchesToAggregate),
    tx: AggregatedTransaction,
) -> Result<String, Infallible> {
    let (txs, size) = mempool.0.get_current_txs().await;

    mempool.1.add_batch((txs, tx)).await;

    mempool.0.clear_upto_tx(size).await;

    Ok(String::from("Added batch"))
}
