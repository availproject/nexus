use crate::utils::{add_check_body_and_response, get_mock_server, run_mock_geth_adapter_once, run_nexus_client};
use adapter_sdk::api::NexusAPI;
use nexus_core::types::{AppAccountId, AppId};
use std::time::Duration;
use tokio::time::sleep;

mod utils;

// As defined in the mock geth adapter
const NEXUS_PORT: u32 = 7000;

#[tokio::test]
async fn test_integration_mock_geth_adapter() {
    let eth_response = include_str!("./data/0_get_block_by_number.json");
    // Creating the mock server for eth client
    let mock_server = get_mock_server();
    let mock = add_check_body_and_response(
        &mock_server,
        r#"
                {
                    "method": "eth_getBlockByNumber"
                }
            "#,
        eth_response,
    );
    // run nexus
    let _nexus_client = run_nexus_client().await.expect("Failed to run nexus");
    // run mock geth adapter once
    let _geth_adapter = run_mock_geth_adapter_once(mock_server.base_url())
        .await
        .expect("Failed to run mock geth adapter");

    // Get initial state of the registered rollup
    let nexus_api = NexusAPI::new(&format!("http://0.0.0.0:{}", NEXUS_PORT));
    let app_id = AppId(100);
    let app_account_id = AppAccountId::from(app_id);
    let initial_account_state = nexus_api
        .get_account_state(&app_account_id.as_h256())
        .await
        .expect("Failed to get state from nexus");

    // Get final state of the registered rollup
    sleep(Duration::from_secs(10)).await;
    let final_account_state = nexus_api
        .get_account_state(&app_account_id.as_h256())
        .await
        .expect("Failed to get state from nexus");

    // Assert conditions
    assert_ne!(
        initial_account_state.nexus_state_root_hex, final_account_state.nexus_state_root_hex,
        "Nexus State Root hash must be different from the initial state root hash"
    );
    assert_ne!(
        initial_account_state.account.height, final_account_state.account.height,
        "Rollup height must be updated"
    );
    assert_ne!(
        initial_account_state.account.last_proof_height, final_account_state.account.last_proof_height,
        "Rollup last proof height must be updated"
    );

    mock.assert();
}
