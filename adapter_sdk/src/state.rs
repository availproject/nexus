// track starting block of the rollup.
// track the last queried block of the rollup
// manage a basic data store for the proof generated with the following data: till_avail_block, proof, receipt

use crate::db::DB;
use crate::traits::{Proof, RollupPublicInputs};
use crate::types::{AdapterConfig, AdapterPrivateInputs, AdapterPublicInputs, RollupProof};
use anyhow::{anyhow, Error};
use nexus_core::types::{AppId, AvailHeader, StatementDigest, H256};
use relayer::Relayer;
use risc0_zkvm::{default_prover, Receipt};
use risc0_zkvm::{
    serde::{from_slice, to_vec},
    sha::rust_crypto::Digest,
    ExecutorEnv,
};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::Arc;
use std::thread;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct InclusionProof(pub Vec<u8>);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct QueueItem<I: RollupPublicInputs + Clone, P: Proof<I> + Clone> {
    proof: Option<RollupProof<I, P>>,
    blob: Option<(H256, InclusionProof)>,
    header: AvailHeader,
}

// usage : create an object for this struct and use as a global dependency
pub struct AdapterState<
    PI: RollupPublicInputs + Clone + DeserializeOwned + Serialize + 'static,
    P: Proof<PI> + Clone + DeserializeOwned + Serialize + 'static,
> {
    pub starting_block_number: u32,
    pub queue: Arc<Mutex<VecDeque<QueueItem<PI, P>>>>,
    pub previous_adapter_proof: Option<(Receipt, AdapterPublicInputs, u32)>,
    pub elf: Vec<u8>,
    pub elf_id: StatementDigest,
    pub vk: [u8; 32],
    pub app_id: AppId,
    pub db: Arc<Mutex<DB<PI, P>>>,
}

impl<
        PI: RollupPublicInputs + Clone + DeserializeOwned + Serialize + Send,
        P: Proof<PI> + Clone + DeserializeOwned + Serialize + Send,
    > AdapterState<PI, P>
{
    pub fn new(storage_path: String, config: AdapterConfig) -> Self {
        let db = DB::from_path(storage_path);

        AdapterState {
            starting_block_number: config.rollup_start_height,
            queue: Arc::new(Mutex::new(VecDeque::new())),
            previous_adapter_proof: None,
            elf: config.elf,
            elf_id: config.adapter_elf_id,
            vk: config.vk,
            app_id: config.app_id,
            db: Arc::new(Mutex::new(db)),
        }
    }

    pub async fn run(&mut self) -> Result<(), Error> {
        let (stored_queue, previous_adapter_proof) = {
            let db = self.db.lock().await;

            (db.get_last_known_queue()?, db.get_last_proof()?)
        };
        let mut queue = self.queue.lock().await;
        queue.clear();

        //TODO: Optimise below part.
        stored_queue
            .iter()
            .for_each(|item| queue.push_back(item.clone()));

        drop(queue);
        self.previous_adapter_proof = previous_adapter_proof;

        //On every new header,
        //Check if the block is empty for the stored app ID.
        let mut relayer = Relayer::new();
        let receiver = relayer.receiver();
        let start_height = match &self.previous_adapter_proof {
            Some(i) => i.2,
            None => self.starting_block_number,
        };

        let relayer_handle = tokio::spawn(async move {
            println!("Start height {}", start_height);
            //TODO: Should be able to start from last processed height.
            relayer.start(start_height).await;
        });

        let queue_clone = self.queue.clone();
        let db_clone = self.db.clone();

        let avail_syncer_handle = tokio::spawn(async move {
            let mut receiver = receiver.lock().await;

            while let Some(header) = receiver.recv().await {
                let new_queue_item = QueueItem {
                    proof: None,
                    blob: None,
                    header: AvailHeader::from(&header),
                };
                let mut queue = queue_clone.lock().await;
                queue.push_back(new_queue_item);

                println!("Updated queue");
                //Storing the queue in storage.
                db_clone
                    .lock()
                    .await
                    .store_last_known_queue(&queue)
                    .unwrap();
            }
        });

        match self.process_queue().await {
            Ok(_) => (),
            Err(e) => println!("{:?}", e),
        };

        tokio::try_join!(avail_syncer_handle, relayer_handle).unwrap();

        Ok(())
    }

    pub async fn process_queue(&mut self) -> Result<Receipt, Error> {
        loop {
            println!("processing item");
            let queue_item = {
                let queue_lock = self.queue.lock().await;
                let item = queue_lock.front().cloned();
                item
            };

            if queue_item.is_none() {
                println!("Empty items. Waiting for 2 seconds");
                thread::sleep(Duration::from_secs(2));

                continue; // Restart the loop
            };
            let queue_item = queue_item.unwrap();
            println!("Processing item {:?}", queue_item.header.number);
            if queue_item.blob.is_some() && queue_item.proof.is_none() {
                println!("Proof not available for the blob yet. Checking in 10 seconds");
                thread::sleep(Duration::from_secs(10));

                continue; // Restart the loop
            };

            let receipt = match self.verify_and_generate_proof(&queue_item) {
                Err(e) => {
                    println!("{:?}", &e);

                    return Err(e);
                }
                Ok(i) => i,
            };
            println!("Got rreceipt");
            let adapter_pi: AdapterPublicInputs = from_slice(&receipt.journal.bytes)?;
            self.queue.lock().await.pop_front();

            self.db.lock().await.store_last_proof(&(
                receipt.clone(),
                adapter_pi.clone(),
                queue_item.header.number,
            ))?;
            self.previous_adapter_proof = Some((
                receipt.clone(),
                adapter_pi.clone(),
                queue_item.header.number,
            ));
            println!("Sotred everything..");
        }
    }

    pub async fn add_proof(&mut self, proof: RollupProof<PI, P>) -> Result<(), Error> {
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
                None => continue,
            }
            //Improvement will be to check if the proof is verifying.
        }

        if updated_proof {
            return Ok(());
        }

        Err(anyhow!("Blob not found for given proof"))
    }

    // function to generate proof against avail data when proof is received and verified from the rollup
    fn verify_and_generate_proof(
        &mut self,
        queue_item: &QueueItem<PI, P>,
    ) -> Result<Receipt, Error> {
        let private_inputs = AdapterPrivateInputs {
            header: queue_item.header.clone(),
            app_id: self.app_id.clone(),
        };

        let prev_pi_and_receipt = match &self.previous_adapter_proof {
            None => {
                if queue_item.header.number != self.starting_block_number {
                    return Err(anyhow!("Cannot find previous proof for recursion."));
                }

                None
            }
            Some(i) => Some(i.clone()),
        };

        let mut env_builder = ExecutorEnv::builder();

        let prev_pi: Option<AdapterPublicInputs> = match prev_pi_and_receipt {
            None => None,
            Some((receipt, pi, _)) => {
                println!("Added assumption.");

                env_builder.add_assumption(receipt);

                Some(pi)
            }
        };

        let env = env_builder
            .write(&prev_pi)
            .unwrap()
            .write(&queue_item.proof)
            .unwrap()
            .write(&private_inputs)
            .unwrap()
            .write(&self.elf_id)
            .unwrap()
            .write(&self.vk)
            .unwrap()
            .build()
            .unwrap();

        let prover = default_prover();

        let receipt = prover.prove(env, &self.elf);

        receipt
    }
}
