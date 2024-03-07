use std::str::FromStr;
use std::sync::mpsc::Receiver;

use avail_subxt::config::polkadot::H256;
use henosis::fetcher::fetch_proof_and_pub_signal;
use reqwest::Error;
// use avail_subxt;
use nexus_core::types::SubmitProof;
use nexus_core::agg_types::SubmitProofTransaction;
use ckb_types::h256;
use converter::converter::converter_fflonk_to_groth16;
use tokio::runtime::Runtime;
use tokio::task;

fn main() {
    let transaction = generate_proof_transaction();
    let subscription = get_avail_subscription();
    monitor_avail_and_send_proof(subscription)
}
// [0u8; 32]

async fn get_avail_subscription() -> Subscription<Header> {
    let (subxt_client, _) =
    avail_subxt::build_client("wss://goldberg.avail.tools:443/ws", false)
        .await
        .unwrap();
    println!("Built client");

    //TODO: Need to change this to imported headers, and handle re orgs.
    let mut header_subscription = subxt_client
        .rpc()
        .subscribe_finalized_block_headers()
        .await
        .expect("Subscription initialisation failed.");

    println!("subscribed");
    header_subscription
}

async fn monitor_avail_and_send_proof(header_subscription: Subscription<Header>) {
    while let Some(header_result) = header_subscription.next().await {
        println!("Got next");
        match header_result {
            Ok(header) => {
                println!("Got header: {:?}", header.parent_hash);
                let tx = generate_proof_transaction().await();

                if let Err(e) = send_post_request("<TBD>",tx) {
                    println!("Failed to send header: {}", e);
                }
            }
            Err(e) => {
                println!("Error getting next header: {}", e);
                break;
            }
        }
    }

    println!("exited...")
}

fn generate_proof_transaction()  {
    let rt = Runtime::new().unwrap();
    let tx_hash_str = "0x38517b8514418d4fca0ff8b6dffe43199bfccd5b368523d747b01f76471bb8a4";
    let txn_hash = tx_hash_str.parse().unwrap();
    let resp = rt.block_on(async move {
        fetch_proof_and_pub_signal(txn_hash).await
    });
    let receipt = converter_fflonk_to_groth16([resp.0], [resp.1]);

    let public_inputs: RollupPublicInputsV2 = RollupPublicInputsV2 {
        prev_state_root: H256::from_str("0x"),
        next_state_root: H256::from_str("0x"),
        tx_root: H256::from_str("0x"),
        statement: H256::from_str("0x"),
    };

    let proof_params = SubmitProof{
        app_account_id: 1,
        public_inputs
    };

    let transaction = SubmitProofTransaction {
        proof: receipt.snark,
        signature: [u08; ],
        params: proof_params
    };

    transaction
}


async fn send_post_request (
    url: &str,
    body: T,
) -> Result<(), Error> {
    let client: Client = reqwest::Client::new();
    let _response = client.post(url).json(&body).send().await();

    println!("POST request to {} with body completed", url);

    Ok(())
}
