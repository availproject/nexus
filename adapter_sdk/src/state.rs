use crate::adapter_zkvm::verify_proof;
// track starting block of the rollup.
// track the last queried block of the rollup
// manage a basic data store for the proof generated with the following data: till_avail_block, proof, receipt
use crate::proof_storage::GenericProof;
use crate::types::{AdapterPrivateInputs, AdapterPublicInputs};
use avail_subxt;
use nexus_core::traits::{Proof, RollupPublicInputs};
use nexus_core::types::H256;
use risc0_zkvm::Receipt;
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::sleep;

// usage : create an object for this struct and use as a global dependency
#[derive(Debug, Clone)]
pub struct AdapterState<PI: RollupPublicInputs, P: Proof<PI>> {
    pub starting_block_number: u8,
    pub last_queried_block_number: u8,
    pub public_inputs: AdapterPublicInputs,
    private_inputs: AdapterPrivateInputs,
    pub(crate) proof_queue: Arc<Mutex<VecDeque<GenericProof<PI, P>>>>,
    pub blob_data: Arc<Mutex<VecDeque<H256>>>,
    pub vk: [u8; 32],
}

impl<PI: RollupPublicInputs, P: Proof<PI>> AdapterState<PI, P> {
    pub fn new(
        public_inputs: AdapterPublicInputs,
        private_inputs: AdapterPrivateInputs,
        vk: [u8; 32],
    ) -> Self {
        AdapterState {
            starting_block_number: 0,
            last_queried_block_number: 0,
            public_inputs,
            private_inputs,
            proof_queue: Arc::new(Mutex::new(VecDeque::new())),
            blob_data: Arc::new(Mutex::new(VecDeque::new())),
            vk,
        }
    }

    // function triggered by rollup in a loop to process its proofs.
    pub async fn process_queue(&mut self, rollup_public_inputs: PI) {
        self.verify_and_generate_proof(rollup_public_inputs).await
        // TODO: return the proof from above ( by modifying the zkvm ) and use it against blob data
    }

    // queries avail block to search for data related to the given app_id
    pub async fn query_avail_blocks(&self) {
        // TODO: return a mock result from this for now. Later, run this in seperte thread inside setup
        let (subxt_client, _) =
            avail_subxt::build_client("wss://goldberg.avail.tools:443/ws", false)
                .await
                .unwrap();
        println!("Built client");
    }

    pub async fn add_proof(&mut self, proof: GenericProof<PI, P>) {
        // proof input validation ???
        self.proof_queue.lock().await.push_back(proof)
    }

    // function to generate proof against avail data when proof is received and verified from the rollup
    pub async fn verify_and_generate_proof(&mut self, rollup_public_inputs: PI) {
        let proof_lock = self.proof_queue.lock().await;
        let front_proof_ref = proof_lock.front().expect("Queue is empty");
        let front_proof_clone = front_proof_ref.clone(); // Clone the value

        let new_public_inputs = verify_proof(
            front_proof_clone,
            rollup_public_inputs,
            Some(self.public_inputs.clone()),
            self.private_inputs.clone(),
            self.public_inputs.img_id,
            self.vk,
        );

        match (new_public_inputs) {
            Ok(value) => {
                self.public_inputs = value;
                self.proof_queue.lock().await.pop_front();
            }
            Err(e) => println!("Error: {}", e),
        }
    }

    // function to store the till_avail_block, and the corresponding adapter proof generated in local storage
    fn store_local_state(
        &mut self,
        queried_block_number: u8,
        latest_public_inputs: AdapterPublicInputs,
    ) {
        self.last_queried_block_number = queried_block_number;
        self.public_inputs = latest_public_inputs;
    }
}
