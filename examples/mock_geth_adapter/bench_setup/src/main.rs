use adapter_sdk::{api::NexusAPI, types::AdapterConfig};
use anyhow::{Context, Error};
use geth_methods::{ADAPTER_ELF, ADAPTER_ID};
use nexus_core::db::NodeDB;
use nexus_core::types::{
    AccountState, AccountWithProof, AppAccountId, AppId, InitAccount, NexusHeader, NexusRollupPI, Proof, StatementDigest, SubmitProof, Transaction, TxParams, TxSignature, H256
};
use nexus_core::zkvm::risczero::RiscZeroProof;
use nexus_core::zkvm::ProverMode;
use risc0_zkvm::guest::env;
use risc0_zkvm::serde::to_vec;
use risc0_zkvm::{default_prover, ExecutorEnv};
use serde::{Deserialize, Serialize};
use std::env::args;
use std::fs;
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};
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
async fn main() {
    let prover_mode = ProverMode::MockProof;
    let nexus_api = NexusAPI::new(&"http://127.0.0.1:7001");
    let mut submit_proof_transactions = Vec::<Transaction>::new();
    for txn_index in 0..100 {
        let adapter_config = AdapterConfig {
            app_id: AppId(txn_index),
            elf: ADAPTER_ELF.to_vec(),
            adapter_elf_id: StatementDigest(ADAPTER_ID),
            vk: [0u8; 32],
            rollup_start_height: 606460,
            prover_mode: prover_mode.clone(),
            avail_url: String::from("wss://turing-rpc.avail.so:443/ws"),
        };
    
        // Retrieve or initialize the adapter state data from the database
        let adapter_state_data = AdapterStateData {
            last_height: 0,
            adapter_config,
        };
    
        // Main loop to fetch headers and run adapter
        let mut last_height = adapter_state_data.last_height;
        let mut start_nexus_hash = None;

        let app_account_id = AppAccountId::from(adapter_state_data.adapter_config.app_id.clone());
        let account_with_proof: AccountWithProof =
            match nexus_api.get_account_state(&app_account_id.as_h256()).await {
                Ok(i) => i,
                Err(e) => {
                    println!("{:?}", e);
                    continue;
                }
            };

        last_height = account_with_proof.account.height;
        let file_name = format!("headers/nexus_header.json");
        let file_content = fs::read_to_string(file_name).unwrap();
        let header: NexusHeader = serde_json::from_str(&file_content).unwrap();

        let height: u32 = 30; // random height
        let public_inputs = NexusRollupPI {
            nexus_hash: header.hash(),
            state_root: H256::zero(),
            height,
            start_nexus_hash: start_nexus_hash
                .unwrap_or_else(|| H256::from(account_with_proof.account.start_nexus_hash)),
            app_id: app_account_id.clone(),
            img_id: StatementDigest(ADAPTER_ID),
            rollup_hash: Some(H256::zero()),
        };

        let mut env_builder = ExecutorEnv::builder();
        let env = env_builder.write(&public_inputs).unwrap().build().unwrap();
        let prover = default_prover();
        let prove_info = match prover.prove(env, ADAPTER_ELF) {
            Ok(i) => i,
            Err(e) => {
                println!("Unable to generate proof due to error: {:?}", e);
                continue;
            }
        };

        let recursive_proof = RiscZeroProof(prove_info.receipt);

        let tx = Transaction {
            signature: TxSignature([0u8; 64]),
            params: TxParams::SubmitProof(SubmitProof {
                app_id: app_account_id.clone(),
                nexus_hash: header.hash(),
                state_root: public_inputs.state_root.clone(),
                proof: match recursive_proof.clone().try_into() {
                    Ok(i) => i,
                    Err(e) => {
                        println!("Unable to serialise proof: {:?}", e);
                        continue;
                    }
                },
                height: public_inputs.height,
                data: None,
            }),
        };

        submit_proof_transactions.push(tx.clone());
    };

    let json = serde_json::to_string_pretty(&submit_proof_transactions).unwrap();
    fs::write("src/submit_proof_transactions/transactions.json", json).unwrap();    
}
