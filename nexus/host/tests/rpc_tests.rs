use host::{setup_components, AvailToNexusPointer};
use nexus_core::{
    db::NodeDB,
    mempool::Mempool,
    state::VmState,
    types::{AccountState, HeaderStore, NexusHeader, StatementDigest, H256},
};
use serde_json::Value;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;
use warp::test::request;

async fn setup_test_environment(
    test_name: &str,
) -> (Arc<Mutex<NodeDB>>, Arc<Mutex<VmState>>, Mempool) {
    let db_path = format!("./tests/db/rpc_tests_{}", test_name);
    let (node_db, state) = setup_components(&db_path);
    let mempool = Mempool::new();
    (node_db, state, mempool)
}

#[tokio::test]
async fn test_range_endpoint() {
    let (node_db, vm_state, mempool) = setup_test_environment("range").await;

    // Create test data
    let db = node_db.lock().await;
    let header_store = HeaderStore::new(32);
    db.put(b"previous_headers", &header_store).unwrap();
    drop(db);

    // Test the range endpoint
    let api = host::rpc::routes(mempool, node_db.clone(), vm_state);
    let response = request().method("GET").path("/range").reply(&api).await;

    assert_eq!(response.status(), 200);

    // Parse and verify response
    let response_str = String::from_utf8(response.body().to_vec()).unwrap();
    let json: Value = serde_json::from_str(&response_str).unwrap();

    // The response should be a JSON array of hashes
    assert!(json.is_array());
}

#[tokio::test]
async fn test_header_endpoint() {
    let (node_db, vm_state, mempool) = setup_test_environment("header").await;

    // Create test header data
    let test_hash = H256::zero();
    let nexus_hash = H256::zero();
    let test_header = NexusHeader {
        parent_hash: H256::zero(),
        prev_state_root: H256::zero(),
        state_root: H256::zero(),
        avail_header_hash: test_hash,
        number: 1,
    };

    // Store test header and mapping
    {
        let mut db = node_db.lock().await;

        // Store the header
        db.put(nexus_hash.as_slice(), &test_header).unwrap();

        // Store the mapping from avail hash to nexus hash
        let pointer = AvailToNexusPointer {
            nexus_hash,
            number: 1,
        };
        db.put(test_hash.as_slice(), &pointer).unwrap();

        // Also store in previous_headers
        let mut header_store = HeaderStore::new(32);
        header_store.inner.push(test_header.clone());
        db.put(b"previous_headers", &header_store).unwrap();
    }

    // Test the header endpoint
    let api = host::rpc::routes(mempool, node_db.clone(), vm_state);
    let response = request()
        .method("GET")
        .path(&format!(
            "/header?hash={}",
            hex::encode(test_hash.as_slice())
        ))
        .reply(&api)
        .await;

    assert_eq!(response.status(), 200);

    // Parse and verify response
    let response_str = String::from_utf8(response.body().to_vec()).unwrap();
    let header: NexusHeader = serde_json::from_str(&response_str).unwrap();

    // Verify header fields
    assert_eq!(header.number, 1);
    assert_eq!(header.avail_header_hash, test_hash);
    assert_eq!(header.parent_hash, H256::zero());
}

#[tokio::test]
async fn test_account_endpoint() {
    let (node_db, vm_state, mempool) = setup_test_environment("account").await;

    // Create test account data
    let test_account_id = H256::zero();
    let test_account = AccountState {
        statement: StatementDigest([1u32; 8]),
        state_root: H256::zero().into(),
        start_nexus_hash: H256::zero().into(),
        last_proof_height: 1,
        height: 1,
    };

    // Store test account state
    let mut state = vm_state.lock().await;
    let mut account_map = HashMap::new();
    account_map.insert(test_account_id, Some(test_account.clone()));

    // Update state with version 1
    let (batch, _) = state.update_set(account_map, 1).unwrap();
    state.commit(&batch.node_batch).unwrap();
    state.update_version(1).unwrap();
    drop(state);

    // Test the account endpoint
    let api = host::rpc::routes(mempool, node_db.clone(), vm_state);
    let response = request()
        .method("GET")
        .path(&format!(
            "/account?app_account_id={}",
            hex::encode(test_account_id.as_slice())
        ))
        .reply(&api)
        .await;

    assert_eq!(response.status(), 200);

    // Parse and verify response
    let response_str = String::from_utf8(response.body().to_vec()).unwrap();
    let json: Value = serde_json::from_str(&response_str).unwrap();

    // Verify account response structure
    assert!(json.is_object());
    assert!(json.get("account").is_some());
    assert!(json.get("proof").is_some());
}

#[tokio::test]
async fn test_invalid_header_hash() {
    let (node_db, vm_state, mempool) = setup_test_environment("invalid_header").await;

    // Test with invalid hash
    let api = host::rpc::routes(mempool, node_db.clone(), vm_state);
    let response = request()
        .method("GET")
        .path("/header?hash=invalid_hash")
        .reply(&api)
        .await;

    assert_eq!(response.status(), 200); // Currently returns 200 with error message
    assert_eq!(
        String::from_utf8(response.body().to_vec()).unwrap(),
        "Invalid hash"
    );
}

#[tokio::test]
async fn test_invalid_account_id() {
    let (node_db, vm_state, mempool) = setup_test_environment("invalid_account").await;

    // Test with invalid account ID
    let api = host::rpc::routes(mempool, node_db.clone(), vm_state);
    let response = request()
        .method("GET")
        .path("/account?app_account_id=invalid_id")
        .reply(&api)
        .await;

    assert_eq!(response.status(), 200); // Currently returns 200 with error message
    assert_eq!(
        String::from_utf8(response.body().to_vec()).unwrap(),
        "Invalid hash"
    );
}

#[tokio::test]
async fn test_missing_parameters() {
    let (node_db, vm_state, mempool) = setup_test_environment("missing_params").await;
    let api = host::rpc::routes(mempool, node_db.clone(), vm_state);

    // Test header endpoint without hash parameter
    let response = request().method("GET").path("/header").reply(&api).await;

    assert_eq!(response.status(), 200);
    assert_eq!(
        String::from_utf8(response.body().to_vec()).unwrap(),
        "Hash parameter not provided"
    );

    // Test account endpoint without app_account_id parameter
    let response = request().method("GET").path("/account").reply(&api).await;

    assert_eq!(response.status(), 200);
    assert_eq!(
        String::from_utf8(response.body().to_vec()).unwrap(),
        "Hash parameter not provided"
    );
}
