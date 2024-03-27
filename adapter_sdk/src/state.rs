// track starting block of the rollup.
// track the last queried block of the rollup
// manage a basic data store for the proof generated with the following data: till_avail_block, proof, receipt

use crate::types::{AdapterPrivateInputs, AdapterPublicInputs, RollupProof};
use anyhow::{anyhow, Error};
use nexus_core::traits::{Proof, RollupPublicInputs};
use nexus_core::types::{AppId, AvailHeader, H256};
use relayer::types::Header;
use relayer::Relayer;
use risc0_zkp::core::digest::Digest;
use risc0_zkvm::{default_prover, Receipt};
use risc0_zkvm::{serde::to_vec, ExecutorEnv};
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};

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

    pub async fn run(&self) {
        //On every new header,
        //Check if the block is empty for the stored app ID.
        let mut relayer = Relayer::new();
        let receiver = relayer.receiver();

        tokio::spawn(async move {
            relayer.start().await;
        });

        let mut receiver = receiver.lock().await;

        while let Some(header) = receiver.recv().await {
            let new_queue_item = QueueItem {
                proof: None,
                blob: None,
                header: header.into(),
            };
            let mut queue = self.queue.lock().await;
            queue.push_back(new_queue_item);
        }
    }

    pub async fn process_queue(&mut self, rollup_public_inputs: PI) -> Result<Receipt, Error> {
        let mut queue = self.queue.lock().await;

        while let Some(queue_item) = queue.pop_front() {
            match &queue_item.blob {
                Some((ref hash, ref inclusion_proof)) => {
                    if queue_item.proof.is_some() {
                        // Process the proof as before.
                        println!("Processing proof for blob: {:?}", hash);
                        return Err(anyhow!("Failed to process proof for blob: {:?}", hash));
                    } else {
                        queue.push_back(queue_item.clone());
                        sleep(Duration::from_secs(5)).await;
                    }
                }
                None => {
                    println!(
                        "Creating and processing empty proof for header: {:?}",
                        queue_item.header
                    );

                    //If empty, creates an empty proof.
                    let private_inputs = AdapterPrivateInputs {
                        header: queue_item.header,
                        app_id: self.app_id.clone(),
                    };

                    let env = ExecutorEnv::builder()
                        .write(&to_vec(&queue_item.proof)?)
                        .unwrap()
                        .write(&to_vec(&self.previous_adapter_proof)?)
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

                    return prover.prove(env, &self.elf);
                }
            }
        }

        return Err(anyhow!("Failed to process queue"));
    }

    pub async fn add_proof(&mut self, proof: RollupProof<PI, P>) {
        let mut queue = self.queue.lock().await;

        let mut updated_proof: bool = false;
        for height in queue.iter_mut() {
            //Check if blob hash matches the blob hash in PI,
            match height.blob.clone() {
                Some(value) => {
                    //If found, then set updated_proof to true and then reset the proof field from None to the given proof.
                    if value.0 == proof.public_inputs.blob_hash() {
                        updated_proof = true;
                        height.proof = Some(proof);
                        break;
                    }
                }
                None => return,
            }
            //Improvement will be to check if the proof is verifying.
        }
    }

    // function to generate proof against avail data when proof is received and verified from the rollup
    pub async fn verify_and_generate_proof(
        &mut self,
        queueItem: QueueItem<PI, P>,
    ) -> Result<Receipt, Error> {
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
}
