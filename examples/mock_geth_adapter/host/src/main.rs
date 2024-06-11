use adapter_sdk::{api::NexusAPI, types::AdapterConfig};
use anyhow::{Context, Error};
use geth_methods::{ADAPTER_ELF, ADAPTER_ID};
use nexus_core::db::NodeDB;
use nexus_core::state::sparse_merkle_tree::traits::Value;
use nexus_core::types::{
    AccountState, AppAccountId, AppId, InitAccount, RollupPublicInputsV2, StatementDigest,
    TransactionV2, TxParamsV2, TxSignature, H256,
};
use risc0_zkvm::guest::env;
use serde::{Deserialize, Serialize};
use std::env::args;
use std::time::Duration;
use web3::transports::Http;
use web3::types::BlockId;
use web3::Web3;

// Your NodeDB struct and methods implementation here

#[derive(Clone, Serialize, Deserialize)]
struct AdapterStateData {
    last_height: u64,
    adapter_config: AdapterConfig,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Retrieve Ethereum node URL and --dev flag from command line arguments
    let args: Vec<String> = args().collect();

    if args.len() <= 2 {
        if args.len() == 2 && args[1] == "--dev" {
            eprintln!("Usage: cargo run -- <ethereum_node_url> [--dev]");
            return Ok(());
        }

        if args.len() < 2 {
            eprintln!("Usage: cargo run -- <ethereum_node_url> [--dev]");
            return Ok(());
        }
    }
    let ethereum_node_url = &args[1];
    let dev_flag = args.iter().any(|arg| arg == "--dev");
    let nexus_api = NexusAPI::new(&"http://127.0.0.1:7000");

    // Create or open the database
    let db_path = "db";
    let db = NodeDB::from_path(db_path);

    // If --dev flag is used, purge the database
    if dev_flag {
        db.delete(b"adapter_state_data")?;
    }

    // Retrieve or initialize the adapter state data from the database
    let adapter_state_data = if let Some(data) = db.get::<AdapterStateData>(b"adapter_state")? {
        data
    } else {
        // Initialize with default values if no data found in the database
        let adapter_config = AdapterConfig {
            app_id: AppId(100),
            elf: ADAPTER_ELF.to_vec(),
            adapter_elf_id: StatementDigest(ADAPTER_ID),
            vk: [0u8; 32],
            rollup_start_height: 606460,
        };
        AdapterStateData {
            last_height: 0,
            adapter_config,
        }
    };

    // Main loop to fetch headers and run adapter
    let mut last_height = adapter_state_data.last_height;

    let web3 = Web3::new(Http::new(ethereum_node_url).unwrap());
    loop {
        match web3
            .eth()
            .block(BlockId::Number(web3::types::BlockNumber::Latest))
            .await
        {
            Ok(Some(header)) => {
                let current_height = header.number.unwrap().as_u64();
                let range = match nexus_api.get_range().await {
                    Ok(i) => i,
                    Err(e) => {
                        println!("{:?}", e);
                        continue;
                    }
                };
                let app_account_id =
                    AppAccountId::from(adapter_state_data.adapter_config.app_id.clone());
                let account: AccountState =
                    match nexus_api.get_account_state(&app_account_id.as_h256()).await {
                        Ok(i) => i,
                        Err(e) => {
                            println!("{:?}", e);

                            continue;
                        }
                    };

                if range.is_empty() {
                    println!("Nexus does not have a valid range, retrying.");

                    continue;
                }

                if account == AccountState::zero() {
                    let tx = TransactionV2 {
                        signature: TxSignature([0u8; 64]),
                        params: TxParamsV2::InitAccount(InitAccount {
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
                            println!("Error when iniating account: {:?}", e);

                            continue;
                        }
                    }
                }

                let height: u32 = header.number.unwrap().as_u32();

                if current_height > last_height {
                    let public_inputs = RollupPublicInputsV2 {
                        nexus_hash: range[0],
                        state_root: H256::from(header.state_root.as_fixed_bytes().clone()),
                        //TODO: remove unwrap
                        height,
                        start_nexus_hash: H256::from(account.start_nexus_hash),
                        app_id: app_account_id,
                        img_id: account.statement,
                    };
                    // Pass the state root to adapter.run()
                    env::commit(&public_inputs);

                    last_height = current_height;
                }
            }
            Ok(None) => {
                println!("")
            }
            Err(err) => {
                println!("Error fetching latest header: {:?}", err);
            }
        }

        tokio::time::sleep(Duration::from_secs(2)).await;
        // Persist adapter state data to the database
        db.put(
            b"adapter_state_data",
            &AdapterStateData {
                last_height,
                adapter_config: adapter_state_data.adapter_config.clone(),
            },
        )?;
    }
}
