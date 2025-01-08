use adapter_sdk::{api::NexusAPI, types::AdapterConfig};
use anyhow::{anyhow, Context, Error};
use nexus_core::db::NodeDB;
use nexus_core::types::{
    AccountState, AccountWithProof, AppAccountId, AppId, InitAccount, NexusRollupPI,
    StatementDigest, SubmitProof, Transaction, TxParams, TxSignature, H256,
};

#[cfg(feature = "risc0")]
use nexus_core::zkvm::risczero::{RiscZeroProof as Proof, RiscZeroProver as Prover, ZKVM};

#[cfg(feature = "sp1")]
use nexus_core::zkvm::sp1::{Sp1Proof as Proof, Sp1Prover as Prover, SP1ZKVM as ZKVM};

use nexus_core::zkvm::traits::{ZKVMProof, ZKVMProver};
use nexus_core::zkvm::ProverMode;
use proof_api::ProofAPIResponse;
#[cfg(feature = "risc0")]
use risc0_zkvm::guest::env;
#[cfg(feature = "risc0")]
use risc0_zkvm::serde::to_vec;
#[cfg(feature = "risc0")]
use risc0_zkvm::{default_prover, ExecutorEnv};
use serde::{Deserialize, Serialize};
use serde_json::{self, Value as SerdeValue};
use std::collections::HashMap;
use std::env::args;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::fs;
use zksync_core::{L1BatchWithMetadata, ProofWithCommitmentAndL1BatchMetaData, STF};

#[cfg(any(feature = "sp1"))]
use sp1_sdk::utils;

#[cfg(feature = "risc0")] // for now
use zksync_methods::{ZKSYNC_ADAPTER_ELF, ZKSYNC_ADAPTER_ID};

mod proof_api;
// Your NodeDB struct and methods implementation here

#[derive(Clone, Serialize, Deserialize, Debug)]
struct AdapterStateData {
    last_height: u32,
    adapter_config: AdapterConfig,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    #[cfg(any(feature = "sp1"))]
    utils::setup_logger();

    // Retrieve Ethereum node URL and command-line arguments
    let args: Vec<String> = args().collect();
    if args.len() <= 2 {
        if args.len() == 2 && args[1] == "--dev" {
            eprintln!("Usage: cargo run -- <zksync_proof_api_url> [--dev] [--app_id <value>]");
            return Ok(());
        }

        if args.len() < 2 {
            eprintln!("Usage: cargo run -- <zksync_proof_api_url> [--dev] [--app_id <value>]");
            return Ok(());
        }
    }

    let zksync_proof_api_url = &args[1];
    let dev_flag = args.iter().any(|arg| arg == "--dev");
    let prover_mode = if dev_flag {
        ProverMode::MockProof
    } else {
        ProverMode::Compressed
    };

    // Default app_id
    let mut app_id = 100;

    // Parse the --app_id argument if provided
    if let Some(app_id_index) = args.iter().position(|arg| arg == "--app_id") {
        if let Some(app_id_value) = args.get(app_id_index + 1) {
            match app_id_value.parse::<u32>() {
                Ok(id) => app_id = id,
                Err(_) => {
                    eprintln!("Invalid app_id value. Please provide a valid number.");
                    return Ok(());
                }
            }
        } else {
            eprintln!("Usage: cargo run -- <zksync_proof_api_url> [--dev] [--app_id <value>]");
            return Ok(());
        }
    }

    let nexus_api = NexusAPI::new(&"http://dev.nexus.avail.tools");

    // Create or open the database
    let db_path = format!("db/{:?}", app_id);
    let db = NodeDB::from_path(&db_path);

    #[cfg(feature = "sp1")]
    let ZKSYNC_ADAPTER_ELF: &[u8] =
        include_bytes!("../../methods/sp1-guest/elf/riscv32im-succinct-zkvm-elf");

    #[cfg(feature = "sp1")]
    let ZKSYNC_ADAPTER_ID = Prover::new(ZKSYNC_ADAPTER_ELF.to_vec(), prover_mode.clone()).vk(); // since sp1 doesn't implements verify method on proof object

    // Retrieve or initialize the adapter state data from the database
    let adapter_state_data =
        if let Some(data) = db.get::<AdapterStateData>(b"adapter_state_data")? {
            data
        } else {
            // Initialize with default values if no data found in the database
            let adapter_config = AdapterConfig {
                app_id: AppId(app_id),
                elf: ZKSYNC_ADAPTER_ELF.to_vec(),
                adapter_elf_id: StatementDigest(ZKSYNC_ADAPTER_ID),
                vk: [0u8; 32],
                rollup_start_height: 606460,
                prover_mode: prover_mode.clone(),
                avail_url: String::from("wss://turing-rpc.avail.so:443/ws"),
            };
            AdapterStateData {
                last_height: 0,
                adapter_config,
            }
        };

    // Main loop to fetch headers and run adapter
    let mut last_height = adapter_state_data.last_height;
    let mut start_nexus_hash: Option<H256> = None;
    let stf = STF::new(
        ZKSYNC_ADAPTER_ID,
        ZKSYNC_ADAPTER_ELF.to_vec(),
        prover_mode.clone(),
    );

    println!(
        "Starting nexus with AppAccountId: {:?} \n, and start height {last_height}",
        AppAccountId::from(adapter_state_data.adapter_config.app_id.clone()),
    );

    let proof_api = proof_api::ProofAPI::new(zksync_proof_api_url);

    let app_account_id = AppAccountId::from(adapter_state_data.adapter_config.app_id.clone());
    let account_with_proof: AccountWithProof = nexus_api
        .get_account_state(&app_account_id.as_h256())
        .await?;
    let height_on_nexus = account_with_proof.account.height;

    // if adapter_state_data.adapter_config.adapter_elf_id.clone()
    //     != account_with_proof.account.statement.clone()
    // {
    //     if account_with_proof.account != AccountState::zero() {
    //         println!(
    //             "❌ ❌ ❌, statement digest not matching \n{:?} \n== \n{:?}",
    //             &adapter_state_data.adapter_config.adapter_elf_id,
    //             &account_with_proof.account.statement
    //         );
    //     }
    // }

    //Commenting below, as last height should be last height known to adapter, and should create proofs from that point.
    //last_height = account_with_proof.account.height;

    if account_with_proof.account == AccountState::zero() {
        let tx = Transaction {
            signature: TxSignature([0u8; 64]),
            params: TxParams::InitAccount(InitAccount {
                app_id: app_account_id.clone(),
                statement: StatementDigest(ZKSYNC_ADAPTER_ID),
                start_nexus_hash: account_with_proof.nexus_header.hash(),
            }),
        };

        // fs::write("./init_tx.json", serde_json::to_string(&tx).unwrap()).await;

        nexus_api.send_tx(tx).await?;

        println!("Waiting for 10 seconds for account to be initiated");
        tokio::time::sleep(Duration::from_secs(10)).await;
    }

    loop {
        println!("Processing L1 batch number: {}", last_height + 1);

        match proof_api.get_proof_for_l1_batch(last_height + 1).await {
            Ok(ProofAPIResponse::Found((proof_with_commitment_and_l1_batch_meta_data, proof))) => {
                let ProofWithCommitmentAndL1BatchMetaData {
                    proof_with_l1_batch_metadata,
                    blob_commitments,
                    pubdata_commitments,
                    versioned_hashes,
                } = proof_with_commitment_and_l1_batch_meta_data;
                let batch_metadata = proof_with_l1_batch_metadata.metadata;
                let current_height = batch_metadata.header.number.0;
                // println!("metadata: {:?}", batch_metadata);

                let app_account_id =
                    AppAccountId::from(adapter_state_data.adapter_config.app_id.clone());
                let account_with_proof: AccountWithProof =
                    match nexus_api.get_account_state(&app_account_id.as_h256()).await {
                        Ok(i) => i,
                        Err(e) => {
                            println!("{:?}", e);

                            continue;
                        }
                    };
                let height_on_nexus = account_with_proof.account.height;

                // if adapter_state_data.adapter_config.adapter_elf_id.clone()
                //     != account_with_proof.account.statement.clone()
                // {
                //     if account_with_proof.account != AccountState::zero() {
                //         println!(
                //             "❌ ❌ ❌, statement digest not matching \n{:?} \n== \n{:?}",
                //             &adapter_state_data.adapter_config.adapter_elf_id,
                //             &account_with_proof.account.statement
                //         );
                //     }
                // }

                //Commenting below, as last height should be last height known to adapter, and should create proofs from that point.
                //last_height = account_with_proof.account.height;

                if account_with_proof.account == AccountState::zero() {
                    println!("Account state is not initiated, restart may be required");
                    tokio::time::sleep(Duration::from_secs(2)).await;

                    continue;
                }

                let (prev_proof_with_pi, account_state): (
                    Option<Proof>,
                    Option<(AppAccountId, AccountState)>,
                ) = if last_height == 0 {
                    (
                        None,
                        //TODO: Remove this clone of app account id.
                        Some((app_account_id.clone(), account_with_proof.account.clone())),
                    )
                } else {
                    match db.get(&last_height.to_be_bytes())? {
                        Some(i) => (Some(i), None),
                        None => {
                            return Err(anyhow!("previous proof and metadata not found for last height as per adapter state"))
                        }
                    }
                };
                let range = match nexus_api.get_range().await {
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

                let mut recursive_proof = stf.create_recursive_proof::<Prover, Proof, ZKVM>(
                    prev_proof_with_pi,
                    account_state,
                    proof,
                    batch_metadata.clone(),
                    pubdata_commitments,
                    versioned_hashes,
                    range[0],
                )?;

                println!(
                    "Current proof data: {:?}",
                    recursive_proof
                        .clone()
                        .public_inputs::<NexusRollupPI>()
                        .unwrap()
                        .rollup_hash
                        .unwrap()
                );

                let rollup_hash = recursive_proof
                    .clone()
                    .public_inputs::<NexusRollupPI>()
                    .unwrap()
                    .rollup_hash
                    .unwrap();

                #[cfg(feature = "risc0")]
                match recursive_proof.0.verify(ZKSYNC_ADAPTER_ID) {
                    Ok(()) => {
                        println!("Proof verification successful");
                        ()
                    }
                    Err(e) => return Err(anyhow!("Proof generated is invalid.")),
                }

                #[cfg(feature = "sp1")]
                match recursive_proof.verify(
                    None,
                    Some(ZKSYNC_ADAPTER_ELF.to_vec()),
                    prover_mode.clone(),
                ) {
                    Ok(()) => {
                        println!("Proof verification successful");
                        ()
                    }
                    Err(e) => return Err(anyhow!("Proof generated is invalid.")),
                }

                if current_height > height_on_nexus {
                    let public_inputs = NexusRollupPI {
                        nexus_hash: range[0],
                        state_root: H256::from(
                            batch_metadata.metadata.root_hash.as_fixed_bytes().clone(),
                        ),
                        //TODO: remove unwrap
                        height: current_height,
                        start_nexus_hash: start_nexus_hash.unwrap_or_else(|| {
                            H256::from(account_with_proof.account.start_nexus_hash)
                        }),
                        app_id: app_account_id.clone(),
                        img_id: StatementDigest(ZKSYNC_ADAPTER_ID),
                        rollup_hash: Some(rollup_hash),
                    };

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

                                    continue;
                                }
                            },
                            height: public_inputs.height,
                            data: public_inputs.rollup_hash.clone(),
                        }),
                    };

                    // fs::write(
                    //     format!("./submitproof_tx_{}.json", public_inputs.height),
                    //     serde_json::to_string(&tx).unwrap(),
                    // )
                    // .await;
                    match nexus_api.send_tx(tx).await {
                        Ok(i) => {
                            println!(
                                "Submitted proof to update state root on nexus. AppAccountId: {:?} Response: {:?} Stateroot: {:?}",
                                &app_account_id, i, &public_inputs.state_root
                            )
                        }
                        Err(e) => {
                            println!("Error when iniating account: {:?}", e);

                            continue;
                        }
                    }
                } else {
                    println!("Current height is lesser than height on nexus. current height: {} nexus height: {}", current_height, height_on_nexus);
                }

                // Persist adapter state data to the database
                db.put(&current_height.to_be_bytes(), &recursive_proof)?;
                db.put(
                    b"adapter_state_data",
                    &AdapterStateData {
                        last_height: last_height + 1,
                        adapter_config: adapter_state_data.adapter_config.clone(),
                    },
                )?;

                last_height = current_height;

                if last_height < height_on_nexus {
                    //No need to wait, can continue loop, as still need to catch up with latest height.
                    continue;
                }
            }
            Ok(ProofAPIResponse::Pending) => {
                println!("Got no header, sleeping for 10 seconds to try fetching");
            }
            Ok(ProofAPIResponse::Pruned) => {
                println!("Error fetching proof - Already pruned. Need to fetch from indexer");

                return Err(anyhow!("Error fetching proof - Already pruned. Need to fetch from indexer which is not implemented, exiting"));
            }
            Err(e) => {
                println!("Err while fetching proof {:?}", e);
            }
        }

        tokio::time::sleep(Duration::from_secs(10)).await;
    }
}
