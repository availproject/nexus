pub mod types;
use crate::types::Header;
use avail_subxt::config::Header as HeaderTrait;
use nexus_core::types::H256;
use std::io::prelude::*;
use std::thread;
use std::{fs::File, sync::Arc};
use tokio::sync::{
    mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
    Mutex,
};
use tokio::time::Duration;

pub struct Relayer {
    sender: UnboundedSender<Header>,
    receiver: Arc<Mutex<UnboundedReceiver<Header>>>,
}

impl Relayer {
    pub fn new() -> Self {
        //TODO: Check if this has to be shifted to a bounder channel.
        let (sender, receiver) = unbounded_channel::<Header>();

        Self {
            sender,
            receiver: Arc::new(Mutex::new(receiver)),
        }
    }

    pub fn receiver(&mut self) -> Arc<Mutex<UnboundedReceiver<Header>>> {
        self.receiver.clone()
    }

    pub async fn get_header_hash(&self, height: u32) -> H256 {
        let (subxt_client, _) =
            avail_subxt::build_client("wss://goldberg.avail.tools:443/ws", false)
                .await
                .unwrap();

        let hash = match subxt_client.rpc().block_hash(Some(height.into())).await {
            Ok(i) => i,
            Err(e) => {
                panic!("Cannot initiate rollup")
            }
        };

        H256::from(hash.unwrap().as_fixed_bytes().clone())
    }

    pub async fn start(&self, start_height: u32) -> () {
        println!("Started client.");
        let (subxt_client, _) =
            avail_subxt::build_client("wss://goldberg.avail.tools:443/ws", false)
                .await
                .unwrap();
        println!("Built client");

        let mut next_height = start_height;

        loop {
            let hash = match subxt_client
                .rpc()
                .block_hash(Some(next_height.into()))
                .await
            {
                Ok(i) => i,
                Err(e) => {
                    println!("Faced error {:?}, when getting block: {}", e, next_height);

                    thread::sleep(Duration::from_secs(2));
                    continue;
                }
            };

            if hash.is_none() {
                println!("No block yet. trying again in 2 seconds");

                thread::sleep(Duration::from_secs(2));
                continue;
            }

            let hash = hash.unwrap();

            let header = match subxt_client.rpc().header(Some(hash)).await {
                Ok(i) => i,
                Err(e) => {
                    println!("Faced error {:?}, when getting header: {}", e, next_height);

                    thread::sleep(Duration::from_secs(2));
                    continue;
                }
            };

            if header.is_none() {
                println!("No block yet. trying again in 2 seconds");

                thread::sleep(Duration::from_secs(2));
                continue;
            }

            let header = header.unwrap();

            if let Err(e) = self.sender.send(header) {
                println!("Failed to send header: {}", e);

                break;
            }

            next_height = next_height + 1;
        }

        // //TODO: Need to change this to imported headers, and handle re orgs.
        // let mut header_subscription = subxt_client
        //     .rpc()
        //     .subscribe_finalized_block_headers()
        //     .await
        //     .expect("Subscription initialisation failed.");

        // println!("subscribed");
        // while let Some(header_result) = header_subscription.next().await {
        //     println!("Got next");
        //     match header_result {
        //         Ok(header) => {
        //             println!("Sending header: {:?}", header.parent_hash);
        //             if let Err(e) = self.sender.send(header) {
        //                 println!("Failed to send header: {}", e);
        //             }
        //         }
        //         Err(e) => {
        //             println!("Error getting next header: {}", e);

        //             break;
        //         }
        //     }
        // }

        // println!("exited...")
        //Need to handle exit here.
    }
}
