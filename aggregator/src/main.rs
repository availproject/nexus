use crate::rpc::routes;
use avail_subxt;
use henosis::fetcher::fetch_proof_and_pub_signal;
use nexus_core::agg_types::SubmitProofTransaction;
use nexus_core::types::{AppAccountId, RollupPublicInputsV2, SubmitProof, TxSignature};
use reqwest::Error;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
pub use sparse_merkle_tree::H256;

fn main() {
    monitor_avail_and_receive_proof().await;

    let routes = routes();
    let cors = warp::cors()
        .allow_any_origin()
        .allow_methods(vec!["POST"])
        .allow_headers(vec!["content-type"]);
    let routes = routes.with(cors);
    let server: tokio::task::JoinHandle<()> = tokio::spawn(async move {
        println!("trying to start rpc server");
        let address =
            SocketAddr::from_str(format!("{}:{}", String::from("127.0.0.1"), 7000).as_str())
                .context("Unable to parse host address from config")
                .unwrap();

        println!("RPC Server running on: {:?}", &address);
        warp::serve(routes).run(address).await;
    });

    let result = tokio::try_join!(server, execution_engine, relayer);

    match result {
        Ok((_, _, _)) => {
            println!("Exiting node, should not have happened.");
        }
        Err(e) => {
            println!("Exiting node, should not have happened. {:?}", e);
        }
    }
}

async fn monitor_avail_and_receive_proof() {
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
            }
            Err(e) => {
                println!("Error getting next header: {}", e);
                break;
            }
        }
    }
}
