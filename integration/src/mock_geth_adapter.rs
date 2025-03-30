use crate::utils::{add_check_body_and_response, get_mock_server, run_mock_geth_adapter_once, run_nexus_client};
use adapter_sdk::api::NexusAPI;
use anyhow::anyhow;
use avail_rust::rpc;
use avail_rust::SDK;
use geth_methods::ADAPTER_ID;
use nexus_core::types::{AccountWithProof, AppAccountId, AppId, InitAccount, Sha256, StatementDigest, Transaction, TxParams, TxSignature, H256};
use nexus_core::utils::hasher::Digest;
use std::env;
use std::time::Duration;
use tokio::time::sleep;

// As defined in the mock geth adapter
const NEXUS_PORT: u32 = 7002;

#[tokio::test]
async fn test_integration_mock_geth_adapter() {
    let args: Vec<String> = env::args().collect();
    let avail_rpc_url = args
        .clone()
        .iter()
        .find(|arg| arg.starts_with("--avail-rpc="))
        .map(|arg| arg.trim_start_matches("--avail-rpc="))
        .unwrap_or("wss://zero-devnet.avail.so:443/ws")
        .to_string();
    let eth_response = include_str!("../../integration/data/0_get_block_by_number.json");
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
    let sdk = SDK::new(&avail_rpc_url).await.unwrap();

    let finalized_header_hash = rpc::chain::get_finalized_head(&sdk.client)
        .await
        .expect("RPC call should not have failed");

    let finalized_header = rpc::chain::get_header(&sdk.client, Some(finalized_header_hash))
        .await
        .expect("RPC call should not have failed");

    // run nexus
    let _nexus_client = run_nexus_client(&avail_rpc_url, finalized_header.number, NEXUS_PORT)
        .await
        .expect("Failed to run nexus");

    // Check If we are able to fetch state from nexus node
    let app_id = AppId(100);
    let app_account_id = AppAccountId::from(app_id);
    let nexus_api = NexusAPI::new(&format!("http://0.0.0.0:{}", NEXUS_PORT));
    let _ = get_account_state(&nexus_api, app_account_id.clone(), 6)
        .await
        .expect("Failed to get account state");

    // Register account here itself (so that we don't get same height)
    register_account(&nexus_api, app_account_id.clone()).await;

    // Wait for 120 secs as a buffer time
    sleep(Duration::from_secs(120)).await;

    // Get initial state of the registered rollup
    let initial_account_state = get_account_state(&nexus_api, app_account_id.clone(), 2).await.unwrap();

    // run mock geth adapter once
    let _geth_adapter = run_mock_geth_adapter_once(
        &mock_server.base_url(),
        &format!("http://0.0.0.0:{}", NEXUS_PORT),
    )
    .await
    .expect("Failed to run mock geth adapter");

    // Wait for 120 secs as a buffer time
    sleep(Duration::from_secs(120)).await;

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

    // Asserting the hash of account state with nexus state root :

    // This value is taken from the JellyfishMerkleTree
    let hash_hex = "4a4d543a3a4c6561664e6f6465";
    // app_id is calculated using the following steps :
    // sha256(u32 AppId)
    let app_id = "3655ca59b7d566ae06297c200f98d04da2e8e89812d627bc29297c25db60362d";
    let value_hash = final_account_state.value_hash_hex;

    let mut hasher = Sha256::new();
    hasher.update(hex_to_vec(hash_hex));
    hasher.update(hex_to_vec(app_id));
    hasher.update(hex_to_vec(&value_hash));
    let calculated_hash = hasher.finalize().to_vec();

    assert_eq!(
        final_account_state.nexus_state_root_hex,
        hex::encode(calculated_hash)
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
            statement: StatementDigest(ADAPTER_ID),
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

fn hex_to_vec(hex: &str) -> Vec<u8> {
    let hex = hex.trim_start_matches("0x");
    hex.as_bytes()
        .chunks(2)
        .map(|chunk| u8::from_str_radix(std::str::from_utf8(chunk).unwrap(), 16).unwrap())
        .collect()
}
