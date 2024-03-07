use core::convert::Infallible;
use nexus_core::mempool::{self, Mempool};
use nexus_core::types::TransactionV2;
use std::sync::Mutex;
use warp::{reply::Reply, Filter, Rejection};

pub fn routes(mempool: Mempool) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    let buy_nft = warp::post()
        .and(warp::path("tx"))
        .and(warp::any().map(move || mempool.clone()))
        .and(warp::body::json())
        .and_then(submit_tx);

    buy_nft
}

pub async fn submit_tx(mempool: Mempool, tx: TransactionV2) -> Result<String, Infallible> {
    mempool.add_tx(tx).await;

    Ok(String::from("Added tx"))
}
