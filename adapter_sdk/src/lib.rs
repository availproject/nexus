// use std::sync::mpsc::Receiver;
// use std::{result, str::FromStr};

// use avail_subxt;
// use henosis::fetcher::fetch_proof_and_pub_signal;
// use nexus_core::types::H256;
// use reqwest::Error;
// // use avail_subxt;
// use converter::converter::converter_fflonk_to_groth16;
// use nexus_core::agg_types::SubmitProofTransaction;
// use nexus_core::types::SubmitProof;
// use nexus_core::types::{AppAccountId, RollupPublicInputsV2, TxSignature};
// use serde::{de::DeserializeOwned, Deserialize, Serialize};
// use tokio::runtime::Runtime;
// use tokio::task;

// mod adapter;
// pub mod adapter_zkvm;
// pub mod types;

// fn main() {
//     let rt = tokio::runtime::Runtime::new().unwrap();
//     rt.block_on(monitor_avail_and_send_proof());
// }

// async fn monitor_avail_and_send_proof() {
//     let (subxt_client, _) = avail_subxt::build_client("wss://goldberg.avail.tools:443/ws", false)
//         .await
//         .unwrap();
//     println!("Built client");

//     //TODO: Need to change this to imported headers, and handle re orgs.
//     let mut header_subscription = subxt_client
//         .rpc()
//         .subscribe_finalized_block_headers()
//         .await
//         .expect("Subscription initialisation failed.");

//     while let Some(header_result) = header_subscription.next().await {
//         println!("Got next");
//         match header_result {
//             Ok(header) => {
//                 println!("Got header: {:?}", header.parent_hash);
//                 let rt: Runtime = Runtime::new().unwrap();
//                 let tx = rt.spawn_blocking(generate_proof_transaction).await.unwrap();

//                 if let Err(e) = send_post_request("<TBD>", tx).await {
//                     println!("Failed to send header: {}", e);
//                 }
//             }
//             Err(e) => {
//                 println!("Error getting next header: {}", e);
//                 break;
//             }
//         }
//     }

//     println!("exited...")
// }

// fn generate_proof_transaction() -> SubmitProofTransaction {
//     let rt = Runtime::new().unwrap();
//     let tx_hash_str = "0x38517b8514418d4fca0ff8b6dffe43199bfccd5b368523d747b01f76471bb8a4";
//     let txn_hash = tx_hash_str.parse().unwrap();
//     let resp = rt.block_on(async move { fetch_proof_and_pub_signal(txn_hash).await });
//     let receipt = converter_fflonk_to_groth16([resp.0], [resp.1]);

//     let public_inputs: RollupPublicInputsV2 = RollupPublicInputsV2 {
//         pre_state_root: H256::zero(),
//         next_state_root: H256::zero(),
//         tx_root: H256::zero(),
//         statement: H256::zero(),
//     };

//     let app_account_id = AppAccountId([0u8; 32]);

//     let proof_params = SubmitProof {
//         app_account_id,
//         public_inputs,
//     };

//     let transaction = SubmitProofTransaction {
//         proof: receipt.snark,
//         signature: TxSignature([0u8; 64]),
//         params: proof_params,
//     };

//     transaction
// }

// async fn send_post_request<T: Serialize + DeserializeOwned>(
//     url: &str,
//     body: T,
// ) -> Result<(), Error> {
//     // Create a reqwest client
//     let client = reqwest::Client::new();

//     let _response = client.post(url).json(&body).send().await?;

//     println!("POST request to {} with body completed.", url);

//     Ok(())
// }

// fn hex_str_to_u8_array(hex_str: &str) -> [u8; 32] {
//     // Remove the "0x" prefix if present
//     let hex_str = hex_str.trim_start_matches("0x");

//     // Decode the hex string to a byte vector
//     let bytes = hex::decode(hex_str).expect("Decoding failed");

//     // Convert Vec<u8> to [u8; 32]
//     let bytes_array: [u8; 32] = bytes.try_into().expect("Incorrect length");

//     bytes_array
// }

pub mod adapter_zkvm;
pub mod proof_storage;
pub mod rollup;
pub mod state;
pub mod types;

use rollup::server;
use state::AdapterState;
use types::AdapterPublicInputs;

pub async fn run(public_inputs: AdapterPublicInputs) {
    let adapter_state = AdapterState::new(public_inputs);

    let state_copy = adapter_state.clone();
    tokio::spawn(async move {
        adapter_state.process_queue().await;
    });

    // Start the server with the state
    server(state_copy).await;
}
