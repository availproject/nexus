use core::convert::Infallible;
use jmt::ValueHash;
use nexus_core::db::NodeDB;
use nexus_core::mempool::{self, Mempool};
use nexus_core::state::sparse_merkle_tree::MerkleProof;
use nexus_core::state::{sparse_merkle_tree::traits::Value, VmState};
use nexus_core::state_machine::StateMachine;
use nexus_core::types::{
    AccountState, AccountWithProof, AvailHeader, HeaderStore, NexusHeader, TransactionV2, H256,
};
use nexus_core::zkvm::traits::ZKProof;
use risc0_zkvm::sha::rust_crypto::Sha256;
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;
use tokio::sync::Mutex;
use warp::{reply::Reply, Filter, Rejection};

use crate::AvailToNexusPointer;

pub fn route<P: ZKProof + Serialize + Clone + DeserializeOwned + Debug + Send>(
    mempool: Mempool<P>,
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
        .and_then(submit_tx::<P>);

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
                            Ok(i) => get_state(db, vm_state, &i).await,
                            Err(_) => Ok(String::from("Invalid hash")),
                        }
                    }
                    None => Ok(String::from("Hash parameter not provided")),
                }
            },
        );

    tx.or(submit_batch).or(header).or(account)
}

pub async fn submit_tx<P: ZKProof + Serialize + Clone + DeserializeOwned + Debug + Send>(
    mempool: Mempool<P>,
    tx: TransactionV2,
) -> Result<String, Infallible> {
    mempool.add_tx(tx).await;

    Ok(String::from("Added tx"))
}

pub async fn get_state(
    db: Arc<Mutex<NodeDB>>,
    state: Arc<Mutex<VmState>>,
    app_account_id: &H256,
) -> Result<String, Infallible> {
    let state_lock = state.lock().await;
    let db_lock = db.lock().await;

    let header_store: HeaderStore = match db_lock.get(b"previous_headers") {
        Ok(Some(i)) => i,
        Ok(None) => HeaderStore::new(32),
        Err(_) => panic!("Header store error"),
    };
    let (account_option, proof) = match state_lock.get_with_proof(app_account_id, 0) {
        Ok(i) => i,
        Err(e) => return Ok(String::from("Internal error")),
    };
    let root = match state_lock.get_root(0) {
        Ok(i) => i,
        Err(e) => return Ok(String::from("Internal error")),
    };

    let account = if let Some(a) = account_option {
        a
    } else {
        AccountState::zero()
    };
    let siblings: Vec<[u8; 32]> = proof
        .siblings()
        .iter()
        .map(|s| s.hash::<Sha256>())
        .collect();
    let value_hash = ValueHash::with::<Sha256>(account.encode()).0;

    let response = AccountWithProof {
        account: account.clone(),
        proof: siblings.clone(),
        value_hash: value_hash.clone(),
        account_encoded: hex::encode(account.encode()),
        nexus_header: match header_store.first() {
            Some(i) => i.clone(),
            None => return Ok(String::from("No headers available.")),
        },
        //TODO: Remove below unwrap
        proof_hex: siblings.iter().map(|s| hex::encode(s)).collect(),
        value_hash_hex: hex::encode(value_hash),
        nexus_state_root_hex: hex::encode(root.as_fixed_slice()),
    };

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
