// track starting block of the rollup.
// track the last queried block of the rollup
// manage a basic data store for the proof generated with the following data: till_avail_block, proof, receipt

use crate::types::{AdapterPrivateInputs, AdapterPublicInputs, RollupProof};
use anyhow::{anyhow, Error};
use nexus_core::traits::{Proof, RollupPublicInputs};
use nexus_core::types::{AppId, AvailHeader, H256};
use risc0_zkp::core::digest::Digest;
use risc0_zkvm::{default_prover, Receipt};
use risc0_zkvm::{serde::to_vec, ExecutorEnv};
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
struct InclusionProof(Vec<u8>);

#[derive(Debug, Clone)]
struct QueueItem<I: RollupPublicInputs, P: Proof<I>> {
    proof: Option<RollupProof<I, P>>,
    blob: Option<(H256, InclusionProof)>,
    header: AvailHeader,
}

// usage : create an object for this struct and use as a global dependency
#[derive(Debug, Clone)]
pub struct AdapterState<PI: RollupPublicInputs, P: Proof<PI>> {
    pub starting_block_number: u8,
    pub last_queried_block_number: u8,
    pub queue: Arc<Mutex<VecDeque<QueueItem<PI, P>>>>,
    pub previous_adapter_proof: Option<(Receipt, AdapterPublicInputs)>,
    pub elf: Box<[u8]>,
    pub elf_id: Digest,
    pub vk: [u8; 32],
    pub app_id: AppId,
}

impl<PI: RollupPublicInputs, P: Proof<PI>> AdapterState<PI, P> {
    pub fn new(app_id: AppId, vk: [u8; 32], zkvm_elf: &[u8], zkvm_id: impl Into<Digest>) -> Self {
        AdapterState {
            starting_block_number: 0,
            last_queried_block_number: 0,
            queue: Arc::new(Mutex::new(VecDeque::new())),
            previous_adapter_proof: None,
            elf: zkvm_elf.into(),
            elf_id: zkvm_id.into(),
            vk,
            app_id,
        }
    }

    pub async fn run() {
        //On every new header,
        //Check if the block is empty for the stored app ID.
    }

    // function triggered by rollup in a loop to pro+cess its proofs.
    pub async fn process_queue(&mut self, rollup_public_inputs: PI) {
        // self.verify_and_generate_proof(rollup_public_inputs).await
        // TODO: return the proof from above ( by modifying the zkvm ) and use it against blob data

        //Loops through the queue, for the first item in the queue, checks if the blob is empty.
        //If empty, creates an empty proof.
        //If not empty, checks if the proof is already submitted. If not submitted, exits loop until it is available.
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

    pub async fn add_proof(&mut self, proof: RollupProof<PI, P>) {
        let queue = self.queue.lock().await;

        let updated_proof: bool = false;
        for height in queue.iter() {
            //Check if blob hash matches the blob hash in PI,
            //If found, then set updated_proof to true and then reset the proof field from None to the given proof.
            //Improvement will be to check if the proof is verifying.
        }
        // proof input validation ???
    }

    // function to generate proof against avail data when proof is received and verified from the rollup
    pub async fn verify_and_generate_proof(
        &mut self,
        queueItem: QueueItem<PI, P>,
    ) -> Result<Receipt, Error> {
        let proof_lock = self.queue.lock().await;
        let front_proof_ref = proof_lock.front().expect("Queue is empty");
        let front_proof_clone = front_proof_ref.clone(); // Clone the value
        let private_inputs = AdapterPrivateInputs {
            header: queueItem.header,
            app_id: self.app_id.clone(),
        };

        let proof = match queueItem.proof {
            None => return Err(anyhow!("Proof is empty")),
            Some(i) => i,
        };

        let prev_pi = match &self.previous_adapter_proof {
            None => {
                if self.last_queried_block_number != self.starting_block_number {
                    return Err(anyhow!("Cannot find previous proof for recursion."));
                }

                None
            }
            Some(i) => Some(i.clone()),
        };

        let env = ExecutorEnv::builder()
            .write(&to_vec(&proof)?)
            .unwrap()
            .write(&to_vec(&prev_pi)?)
            .unwrap()
            .write(&to_vec(&private_inputs)?)
            .unwrap()
            .write(&to_vec(&self.elf_id)?)
            .unwrap()
            .write(&to_vec(&self.vk)?)
            .unwrap()
            .build()
            .unwrap();

        let prover = default_prover();

        let receipt = prover.prove(env, &self.elf);

        receipt
    }

    // function to store the till_avail_block, and the corresponding adapter proof generated in local storage
    // fn store_local_state(
    //     &mut self,
    //     queried_block_number: u8,
    //     latest_public_inputs: AdapterPublicInputs,
    // ) {
    //     self.last_queried_block_number = queried_block_number;
    //     self.public_inputs = latest_public_inputs;
    // }
}
