use avail_subxt;
use henosis::fetcher::fetch_proof_and_pub_signal;
use nexus_core::agg_types::SubmitProofTransaction;
use nexus_core::types::{AppAccountId, RollupPublicInputsV2, SubmitProof, TxSignature};
use reqwest::Error;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
pub use sparse_merkle_tree::H256;

#[tokio::main]
async fn main() {
    monitor_avail_and_send_proof().await
}

async fn monitor_avail_and_send_proof() {
    let (subxt_client, _) = avail_subxt::build_client("wss://goldberg.avail.tools:443/ws", false)
        .await
        .unwrap();
    println!("Built client");

    //TODO: Need to change this to imported headers, and handle re orgs.
    let mut header_subscription = subxt_client
        .rpc()
        .subscribe_finalized_block_headers()
        .await
        .expect("Subscription initialisation failed.");

    while let Some(header_result) = header_subscription.next().await {
        println!("Got next");
        match header_result {
            Ok(header) => {
                println!("Got header: {:?}", header.parent_hash);
                let tx = generate_proof_transaction().await;

                if let Err(e) = send_post_request("<TBD>", tx).await {
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

async fn generate_proof_transaction() -> SubmitProofTransaction {
    let tx_hash =
        hex_str_to_u8_array("0x38517b8514418d4fca0ff8b6dffe43199bfccd5b368523d747b01f76471bb8a4");
    let (proof, _, _, nle, nsr) = fetch_proof_and_pub_signal(tx_hash.into()).await;

    let public_vars = RollupPublicInputsV2 {
        next_state_root: nsr.into(),
        pre_state_root: nle.into(),
        statement: tx_hash.into(),
        tx_root: tx_hash.into(),
    };

    let proof_params = SubmitProof {
        app_account_id: AppAccountId([0u8; 32]),
        public_inputs: public_vars,
    };

    let transaction = SubmitProofTransaction {
        proof: proof.iter().flat_map(|s| s.bytes()).collect(),
        signature: TxSignature([0u8; 64]),
        params: proof_params,
    };

    transaction
}

async fn send_post_request<T: Serialize + DeserializeOwned>(
    url: &str,
    body: T,
) -> Result<(), Error> {
    // Create a reqwest client
    let client = reqwest::Client::new();

    let _response = client.post(url).json(&body).send().await?;

    println!("POST request to {} with body completed.", url);

    Ok(())
}

fn hex_str_to_u8_array(hex_str: &str) -> [u8; 32] {
    // Remove the "0x" prefix if present
    let hex_str = hex_str.trim_start_matches("0x");

    // Decode the hex string to a byte vector
    let bytes = hex::decode(hex_str).expect("Decoding failed");

    // Convert Vec<u8> to [u8; 32]
    let bytes_array: [u8; 32] = bytes.try_into().expect("Incorrect length");

    bytes_array
}
