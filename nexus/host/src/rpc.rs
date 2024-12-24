use core::convert::Infallible;
use jmt::ValueHash;
use nexus_core::db::NodeDB;
use nexus_core::mempool::Mempool;
use nexus_core::state::VmState;
use nexus_core::state_machine::StateMachine;
use nexus_core::types::{
    AccountState, AccountWithProof, AvailHeader, HeaderStore, NexusBlockWithPointers, NexusHeader,
    StatementDigest, Transaction, TransactionWithStatus, H256,
};
use nexus_core::utils::hasher::Sha256;
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use warp::{reply::Reply, Filter, Rejection};

use crate::AvailToNexusPointer;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AccountStateHex {
    pub statement: String,
    pub state_root: String,
    pub start_nexus_hash: String,
    pub last_proof_height: u32,
    pub height: u32,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NexusHeaderHex {
    pub parent_hash: String,
    pub prev_state_root: String,
    pub state_root: String,
    pub avail_header_hash: String,
    pub number: u32,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AccountWithProofHex {
    pub account: AccountStateHex,
    pub proof: Vec<String>,
    pub value_hash: String,
    pub nexus_header: NexusHeaderHex,
}

impl From<NexusHeader> for NexusHeaderHex {
    fn from(value: NexusHeader) -> Self {
        Self {
            parent_hash: hex::encode(value.parent_hash.as_fixed_slice()),
            prev_state_root: hex::encode(value.prev_state_root.as_fixed_slice()),
            state_root: hex::encode(value.state_root.as_fixed_slice()),
            avail_header_hash: hex::encode(value.avail_header_hash.as_fixed_slice()),
            number: value.number,
        }
    }
}

impl From<AccountState> for AccountStateHex {
    fn from(value: AccountState) -> Self {
        Self {
            statement: value.statement.encode().to_string(),
            state_root: hex::encode(value.state_root),
            start_nexus_hash: hex::encode(value.start_nexus_hash),
            last_proof_height: value.last_proof_height,
            height: value.height,
        }
    }
}

impl From<AccountWithProof> for AccountWithProofHex {
    fn from(value: AccountWithProof) -> Self {
        Self {
            account: AccountStateHex::from(value.account),
            proof: value.proof_hex,
            value_hash: value.value_hash_hex,
            nexus_header: NexusHeaderHex::from(value.nexus_header),
        }
    }
}

pub fn routes(
    mempool: Mempool,
    db: Arc<Mutex<NodeDB>>,
    vm_state: Arc<Mutex<VmState>>,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    let mempool_clone = mempool.clone();
    let db_clone = db.clone();
    let vm_state_clone = vm_state.clone();
    let db_clone_2 = db.clone();
    let db_clone_3 = db.clone();
    let db_clone_4 = db.clone();

    let tx = warp::path("tx")
        .and(warp::post())
        .and(warp::any().map(move || mempool_clone.clone()))
        .and(warp::body::json())
        .and_then(submit_tx);
    let tx_status = warp::path("tx_status")
        .and(warp::get())
        .and(warp::any().map(move || db_clone_4.clone()))
        .and(warp::query::<HashMap<String, String>>())
        .and_then(
            |db: Arc<Mutex<NodeDB>>, params: HashMap<String, String>| async move {
                match params.get("tx_hash") {
                    Some(hash_str) => {
                        let tx_hash = H256::try_from(hash_str.as_str());
                        match tx_hash {
                            Ok(hash) => tx_status(db, hash).await,
                            Err(_) => Ok(String::from("Invalid hash")),
                        }
                    }
                    None => Ok(String::from("Hash parameter not provided")),
                }
            },
        );

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
                        let block_hash = match params.get("block_hash") {
                            Some(i) => match H256::try_from(i.as_str()) {
                                Ok(i) => Some(i),
                                Err(_) => return Ok(String::from("Invalid hash")),
                            },
                            None => None,
                        };
                        let app_account_id = H256::try_from(hash_str.as_str());
                        match app_account_id {
                            Ok(i) => get_state(db, vm_state, &i, block_hash).await,
                            Err(_) => Ok(String::from("Invalid hash")),
                        }
                    }
                    None => Ok(String::from("Hash parameter not provided")),
                }
            },
        );

    let account_hex = warp::path("account-hex")
        .and(warp::get())
        .and(warp::any().map(move || db_clone_3.clone()))
        .and(warp::any().map(move || vm_state_clone.clone()))
        .and(warp::query::<HashMap<String, String>>())
        .and_then(
            |db: Arc<Mutex<NodeDB>>,
             vm_state: Arc<Mutex<VmState>>,
             params: HashMap<String, String>| async move {
                match params.get("app_account_id") {
                    Some(hash_str) => {
                        let app_account_id = H256::try_from(hash_str.as_str());
                        match app_account_id {
                            Ok(i) => get_state_hex(db, vm_state, &i).await,
                            Err(_) => Ok(String::from("Invalid hash")),
                        }
                    }
                    None => Ok(String::from("Hash parameter not provided")),
                }
            },
        );

    tx.or(tx_status)
        .or(submit_batch)
        .or(header)
        .or(account)
        .or(account_hex)
}

//TODO: Better status codes and error handling.
pub async fn submit_tx(mempool: Mempool, tx: Transaction) -> Result<String, Infallible> {
    match mempool.add_tx(tx).await {
        Ok(()) => Ok(String::from("Added tx")),
        Err(i) => Ok(String::from("Internal Mempool error")),
    }
}

pub async fn tx_status(db: Arc<Mutex<NodeDB>>, tx_hash: H256) -> Result<String, Infallible> {
    let db_lock = db.lock().await;
    println!("Getting tx status");
    match db_lock.get::<TransactionWithStatus>(tx_hash.as_slice()) {
        Ok(Some(i)) => Ok(serde_json::to_string(&i).expect("Failed to serialize Account to JSON")),
        Ok(None) => return Ok(String::from("Transaction not found")),
        Err(e) => return Ok(String::from("Internal error")),
    }
}

pub async fn get_state(
    db: Arc<Mutex<NodeDB>>,
    state: Arc<Mutex<VmState>>,
    app_account_id: &H256,
    block_hash: Option<H256>,
) -> Result<String, Infallible> {
    let state_lock = state.lock().await;
    let db_lock = db.lock().await;

    let header_store: HeaderStore = match db_lock.get(b"previous_headers") {
        Ok(Some(i)) => i,
        Ok(None) => HeaderStore::new(32),
        Err(_) => panic!("Header store error"),
    };
    // let current_version = match state_lock.get_version(true) {
    //     Ok(Some(i)) => i,
    //     Ok(None) => 0,
    //     Err(e) => return Ok(String::from("Internal db error")),
    // };
    let version: u64 = match block_hash {
        Some(i) => {
            match db_lock.get::<NexusBlockWithPointers>(&[i.as_slice(), b"-block"].concat()) {
                Ok(Some(i)) => i.jmt_version,
                Ok(None) => return Ok(String::from("Block hash not found")),
                Err(e) => return Ok(String::from("Internal db error")),
            }
        }
        None => match state_lock.get_version(true) {
            Ok(Some(i)) => i,
            Ok(None) => 0,
            Err(e) => return Ok(String::from("Internal db error")),
        },
    };

    let (account_option, proof) = match state_lock.get_with_proof(app_account_id, version) {
        Ok(i) => i,
        Err(e) => return Ok(String::from("Internal error")),
    };
    let root = match state_lock.get_root(version) {
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

pub async fn get_state_hex(
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
    let current_version = match state_lock.get_version(true) {
        Ok(Some(i)) => i,
        Ok(None) => 0,
        Err(e) => return Ok(String::from("Internal db error")),
    };

    let (account_option, proof) = match state_lock.get_with_proof(app_account_id, current_version) {
        Ok(i) => i,
        Err(e) => return Ok(String::from("Internal error")),
    };
    let root = match state_lock.get_root(current_version) {
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

    let account_with_proof = AccountWithProof {
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

    let response = AccountWithProofHex::from(account_with_proof);

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
