// track starting block of the rollup.
// track the last queried block of the rollup
// manage a basic data store for the proof generated with the following data: till_avail_block, proof, receipt
use crate::proof_storage::{GenericProof, ProofTrait};
use crate::types::AdapterPublicInputs;
use risc0_zkvm::Receipt;
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::sleep;

// usage : create an object for this struct and use as a global dependency
#[derive(Debug, Clone)]
pub struct AdapterState<P: ProofTrait + 'static> {
    pub starting_block_number: u8,
    pub last_queried_block_number: u8,
    pub public_inputs: AdapterPublicInputs,
    proof_queue: Arc<Mutex<VecDeque<Box<GenericProof<P>>>>>,
}

impl<P: ProofTrait + 'static> AdapterState<P> {
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
                }
                None => {
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
