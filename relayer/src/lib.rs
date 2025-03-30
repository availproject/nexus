pub mod types;
use crate::types::Header;
use avail_rust::prelude::*;
use nexus_core::types::H256;
use std::future::Future;
use std::sync::Arc;
use tokio::sync::{
    mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
    watch,
};
use tokio::time::Duration;

pub struct SimpleRelayer {
    rpc_url: String,
    sender: UnboundedSender<Header>,
    receiver: Arc<tokio::sync::Mutex<UnboundedReceiver<Header>>>,
    stop: watch::Sender<bool>,
}

pub trait Relayer {
    fn receiver(&mut self) -> Arc<tokio::sync::Mutex<UnboundedReceiver<Header>>>;
    fn get_header_hash(&self, height: u32) -> impl Future<Output = H256> + Send;
    fn start(&self, start_height: u32) -> impl Future<Output = ()> + Send;
    fn stop(&self);
}

impl Relayer for SimpleRelayer {
    fn receiver(&mut self) -> Arc<tokio::sync::Mutex<UnboundedReceiver<Header>>> {
        self.receiver.clone()
    }

    fn get_header_hash(&self, height: u32) -> impl Future<Output = H256> + Send {
        async move {
            let sdk = SDK::new(&self.rpc_url).await.unwrap();

            let hash = rpc::chain::get_block_hash(&sdk.client, Some(height.into()))
                .await
                .expect("cannot get block hash");

            H256::from(hash.as_fixed_bytes().clone())
        }
    }

    fn start(&self, start_height: u32) -> impl Future<Output = ()> + Send {
        async move {
            let sdk = SDK::new(&self.rpc_url).await.unwrap();
            println!("Built client");
            let mut next_height = start_height;
            let mut stop_rx = self.stop.subscribe();
            loop {
                if *stop_rx.borrow() {
                    println!("Stopping the relayer.");
                    break;
                }

                //TODO: Add reconnection logic
                // if !sdk.client.is_connected {

                // }

                let finalized_header_hash = match rpc::chain::get_finalized_head(&sdk.client).await {
                    Ok(i) => i,
                    Err(_) => {
                        println!("Error getting finalized_header_hash: {}", next_height);
                        tokio::time::sleep(Duration::from_secs(2)).await;
                        continue;
                    }
                };

                let finalized_header = match rpc::chain::get_header(&sdk.client, Some(finalized_header_hash)).await {
                    Ok(i) => i,
                    Err(e) => {
                        println!("Error getting finalized_header: {} {:?}", next_height, e);
                        tokio::time::sleep(Duration::from_secs(2)).await;
                        continue;
                    }
                };

                let header = if finalized_header.number == next_height {
                    finalized_header.clone()
                } else if finalized_header.number < next_height {
                    println!("Waiting for block {} to finalize", next_height);
                    tokio::time::sleep(Duration::from_secs(2)).await;
                    continue;
                } else {
                    let hash = match rpc::chain::get_block_hash(&sdk.client, Some(next_height)).await {
                        Ok(i) => i,
                        Err(_) => {
                            println!("Error getting block: {}", next_height);
                            tokio::time::sleep(Duration::from_secs(2)).await;
                            continue;
                        }
                    };

                    let header = match rpc::chain::get_header(&sdk.client, Some(hash)).await {
                        Ok(i) => i,
                        Err(_) => {
                            println!("Error getting header: {}", next_height);
                            tokio::time::sleep(Duration::from_secs(2)).await;
                            continue;
                        }
                    };

                    header
                };

                if let Err(e) = self.sender.send(header) {
                    println!("Failed to send header: {}", e);
                    break;
                }

                next_height += 1;
            }
        }
    }

    fn stop(&self) {
        let _ = self.stop.send(true); // Signal stop
    }
}

impl SimpleRelayer {
    pub fn new(rpc_url: &str) -> Self {
        let (sender, receiver) = unbounded_channel::<Header>();
        let (stop_tx, _) = watch::channel(false);

        Self {
            rpc_url: rpc_url.to_string(),
            sender,
            receiver: Arc::new(tokio::sync::Mutex::new(receiver)),
            stop: stop_tx,
        }
    }
}
