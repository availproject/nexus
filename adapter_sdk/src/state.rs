// track starting block of the rollup.
// track the last queried block of the rollup
// manage a basic data store for the proof generated with the following data: till_avail_block, proof, receipt
use crate::types::AdapterPublicInputs;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::sleep;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Proof(pub [u8; 32]);

// usage : create an object for this struct and use as a global dependency
#[derive(Debug)]
pub struct AdapterState {
    pub starting_block_number: u8,
    pub last_queried_block_number: u8,
    pub public_inputs: AdapterPublicInputs,
    proof_queue: Arc<Mutex<VecDeque<Proof>>>,
}

impl AdapterState {
    pub fn new(public_inputs: AdapterPublicInputs) -> Self {
        AdapterState {
            starting_block_number: 0,
            last_queried_block_number: 0,
            public_inputs,
            proof_queue: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    pub async fn process_queue(&self) {
        loop {
            let maybe_proof = {
                let mut queue = self.proof_queue.lock().await;
                queue.pop_front()
            };

            match maybe_proof {
                Some(proof) => {
                    println!("Processing proof: {:?}", proof);
                    // Add here the processing logic for each proof

                    // If the processing is significantly long or blocking, consider spawning a new task for it
                    // tokio::spawn(async move {
                    //     // process proof here
                    // });
                }
                None => {
                    // If the queue is empty, wait for some time before trying again
                    sleep(Duration::from_secs(1)).await;
                }
            }
        }
    }

    // queries avail block to search for data related to the given app_id
    pub fn query_avail_blocks(&self) {}

    // function to generate proof against avail data when proof is received and verified from the rollup
    fn verify_and_generate_proof(&self) {}

    // function to store the till_avail_block, and the corresponding adapter proof generated in local storage
    fn store_local_state(&self) {}
}
