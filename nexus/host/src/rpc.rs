use core::convert::Infallible;
use jmt::ValueHash;
use nexus_core::db::NodeDB;
use nexus_core::mempool::Mempool;
use nexus_core::state::VmState;
use nexus_core::state_machine::StateMachine;
use nexus_core::types::{
    AccountState, AccountWithProof, AvailHeader, HeaderStore, NexusBlockWithPointers,
    NexusBlockWithTransactions, NexusHeader, StatementDigest, Transaction, TransactionWithStatus,
    H256,
};
use nexus_core::utils::hasher::Sha256;
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use warp::reject::custom;
use warp::reply::WithStatus;
use warp::{
    http::StatusCode, http::Uri, hyper::Response, path::FullPath, path::Tail, reply::Reply, Filter,
    Rejection,
};

use crate::AvailToNexusPointer;

use utoipa::OpenApi;
use utoipa_swagger_ui::Config;

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

#[derive(OpenApi)]
#[openapi(
    paths(
        health_check,
        submit_tx,
        tx_status,
        get_block,
        get_state,
        get_state_hex,
        get_header,
        range
    ),
    components(
        schemas(
            nexus_core::types::AccountWithProof,
            nexus_core::types::NexusBlockWithTransactions,
            nexus_core::types::TransactionWithStatus,
            nexus_core::types::Transaction,
            nexus_core::types::TxParams,
            nexus_core::types::SubmitProof,
            nexus_core::types::InitAccount,
            nexus_core::types::NexusHeader,
            nexus_core::types::TransactionStatus,
            nexus_core::state::types::AccountState,
            nexus_core::state::types::StatementDigest
        )
    ),
    tags(
        (name = "nexus", description = "Nexus Node API endpoints")
    )
)]
struct ApiDoc;

/// Check if the node is alive
#[utoipa::path(
    get,
    path = "/health",
    tag = "nexus",
    responses(
        (status = 200, description = "Node is alive", body = String)
    )
)]
async fn health_check() -> impl Reply {
    warp::reply::json(&serde_json::json!({"status": "Alive ser."}))
}

/// Submit a new transaction to the mempool
#[utoipa::path(
    post,
    path = "/tx",
    tag = "nexus",
    request_body = Transaction,
    responses(
        (status = 200, description = "Transaction added successfully", body = String),
        (status = 500, description = "Internal mempool error", body = String)
    )
)]
async fn submit_tx(mempool: Mempool, tx: Transaction) -> Result<WithStatus<String>, Rejection> {
    match mempool.add_tx(tx).await {
        Ok(()) => Ok(warp::reply::with_status(
            "Added tx".to_string(),
            warp::http::StatusCode::OK,
        )),
        Err(_) => Ok(warp::reply::with_status(
            "Internal Mempool error".to_string(),
            warp::http::StatusCode::INTERNAL_SERVER_ERROR,
        )),
    }
}

/// Get transaction status by hash
#[utoipa::path(
    get,
    path = "/tx_status",
    tag = "nexus",
    params(
        ("tx_hash" = String, Query, description = "Transaction hash in hex format")
    ),
    responses(
        (status = 200, description = "Transaction status found", body = TransactionWithStatus),
        (status = 404, description = "Transaction not found", body = String),
        (status = 400, description = "Invalid hash format", body = String),
        (status = 500, description = "Internal error", body = String)
    )
)]
async fn tx_status(db: Arc<Mutex<NodeDB>>, tx_hash: H256) -> Result<WithStatus<String>, Rejection> {
    let db_lock = db.lock().await;
    match db_lock.get::<TransactionWithStatus>(tx_hash.as_slice()) {
        Ok(Some(i)) => Ok(warp::reply::with_status(
            serde_json::to_string(&i).expect("Failed to serialize Account to JSON"),
            warp::http::StatusCode::OK,
        )),
        Ok(None) => Ok(warp::reply::with_status(
            "Transaction not found".to_string(),
            warp::http::StatusCode::NOT_FOUND,
        )),
        Err(_) => Ok(warp::reply::with_status(
            "Internal error".to_string(),
            warp::http::StatusCode::INTERNAL_SERVER_ERROR,
        )),
    }
}

/// Get block by hash or latest
#[utoipa::path(
    get,
    path = "/block",
    tag = "nexus",
    params(
        ("block_hash" = Option<String>, Query, description = "Block hash in hex format. If not provided, returns the latest block"),
        ("block_number" = Option<u32>, Query, description = "Block number to query. If not provided, returns the latest block")
    ),
    responses(
        (status = 200, description = "Block found", body = NexusBlockWithTransactions),
        (status = 404, description = "Block not found", body = String),
        (status = 400, description = "Invalid hash format", body = String),
        (status = 500, description = "Internal error", body = String)
    )
)]
async fn get_block(
    db: Arc<Mutex<NodeDB>>,
    block_hash_opt: Option<H256>,
    block_number_opt: Option<u32>,
) -> Result<WithStatus<String>, Rejection> {
    let db_lock = db.lock().await;

    let nexus_hash = if block_number_opt.is_some() {
        let block_number = block_number_opt.unwrap();
        match db_lock.get::<H256>(&[block_number.to_be_bytes().as_slice(), b"-block"].concat()) {
            Ok(Some(hash)) => hash,
            Ok(None) => {
                return Ok(warp::reply::with_status(
                    "Nexus height does not exist".to_string(),
                    warp::http::StatusCode::BAD_REQUEST,
                ))
            }
            Err(_) => {
                return Ok(warp::reply::with_status(
                    "Internal error when retrieving block number to hash mapping".to_string(),
                    warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                ))
            }
        }
    } else {
        match block_hash_opt {
            Some(hash) => hash,
            None => match db_lock.get::<HeaderStore>(b"previous_headers") {
                Ok(Some(headers)) => match headers.first().map(|h| h.hash()) {
                    Some(hash) => hash,
                    None => {
                        return Ok(warp::reply::with_status(
                            "Latest headers not retrievable".to_string(),
                            warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                        ))
                    }
                },
                _ => {
                    return Ok(warp::reply::with_status(
                        "Latest headers not retrievable".to_string(),
                        warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                    ))
                }
            },
        }
    };

    let block =
        match db_lock.get::<NexusBlockWithPointers>(&[nexus_hash.as_slice(), b"-block"].concat()) {
            Ok(Some(b)) => b,
            Ok(None) => {
                return Ok(warp::reply::with_status(
                    "Block not found".to_string(),
                    warp::http::StatusCode::BAD_REQUEST,
                ))
            }
            Err(_) => {
                return Ok(warp::reply::with_status(
                    "Error retrieving block".to_string(),
                    warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                ))
            }
        };

    let txs = block
        .block
        .transactions
        .iter()
        .filter_map(|tx_hash| {
            db_lock
                .get::<TransactionWithStatus>(tx_hash.hash.as_slice())
                .ok()
                .and_then(|opt_tx| opt_tx)
        })
        .collect::<Vec<_>>();

    if txs.len() != block.block.transactions.len() {
        return Ok(warp::reply::with_status(
            "Some transactions not found".to_string(),
            warp::http::StatusCode::INTERNAL_SERVER_ERROR,
        ));
    }

    let block_with_txs: NexusBlockWithTransactions = NexusBlockWithTransactions {
        transactions: txs,
        header: block.block.header,
    };

    match serde_json::to_string(&block_with_txs) {
        Ok(serialized_response) => Ok(warp::reply::with_status(
            serialized_response,
            warp::http::StatusCode::OK,
        )),
        Err(_) => Ok(warp::reply::with_status(
            "Internal encoding error".to_string(),
            warp::http::StatusCode::INTERNAL_SERVER_ERROR,
        )),
    }
}

/// Get account state and proof
#[utoipa::path(
    get,
    path = "/account",
    tag = "nexus",
    params(
        ("app_account_id" = String, Query, description = "Account ID in hex format"),
        ("block_hash" = Option<String>, Query, description = "Optional block hash in hex format. If not provided, uses latest state")
    ),
    responses(
        (status = 200, description = "Account state found", body = AccountWithProof),
        (status = 404, description = "Account not found", body = String),
        (status = 400, description = "Invalid hash format", body = String),
        (status = 500, description = "Internal error", body = String)
    )
)]
async fn get_state(
    db: Arc<Mutex<NodeDB>>,
    state: Arc<Mutex<VmState>>,
    app_account_id: &H256,
    block_hash: Option<H256>,
) -> Result<WithStatus<String>, Rejection> {
    let state_lock = state.lock().await;
    let db_lock = db.lock().await;

    let header_store: HeaderStore = match db_lock.get(b"previous_headers") {
        Ok(Some(i)) => i,
        Ok(None) => HeaderStore::new(32),
        Err(_) => {
            return Ok(warp::reply::with_status(
                "Header store error".to_string(),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    };

    let version: u64 = match block_hash {
        Some(i) => {
            match db_lock.get::<NexusBlockWithPointers>(&[i.as_slice(), b"-block"].concat()) {
                Ok(Some(i)) => i.jmt_version,
                Ok(None) => {
                    return Ok(warp::reply::with_status(
                        "Block hash not found".to_string(),
                        warp::http::StatusCode::BAD_REQUEST,
                    ))
                }
                Err(_) => {
                    return Ok(warp::reply::with_status(
                        "Internal db error".to_string(),
                        warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                    ))
                }
            }
        }
        None => match state_lock.get_version(true) {
            Ok(Some(i)) => i,
            Ok(None) => 0,
            Err(_) => {
                return Ok(warp::reply::with_status(
                    "Internal db error".to_string(),
                    warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                ))
            }
        },
    };

    let (account_option, proof) = match state_lock.get_with_proof(app_account_id, version) {
        Ok(i) => i,
        Err(_) => {
            return Ok(warp::reply::with_status(
                "Internal error".to_string(),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    };
    let root = match state_lock.get_root(version) {
        Ok(i) => i,
        Err(_) => {
            return Ok(warp::reply::with_status(
                "Internal error".to_string(),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    };

    let account = account_option.unwrap_or_else(AccountState::zero);
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
            None => {
                return Ok(warp::reply::with_status(
                    "No headers available.".to_string(),
                    warp::http::StatusCode::NOT_FOUND,
                ))
            }
        },
        proof_hex: siblings.iter().map(|s| hex::encode(s)).collect(),
        value_hash_hex: hex::encode(value_hash),
        nexus_state_root_hex: hex::encode(root.as_fixed_slice()),
    };

    let serialized_response = match serde_json::to_string(&response) {
        Ok(i) => i,
        Err(e) => {
            return Ok(warp::reply::with_status(
                "Internal encoding error".to_string(),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ));
        }
    };

    Ok(warp::reply::with_status::<String>(
        serialized_response,
        warp::http::StatusCode::OK,
    ))
}

/// Get account state in hex format
#[utoipa::path(
    get,
    path = "/account-hex",
    tag = "nexus",
    params(
        ("app_account_id" = String, Query, description = "Account ID in hex format")
    ),
    responses(
        (status = 200, description = "Account state found in hex format", body = String),
        (status = 404, description = "Account not found", body = String),
        (status = 400, description = "Invalid hash format", body = String),
        (status = 500, description = "Internal error", body = String)
    )
)]
async fn get_state_hex(
    db: Arc<Mutex<NodeDB>>,
    state: Arc<Mutex<VmState>>,
    app_account_id: &H256,
) -> Result<WithStatus<String>, Rejection> {
    let state_lock = state.lock().await;
    let db_lock = db.lock().await;

    let header_store: HeaderStore = match db_lock.get(b"previous_headers") {
        Ok(Some(i)) => i,
        Ok(None) => HeaderStore::new(32),
        Err(_) => {
            return Ok(warp::reply::with_status(
                "Header store error".to_string(),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    };
    let current_version = match state_lock.get_version(true) {
        Ok(Some(i)) => i,
        Ok(None) => 0,
        Err(_) => {
            return Ok(warp::reply::with_status(
                "Internal db error".to_string(),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    };

    let (account_option, proof) = match state_lock.get_with_proof(app_account_id, current_version) {
        Ok(i) => i,
        Err(_) => {
            return Ok(warp::reply::with_status(
                "Internal error".to_string(),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    };
    let root = match state_lock.get_root(current_version) {
        Ok(i) => i,
        Err(_) => {
            return Ok(warp::reply::with_status(
                "Internal error".to_string(),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    };

    let account = account_option.unwrap_or_else(AccountState::zero);
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
            None => {
                return Ok(warp::reply::with_status(
                    "No headers available.".to_string(),
                    warp::http::StatusCode::NOT_FOUND,
                ))
            }
        },
        proof_hex: siblings.iter().map(|s| hex::encode(s)).collect(),
        value_hash_hex: hex::encode(value_hash),
        nexus_state_root_hex: hex::encode(root.as_fixed_slice()),
    };

    let response = AccountWithProofHex::from(account_with_proof);

    let serialized_response = match serde_json::to_string(&response) {
        Ok(i) => i,
        Err(e) => {
            return Ok(warp::reply::with_status(
                "Internal encoding error".to_string(),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ));
        }
    };

    Ok(warp::reply::with_status::<String>(
        serialized_response,
        warp::http::StatusCode::OK,
    ))
}

/// Get header by Avail hash
#[utoipa::path(
    get,
    path = "/header",
    tag = "nexus",
    params(
        ("hash" = String, Query, description = "Avail block hash in hex format")
    ),
    responses(
        (status = 200, description = "Header found", body = NexusHeader),
        (status = 404, description = "Header not found", body = String),
        (status = 400, description = "Invalid hash format", body = String),
        (status = 500, description = "Internal error", body = String)
    )
)]
async fn get_header(
    db: Arc<Mutex<NodeDB>>,
    avail_hash: H256,
) -> Result<WithStatus<String>, Rejection> {
    let db_lock = db.lock().await;

    let nexus_hash: H256 = match db_lock.get::<AvailToNexusPointer>(avail_hash.as_slice()) {
        Ok(Some(i)) => i.nexus_hash,
        Ok(None) => {
            return Ok(warp::reply::with_status(
                "Avail header not yet processed.".to_string(),
                warp::http::StatusCode::NOT_FOUND,
            ))
        }
        Err(_) => {
            return Ok(warp::reply::with_status(
                "Node DB error. Cannot find mapping".to_string(),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    };

    let nexus_header: NexusHeader = match db_lock.get(nexus_hash.as_slice()) {
        Ok(Some(i)) => i,
        Ok(None) => {
            return Ok(warp::reply::with_status(
                "Internal error".to_string(),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
        Err(_) => {
            return Ok(warp::reply::with_status(
                "Node DB error. Cannot find nexus header".to_string(),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    };

    let serialized_response = match serde_json::to_string(&nexus_header) {
        Ok(i) => i,
        Err(_) => {
            return Ok(warp::reply::with_status(
                "Internal encoding error".to_string(),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ));
        }
    };

    Ok(warp::reply::with_status::<String>(
        serialized_response,
        warp::http::StatusCode::OK,
    ))
}

/// Get block range
#[utoipa::path(
    get,
    path = "/range",
    tag = "nexus",
    responses(
        (status = 200, description = "Block range retrieved successfully", body = String),
        (status = 500, description = "Internal error", body = String)
    )
)]
async fn range(db: Arc<Mutex<NodeDB>>) -> Result<WithStatus<String>, Rejection> {
    let db_lock = db.lock().await;

    let header_store: HeaderStore = match db_lock.get(b"previous_headers") {
        Ok(Some(i)) => i,
        Ok(None) => HeaderStore::new(32),
        Err(_) => {
            return Ok(warp::reply::with_status(
                "Header store error".to_string(),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    };

    let range: Vec<H256> = header_store.inner().iter().map(|h| h.hash()).collect();
    let serialized_range = match serde_json::to_string(&range) {
        Ok(i) => i,
        Err(e) => {
            return Ok(warp::reply::with_status(
                "Internal encoding error".to_string(),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ));
        }
    };

    Ok(warp::reply::with_status::<String>(
        serialized_range,
        warp::http::StatusCode::OK,
    ))
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
    let db_clone_5 = db.clone();

    let health_check = warp::path("health")
        .and(warp::get())
        .map(|| warp::reply::json(&serde_json::json!({"status": "Alive ser."})));

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
                            Err(_) => Ok(warp::reply::with_status(
                                "Invalid hash".to_string(),
                                warp::http::StatusCode::BAD_REQUEST,
                            )),
                        }
                    }
                    None => Ok(warp::reply::with_status(
                        "Hash parameter not provided".to_string(),
                        warp::http::StatusCode::BAD_REQUEST,
                    )),
                }
            },
        );

    let block = warp::path("block")
        .and(warp::get())
        .and(warp::any().map(move || db_clone_5.clone()))
        .and(warp::query::<HashMap<String, String>>())
        .and_then(
            |db: Arc<Mutex<NodeDB>>, params: HashMap<String, String>| async move {
                let block_hash = match params
                    .get("block_hash")
                    .map(|hash_str| H256::try_from(hash_str.as_str()))
                    .transpose()
                    .map_err(|_| {
                        warp::reply::with_status(
                            "Invalid hash".to_string(),
                            warp::http::StatusCode::BAD_REQUEST,
                        )
                    }) {
                    Ok(i) => i,
                    Err(e) => return Ok(e),
                };

                let block_number = match params
                    .get("block_number")
                    .map(|hash_str| hash_str.parse::<u32>())
                    .transpose()
                    .map_err(|_| {
                        warp::reply::with_status(
                            "Invalid block number".to_string(),
                            warp::http::StatusCode::BAD_REQUEST,
                        )
                    }) {
                    Ok(i) => i,
                    Err(e) => return Ok(e),
                };

                get_block(db, block_hash, block_number).await
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
                            Ok(hash) => get_header(db, hash).await,
                            Err(_) => Ok(warp::reply::with_status(
                                "Invalid hash".to_string(),
                                warp::http::StatusCode::BAD_REQUEST,
                            )),
                        }
                    }
                    None => Ok(warp::reply::with_status(
                        "Hash parameter not provided".to_string(),
                        warp::http::StatusCode::BAD_REQUEST,
                    )),
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
                                Err(_) => {
                                    return Ok(warp::reply::with_status(
                                        "Invalid hash".to_string(),
                                        warp::http::StatusCode::BAD_REQUEST,
                                    ))
                                }
                            },
                            None => None,
                        };
                        let app_account_id = H256::try_from(hash_str.as_str());
                        match app_account_id {
                            Ok(i) => get_state(db, vm_state, &i, block_hash).await,
                            Err(_) => Ok(warp::reply::with_status(
                                "Invalid hash".to_string(),
                                warp::http::StatusCode::BAD_REQUEST,
                            )),
                        }
                    }
                    None => Ok(warp::reply::with_status(
                        "Hash parameter not provided".to_string(),
                        warp::http::StatusCode::BAD_REQUEST,
                    )),
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
                            Err(_) => Ok(warp::reply::with_status(
                                "Invalid hash".to_string(),
                                warp::http::StatusCode::BAD_REQUEST,
                            )),
                        }
                    }
                    None => Ok(warp::reply::with_status(
                        "Hash parameter not provided".to_string(),
                        warp::http::StatusCode::BAD_REQUEST,
                    )),
                }
            },
        );

    let config = Arc::new(Config::from("/api-doc.json"));
    let api_doc = warp::path("api-doc.json")
        .and(warp::get())
        .map(|| warp::reply::json(&ApiDoc::openapi()));

    let swagger_ui = warp::path("swagger-ui")
        .and(warp::get())
        .and(warp::path::full())
        .and(warp::path::tail())
        .and(warp::any().map(move || config.clone()))
        .and_then(serve_swagger);

    tx.or(health_check)
        .or(tx_status)
        .or(block)
        .or(submit_batch)
        .or(header)
        .or(account)
        .or(account_hex)
        .or(api_doc)
        .or(swagger_ui)
}

async fn serve_swagger(
    full_path: FullPath,
    tail: Tail,
    config: Arc<Config<'static>>,
) -> Result<Box<dyn Reply + 'static>, Rejection> {
    if full_path.as_str() == "/swagger-ui" {
        return Ok(Box::new(warp::redirect::found(Uri::from_static(
            "/swagger-ui/",
        ))));
    }

    let path = tail.as_str();
    match utoipa_swagger_ui::serve(path, config) {
        Ok(file) => {
            if let Some(file) = file {
                Ok(Box::new(
                    Response::builder()
                        .header("Content-Type", file.content_type)
                        .body(file.bytes),
                ))
            } else {
                Ok(Box::new(StatusCode::NOT_FOUND))
            }
        }
        Err(error) => Ok(Box::new(
            Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(error.to_string()),
        )),
    }
}
