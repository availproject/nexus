use anyhow::Error;
use core::convert::Infallible;
use nexus_core::db::NodeDB;
use nexus_core::mempool::{self, Mempool};
use nexus_core::state_machine::StateMachine;
use nexus_core::types::{AvailHeader, HeaderStore, TransactionV2, H256};
use nexus_core::zkvm::traits::ZKProof;
use serde::{de::DeserializeOwned, Serialize};
use serde_json;
use std::fmt::Debug;
use std::sync::Arc;
use tokio::sync::Mutex;
use warp::{reply::Reply, Filter, Rejection};

pub fn routes<P: ZKProof + Serialize + Clone + DeserializeOwned + Debug + Send>(
    mempool: Mempool<P>,
    db: Arc<Mutex<NodeDB>>,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    let mempool_clone = mempool.clone();

    let tx = warp::path("tx")
        .and(warp::post())
        .and(warp::any().map(move || mempool_clone.clone()))
        .and(warp::body::json())
        .and_then(submit_tx::<P>);

    let submit_batch = warp::path("range")
        .and(warp::get())
        .and(warp::any().map(move || db.clone()))
        .and_then(range);

    tx.or(submit_batch)
}

pub async fn submit_tx<P: ZKProof + Serialize + Clone + DeserializeOwned + Debug + Send>(
    mempool: Mempool<P>,
    tx: TransactionV2<P>,
) -> Result<String, Infallible> {
    mempool.add_tx(tx).await;

    Ok(String::from("Added tx"))
}

pub async fn range(db: Arc<Mutex<NodeDB>>) -> Result<String, Infallible> {
    let db_lock = db.lock().await;

    let header_store: HeaderStore = match db_lock.get(b"previous_headers") {
        Ok(Some(i)) => i,
        Ok(None) => HeaderStore::new(32),
        Err(_) => panic!("Header not error"),
    };

    let range: Vec<H256> = header_store.inner().iter().map(|h| h.hash()).collect();

    let string =
        serde_json::to_string(&range).expect("Failed to serialize AvailHeader vector to JSON");

    Ok(string)
}
