use anyhow::Error;
use core::convert::Infallible;
use nexus_core::db::NodeDB;
use nexus_core::mempool::{self, Mempool};
use nexus_core::state::sparse_merkle_tree::MerkleProof;
use nexus_core::state::{sparse_merkle_tree::traits::Value, VmState};
use nexus_core::state_machine::StateMachine;
use nexus_core::types::{
    AccountState, AccountWithProof, AvailHeader, HeaderStore, NexusHeader, TransactionV2, H256,
};
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use warp::{reply::Reply, Filter, Rejection};

use crate::AvailToNexusPointer;

pub fn routes(
    mempool: Mempool,
    db: Arc<Mutex<NodeDB>>,
    vm_state: Arc<Mutex<VmState>>,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    let mempool_clone = mempool.clone();
    let db_clone = db.clone();
    let db_clone_2 = db.clone();

    let tx = warp::path("tx")
        .and(warp::post())
        .and(warp::any().map(move || mempool_clone.clone()))
        .and(warp::body::json())
        .and_then(submit_tx);

    let submit_batch = warp::path("range")
        .and(warp::get())
        .and(warp::any().map(move || db_clone_2.clone()))
        .and_then(range);

    let header = warp::path("header")
        .and(warp::get())
        .and(warp::any().map(move || db_clone.clone()))
        .and(warp::query::<HashMap<String, String>>())
        .and_then(
            |db: Arc<Mutex<NodeDB>>, params: HashMap<String, String>| async move {
                match params.get("hash") {
                    Some(hash_str) => {
                        let avail_hash = H256::try_from(hash_str.as_str());
                        match avail_hash {
                            Ok(hash) => header(db, hash).await,
                            Err(_) => Ok(String::from("Invalid hash")),
                        }
                    }
                    None => Ok(String::from("Hash parameter not provided")),
                }
            },
        );

    let account = warp::path("account")
        .and(warp::get())
        .and(warp::any().map(move || db.clone()))
        .and(warp::any().map(move || vm_state.clone()))
        .and(warp::query::<HashMap<String, String>>())
        .and_then(
            |db: Arc<Mutex<NodeDB>>,
             vm_state: Arc<Mutex<VmState>>,
             params: HashMap<String, String>| async move {
                match params.get("app_account_id") {
                    Some(hash_str) => {
                        let app_account_id = H256::try_from(hash_str.as_str());
                        match app_account_id {
                            Ok(i) => get_state(vm_state, &i).await,
                            Err(_) => Ok(String::from("Invalid hash")),
                        }
                    }
                    None => Ok(String::from("Hash parameter not provided")),
                }
            },
        );

    tx.or(submit_batch).or(header).or(account)
}

//TODO: Better status codes and error handling.
pub async fn submit_tx(mempool: Mempool, tx: TransactionV2) -> Result<String, Infallible> {
    mempool.add_tx(tx).await;

    Ok(String::from("Added tx"))
}

pub async fn get_state(
    state: Arc<Mutex<VmState>>,
    app_account_id: &H256,
) -> Result<String, Infallible> {
    let state_lock = state.lock().await;

    let (account, proof): (AccountState, MerkleProof) =
        match state_lock.get_with_proof(app_account_id) {
            Ok(i) => i,
            Err(e) => return Ok(String::from("Internal error")),
        };

    let response = AccountWithProof { account, proof };

    Ok(serde_json::to_string(&response).expect("Failed to serialize Account to JSON"))
}

pub async fn header(db: Arc<Mutex<NodeDB>>, avail_hash: H256) -> Result<String, Infallible> {
    let db_lock = db.lock().await;

    let nexus_hash: H256 = match db_lock.get::<AvailToNexusPointer>(avail_hash.as_slice()) {
        Ok(Some(i)) => i.nexus_hash,
        Ok(None) => return Ok(String::from("Avail header not yet processed.")),
        Err(_) => panic!("Node DB error. Cannot find mapping"),
    };

    let nexus_header: NexusHeader = match db_lock.get(nexus_hash.as_slice()) {
        Ok(Some(i)) => i,
        Ok(None) => return Ok(String::from("Internal error")),
        Err(_) => panic!("Node DB error. Cannot find nexus header"),
    };

    Ok(serde_json::to_string(&nexus_header).expect("Failed to serialize AvailHeader to JSON"))
}

pub async fn range(db: Arc<Mutex<NodeDB>>) -> Result<String, Infallible> {
    let db_lock = db.lock().await;

    let header_store: HeaderStore = match db_lock.get(b"previous_headers") {
        Ok(Some(i)) => i,
        Ok(None) => HeaderStore::new(32),
        Err(_) => panic!("Header store error"),
    };

    let range: Vec<H256> = header_store.inner().iter().map(|h| h.hash()).collect();

    let string =
        serde_json::to_string(&range).expect("Failed to serialize AvailHeader vector to JSON");

    Ok(string)
}
