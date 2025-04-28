use adapter_sdk::{api::NexusAPI, types::AdapterConfig};
use anyhow::Error;
use geth_methods::{ADAPTER_ELF, ADAPTER_ID};
use nexus_core::db::NodeDB;
use nexus_core::traits::NexusTransaction;
use nexus_core::types::{
    AccountState, AccountWithProof, AppAccountId, AppId, InitAccount, NexusRollupPI, Proof, StatementDigest, SubmitProof, Transaction,
    TransactionStatus, TxParams, TxSignature, H256,
};
use nexus_core::zkvm::risczero::RiscZeroProof;
use nexus_core::zkvm::ProverMode;
use risc0_zkvm::serde::to_vec;
use risc0_zkvm::{default_prover, ExecutorEnv};
use serde::{Deserialize, Serialize};
use std::env::args;
use std::time::Duration;
use web3::transports::Http;
use web3::types::BlockId;
use web3::Web3;
// Your NodeDB struct and methods implementation here

#[derive(Clone, Serialize, Deserialize)]
struct AdapterStateData {
    last_height: u32,
    adapter_config: AdapterConfig,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Retrieve Ethereum node URL and --dev flag from command line arguments
    let args: Vec<String> = args().collect();

    if args.len() <= 3 {
        if args.len() == 2 && args[1] == "--dev" {
            eprintln!("Usage: cargo run -- <ethereum_node_url> <nexus_api_url> [--dev] [--poll]");
            return Ok(());
        }

        if args.len() < 3 {
            eprintln!("Usage: cargo run -- <ethereum_node_url> <nexus_api_url> [--dev] [--poll]");
            return Ok(());
        }
    }
    let ethereum_node_url = &args[1];
    let nexus_api_url = &args[2];
    let dev_flag = args.iter().any(|arg| arg == "--dev");
    let poll_flag = args.iter().any(|arg| arg == "--poll");
    let prover_mode = if dev_flag { ProverMode::MockProof } else { ProverMode::Compressed };
    let nexus_api = NexusAPI::new(nexus_api_url);

    // Create or open the database
    let db_path = "db/mock_geth_adapter";
    let db = NodeDB::from_path(db_path);

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
            prover_mode,
            avail_url: String::from("wss://turing-rpc.avail.so:443/ws"),
        };
        AdapterStateData { last_height: 0, adapter_config }
    };

    let app_account_id = AppAccountId::from(adapter_state_data.adapter_config.app_id.clone());
    let account_with_proof: AccountWithProof = nexus_api.get_account_state(&app_account_id.as_h256()).await?;

    // Main loop to fetch headers and run adapter
    let mut last_height = adapter_state_data.last_height;
    let mut start_nexus_hash = None;

    let web3 = Web3::new(Http::new(ethereum_node_url).unwrap());
    loop {
        match web3.eth().block(BlockId::Number(web3::types::BlockNumber::Latest)).await {
            Ok(Some(header)) => {
                println!(
                    ">>> Got header: {:?} last height {}",
                    header, account_with_proof.account.height
                );
                let current_height = header.number.unwrap().as_u32();
                let range = match nexus_api.get_range().await {
                    Ok(i) => i,
                    Err(e) => {
                        println!("{:?}", e);
                        if !poll_flag {
                            return Err(anyhow::anyhow!("Failed to get range from Nexus API"));
                        }
                        continue;
                    }
                };

                last_height = account_with_proof.account.height;

                if range.is_empty() {
                    println!("Nexus does not have a valid range, retrying.");
                    if !poll_flag {
                        return Err(anyhow::anyhow!("Nexus does not have a valid range"));
                    }
                    continue;
                }

                if account_with_proof.account == AccountState::zero() {
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
                            start_nexus_hash = Some(range[0]);
                            println!(
                                "Initiated account on nexus. AppAccountId: {:?} Response: {:?}",
                                &app_account_id, i,
                            )
                        }
                        Err(e) => {
                            println!("Error when iniating account: {:?}", e);
                            if !poll_flag {
                                return Err(anyhow::anyhow!("Failed to initiate account: {:?}", e));
                            }
                            continue;
                        }
                    }
                }

                let height: u32 = header.number.unwrap().as_u32();

                if current_height > last_height {
                    // let app_account_id = AppAccountId::from(adapter_state_data.adapter_config.app_id.clone());
                    // let account_with_proof: AccountWithProof = match nexus_api.get_account_state(&app_account_id.as_h256()).await {
                    //     Ok(i) => i,
                    //     Err(e) => {
                    //         println!("{:?}", e);
                    //
                    //         continue;
                    //     }
                    // };
                    // println!("\n\n\n>>> account with proof : {:?}", account_with_proof);
                    // println!(">>> statement digest : {:?}", hex::encode(account_with_proof.account.statement.0.encode()));
                    // println!(
                    //     ">>> state root : {:?}",
                    //     hex::encode(account_with_proof.account.state_root)
                    // );
                    // println!(
                    //     ">>> start nexus hash : {:?}",
                    //     hex::encode(account_with_proof.account.start_nexus_hash)
                    // );
                    // println!(
                    //     ">>> last proof height : {:?}",
                    //     account_with_proof.account.last_proof_height
                    // );
                    // println!(">>> height : {:?}", account_with_proof.account.height);
                    // println!(">>> key : {:?}", hex::encode(app_account_id.0));

                    let public_inputs = NexusRollupPI {
                        nexus_hash: range[0],
                        state_root: H256::from(header.state_root.as_fixed_bytes().clone()),
                        //TODO: remove unwrap
                        height,
                        start_nexus_hash: start_nexus_hash.unwrap_or_else(|| H256::from(account_with_proof.account.start_nexus_hash)),
                        app_id: app_account_id.clone(),
                        img_id: StatementDigest(ADAPTER_ID),
                        rollup_hash: Some(H256::from([1u8; 32])),
                    };

                    // println!(">>> public inputs: {:?}", public_inputs);

                    let public_input_vec = match to_vec(&public_inputs) {
                        Ok(i) => i,
                        Err(e) => {
                            let err = anyhow::anyhow!("Could not encode public inputs of rollup.");
                            if !poll_flag {
                                return Err(err);
                            }
                            println!("{:?}", err);
                            continue;
                        }
                    };

                    let mut env_builder = ExecutorEnv::builder();
                    let env = env_builder.write(&public_inputs).unwrap().build().unwrap();
                    let prover = default_prover();
                    let prove_info = match prover.prove(env, ADAPTER_ELF) {
                        Ok(i) => i,
                        Err(e) => {
                            println!("Unable to generate proof due to error: {:?}", e);
                            if !poll_flag {
                                return Err(anyhow::anyhow!("Failed to generate proof: {:?}", e));
                            }
                            continue;
                        }
                    };

                    // println!(">>> public inputs : {:?}", hex::encode(prove_info.receipt.clone().journal.bytes));
                    // println!(">>> proof : {:?}", hex::encode(encode_seal(&prove_info.receipt).unwrap()));
                    // let decoded_journal: NexusRollupPI = prove_info.receipt.journal.decode()?;
                    // println!(">>> journal : {:?}", decoded_journal);
                    let recursive_proof = RiscZeroProof(prove_info.receipt);

                    let tx = Transaction {
                        signature: TxSignature([0u8; 64]),
                        params: TxParams::SubmitProof(SubmitProof {
                            app_id: app_account_id.clone(),
                            nexus_hash: range[0],
                            state_root: public_inputs.state_root.clone(),
                            proof: match recursive_proof.clone().try_into() {
                                Ok(i) => i,
                                Err(e) => {
                                    println!("Unable to serialise proof: {:?}", e);
                                    if !poll_flag {
                                        return Err(anyhow::anyhow!("Failed to serialize proof: {:?}", e));
                                    }
                                    continue;
                                }
                            },
                            height: public_inputs.height,
                            data: Some(H256::from([1u8; 32])),
                        }),
                    };

                    match nexus_api.send_tx(tx.clone()).await {
                        Ok(i) => {
                            println!(
                                "Submitted proof to update state root on nexus. AppAccountId: {:?} Response: {:?} Stateroot: {:?}",
                                &app_account_id, i, &public_inputs.state_root
                            );

                            tokio::time::sleep(Duration::from_secs(5)).await;

                            // Check txn status :
                            let txn_hash = tx.hash();
                            let txn_status = nexus_api.get_transaction_status(&txn_hash).await?;

                            //TODO: Below check needs to be done after 60 seconds.
                            // Assert txn status to be successful
                            if txn_status.status != TransactionStatus::InPool {
                                if !poll_flag {
                                    return Err(anyhow::anyhow!("Failed to submit proof"));
                                }

                                continue;
                            }
                        }
                        Err(e) => {
                            println!("Error when iniating account: {:?}", e);
                            if !poll_flag {
                                return Err(anyhow::anyhow!("Failed to submit proof: {:?}", e));
                            }
                            continue;
                        }
                    }

                    last_height = current_height;
                } else {
                    println!(
                        "Current height is lesser than height on nexus. current height: {} nexus height: {}",
                        current_height, last_height
                    );
                }
            }
            Ok(None) => {
                println!("Got no header.");
                if !poll_flag {
                    return Err(anyhow::anyhow!("Got no header from Ethereum node"));
                }
            }
            Err(err) => {
                println!("Error fetching latest header: {:?}", err);
                if !poll_flag {
                    return Err(anyhow::anyhow!("Error fetching latest header: {:?}", err));
                }
            }
        }

        // Persist adapter state data to the database
        db.put(
            b"adapter_state_data",
            &AdapterStateData {
                last_height,
                adapter_config: adapter_state_data.adapter_config.clone(),
            },
        )?;

        println!("Sleeping for 10 seconds\n\n\n");
        tokio::time::sleep(Duration::from_secs(10)).await;

        // let app_account_id = AppAccountId::from(adapter_state_data.adapter_config.app_id.clone());
        // let account_with_proof: AccountWithProof = match nexus_api.get_account_state(&app_account_id.as_h256()).await {
        //     Ok(i) => i,
        //     Err(e) => {
        //         println!("{:?}", e);
        //
        //         continue;
        //     }
        // };
        //
        // println!("\n\n\n>>> account with proof : {:?}", account_with_proof);
        // // println!(">>> statement digest : {:?}", hex::encode(account_with_proof.account.statement.0.encode()));
        // println!(
        //     ">>> state root : {:?}",
        //     hex::encode(account_with_proof.account.state_root)
        // );
        // println!(
        //     ">>> start nexus hash : {:?}",
        //     hex::encode(account_with_proof.account.start_nexus_hash)
        // );
        // println!(
        //     ">>> last proof height : {:?}",
        //     account_with_proof.account.last_proof_height
        // );
        // println!(">>> height : {:?}", account_with_proof.account.height);
        // println!(">>> key : {:?}", hex::encode(app_account_id.0));

        // Exit if not in polling mode
        if !poll_flag {
            println!("Exiting after one execution (--poll flag not provided)");
            break;
        }
    }

    Ok(())
}

// To run :
// With polling: RUST_LOG=debug RISC0_DEV_MODE=1 cargo run -- <ethereum_url> <nexus_api_url> --dev --poll
// Without polling: RUST_LOG=debug RISC0_DEV_MODE=1 cargo run -- <ethereum_url> <nexus_api_url> --dev
