pub mod types;
use crate::types::Header;
use avail_subxt::config::Header as HeaderTrait;
use nexus_core::types::H256;
use std::future::Future;
use std::sync::Arc;
use tokio::sync::{
    mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
    watch,
};
use tokio::time::Duration;

pub struct SimpleRelayer {
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
            let (subxt_client, _) =
                avail_subxt::build_client("wss://goldberg.avail.tools:443/ws", false)
                    .await
                    .unwrap();

            let hash = match subxt_client.rpc().block_hash(Some(height.into())).await {
                Ok(i) => i,
                Err(_) => panic!("Cannot initiate rollup"),
            };

            H256::from(hash.unwrap().as_fixed_bytes().clone())
        }
    }

    fn start(&self, start_height: u32) -> impl Future<Output = ()> + Send {
        async move {
            println!("Started client.");
            let (subxt_client, _) =
                avail_subxt::build_client("wss://turing-rpc.avail.so:443/ws", false)
                    .await
                    .unwrap();
            println!("Built client");

            let mut next_height = start_height;
            let mut stop_rx = self.stop.subscribe();

            loop {
                if *stop_rx.borrow() {
                    println!("Stopping the relayer.");
                    break;
                }

                let hash = match subxt_client
                    .rpc()
                    .block_hash(Some(next_height.into()))
                    .await
                {
                    Ok(i) => i,
                    Err(_) => {
                        println!("Error getting block: {}", next_height);
                        tokio::time::sleep(Duration::from_secs(2)).await;
                        continue;
                    }
                };

                if hash.is_none() {
                    println!("No block yet, trying again in 2 seconds.");
                    tokio::time::sleep(Duration::from_secs(2)).await;
                    continue;
                }

                let hash = hash.unwrap();
                let header = match subxt_client.rpc().header(Some(hash)).await {
                    Ok(i) => i,
                    Err(_) => {
                        println!("Error getting header: {}", next_height);
                        tokio::time::sleep(Duration::from_secs(2)).await;
                        continue;
                    }
                };

                if header.is_none() {
                    println!("No header yet, trying again in 2 seconds.");
                    tokio::time::sleep(Duration::from_secs(2)).await;
                    continue;
                }

                let header = header.unwrap();

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
    pub fn new() -> Self {
        let (sender, receiver) = unbounded_channel::<Header>();
        let (stop_tx, _) = watch::channel(false);

        Self {
            sender,
            receiver: Arc::new(tokio::sync::Mutex::new(receiver)),
            stop: stop_tx,
        }
    }
}
