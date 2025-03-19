use crate::utils::{add_check_body_and_response, get_mock_server, run_mock_geth_adapter_once, run_nexus_client};
use adapter_sdk::api::NexusAPI;
use anyhow::anyhow;
use nexus_core::types::{AccountWithProof, AppAccountId, AppId, InitAccount, StatementDigest, Transaction, TxParams, TxSignature};
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

    // Check If we are able to fetch state from nexus node
    let app_id = AppId(100);
    let app_account_id = AppAccountId::from(app_id);
    let nexus_api = NexusAPI::new(&format!("http://0.0.0.0:{}", NEXUS_PORT));
    let _ = get_account_state(&nexus_api, app_account_id.clone(), 6)
        .await
        .expect("Failed to get account state");

    // Register account here itself (so that we don't get same height)
    register_account(&nexus_api, app_account_id.clone()).await;

    // Wait for 60 secs as a buffer time
    sleep(Duration::from_secs(60)).await;

    // Get initial state of the registered rollup
    let initial_account_state = get_account_state(&nexus_api, app_account_id.clone(), 2).await.unwrap();

    // run mock geth adapter once
    let _geth_adapter = run_mock_geth_adapter_once(mock_server.base_url())
        .await
        .expect("Failed to run mock geth adapter");

    // Wait for 60 secs as a buffer time
    sleep(Duration::from_secs(60)).await;

    let final_account_state = get_account_state(&nexus_api, app_account_id, 2).await.unwrap();

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

/// To get account state with certain number of retries as it takes time to finalize the block by avail.
async fn get_account_state(nexus_api: &NexusAPI, account_id: AppAccountId, mut tries: u32) -> anyhow::Result<AccountWithProof> {
    while tries != 0 {
        match nexus_api.get_account_state(&account_id.as_h256()).await {
            Ok(account_state) => return Ok(account_state),
            Err(e) => {
                tries -= 1;
                println!("Failed to get account state: {:?}, Retrying", e);
                sleep(Duration::from_secs(10)).await;
                continue;
            }
        }
    }
    Err(anyhow!("Account not found"))
}

/// To register the account before running the mock geth adapter
async fn register_account(nexus_api: &NexusAPI, app_account_id: AppAccountId) {
    let adapter_id: [u32; 8] = [
        629658695, 3066157471, 272264335, 1532806124, 2155508261, 1771388806, 3334681027, 2347946843,
    ];

    let range = match nexus_api.get_range().await {
        Ok(i) => i,
        Err(e) => {
            panic!("{:?}", e);
        }
    };

    let tx = Transaction {
        signature: TxSignature([0u8; 64]),
        params: TxParams::InitAccount(InitAccount {
            app_id: app_account_id.clone(),
            statement: StatementDigest(adapter_id),
            start_nexus_hash: range[0],
        }),
    };
    match nexus_api.send_tx(tx).await {
        Ok(i) => {
            println!(
                "Initiated account on nexus. AppAccountId: {:?} Response: {:?}",
                &app_account_id, i,
            )
        }
        Err(e) => {
            panic!("Error when iniating account: {:?}", e);
        }
    }
}
