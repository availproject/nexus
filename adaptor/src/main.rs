use henosis::fetcher::fetch_proof_and_pub_signal;
use reqwest::Error;
use avail_subxt;
use nexus_core::types::SubmitProof;
use nexus_core::agg_types::SubmitProofTransaction;

#[tokio::main]
async fn main() {

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

async fn generate_proof_transaction() -> SubmitProofTransaction {
    let tx_hash = "0x38517b8514418d4fca0ff8b6dffe43199bfccd5b368523d747b01f76471bb8a4";
    let (proof, publicVarsZK) = fetch_proof_and_pub_signal(tx_hash).await();

    let proof_params = SubmitProof{
        app_account_id: 1,
        public_inputs : publicVars
    };

    let transaction = SubmitProofTransaction {
        proof: proof,
        signature: [u08;32],
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
