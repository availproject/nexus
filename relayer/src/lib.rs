pub mod types;
use crate::types::Header;
use avail_subxt::config::Header as HeaderTrait;
use std::io::prelude::*;
use std::{fs::File, sync::Arc};
use tokio::sync::{
    mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
    Mutex,
};

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

    pub async fn start(&self) -> () {
        println!("Started client.");
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
        while let Some(header_result) = header_subscription.next().await {
            println!("Got next");
            match header_result {
                Ok(header) => {
                    println!("Sending header: {:?}", header.parent_hash);
                    if let Err(e) = self.sender.send(header) {
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
        //Need to handle exit here.
    }
}
