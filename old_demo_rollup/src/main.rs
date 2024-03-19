use nexus_core::types::{AppAccountId, AppId, InitAccount, TransactionV2, TxParamsV2, TxSignature};
use reqwest::Error;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

#[tokio::main]
async fn main() -> Result<(), Error> {
    // let init_tx = TransactionV2 {
    //     signature: TxSignature([0; 64]),
    //     params: TxParamsV2::InitAccount(InitAccount {
    //         app_id: AppAccountId::from(AppId(1)),
    //         statement: [1; 32],
    //     }),
    // };
    // let client = reqwest::Client::new();
    // println!("trying to send");

    // send_post_request("http://127.0.0.1:7000/tx", init_tx).await?;

    Ok(())
}

async fn send_post_request<T: Serialize + DeserializeOwned>(
    url: &str,
    body: T,
) -> Result<(), Error> {
    // Create a reqwest client
    let client = reqwest::Client::new();

    // Send the POST request with the JSON body
    let _response = client.post(url).json(&body).send().await?;

    // Simulate some processing time
    //tokio::time::sleep(Duration::from_secs(2)).await;

    println!("POST request to {} with body completed.", url);

    Ok(())
}
