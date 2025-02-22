// track starting block of the rollup.
// track the last queried block of the rollup
// manage a basic data store for the proof generated with the following data: till_avail_block, proof, receipt

use crate::api::NexusAPI;
use crate::db::DB;
use crate::traits::RollupProof;
use crate::types::{
    AdapterConfig, AdapterPrivateInputs, AdapterPublicInputs, RollupProofWithPublicInputs,
    RollupPublicInputs,
};
use anyhow::{anyhow, Error};
use nexus_core::types::Proof;
use nexus_core::types::{
    AppAccountId, AppId, AvailHeader, InitAccount, NexusHeader, Proof as ZKProof, StatementDigest,
    SubmitProof, Transaction, TxParams, TxSignature, H256,
};
#[cfg(feature = "native-risc0")]
use nexus_core::zkvm::risczero::{ProofConversion, RiscZeroProver};
#[cfg(any(feature = "native-sp1"))]
use nexus_core::zkvm::sp1::{ProofConversion, Sp1Prover};
use nexus_core::zkvm::traits::{ZKVMEnv, ZKVMProof, ZKVMProver};
use nexus_core::zkvm::ProverMode;
use relayer::{Relayer, SimpleRelayer};
#[cfg(feature = "native-risc0")]
use risc0_zkvm::{default_prover, ExecutorEnvBuilder, Journal, Prover, Receipt, ReceiptClaim};
#[cfg(feature = "native-risc0")]
use risc0_zkvm::{serde::from_slice, ExecutorEnv};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fmt::Debug as DebugTrait;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::Arc;
use std::{clone, thread};
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};

#[cfg(feature = "native-sp1")]
use sp1_sdk::{utils, ProverClient, SP1PublicValues, SP1Stdin};

// #[cfg(feature = "native-sp1")]
//const ELF: &[u8] = include_bytes!("../../zksync_adapter/methods/sp1-guest/elf/riscv32im-succinct-zkvm-elf");

#[derive(Debug, Clone, Serialize, Deserialize)]
struct InclusionProof(pub Vec<u8>);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct QueueItem<P: RollupProof + Clone> {
    proof: Option<RollupProofWithPublicInputs<P>>,
    blob: Option<(H256, InclusionProof)>,
    header: AvailHeader,
}

// usage : create an object for this struct and use as a global dependency
pub struct AdapterState<
    P: RollupProof + Clone + DeserializeOwned + Serialize + 'static,
    Z: ZKVMEnv + 'static,
    ZP: ZKVMProof
        + DebugTrait
        + Clone
        + DeserializeOwned
        + Serialize
        + Send
        + TryInto<Proof>
        + Debug
        + 'static,
> where
    <ZP as TryInto<Proof>>::Error: Into<anyhow::Error>,
{
    pub starting_block_number: u32,
    pub queue: Arc<Mutex<VecDeque<QueueItem<P>>>>,
    pub previous_adapter_proof: Option<(ZP, AdapterPublicInputs, u32)>,
    pub elf: Vec<u8>,
    pub elf_id: StatementDigest,
    pub vk: [u8; 32],
    pub app_id: AppId,
    pub db: Arc<Mutex<DB<P, ZP>>>,
    pub p: PhantomData<Z>,
    pub pp: PhantomData<ZP>,
    pub nexus_api: NexusAPI,
    pub prover_mode: ProverMode,
    pub avail_url: String,
}

impl<
        P: RollupProof + Clone + DeserializeOwned + Serialize + Send,
        Z: ZKVMEnv,
        ZP: ZKVMProof
            + DebugTrait
            + Clone
            + DeserializeOwned
            + Serialize
            + Send
            + TryInto<Proof>
            + Debug
            + ProofConversion,
    > AdapterState<P, Z, ZP>
where
    <ZP as TryInto<Proof>>::Error: Into<anyhow::Error>,
{
    pub fn new(storage_path: &str, config: AdapterConfig) -> Self {
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
            p: PhantomData,
            pp: PhantomData,
            nexus_api: NexusAPI::new(&"http://127.0.0.1:7000"),
            prover_mode: config.prover_mode,
            avail_url: config.avail_url,
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
        let mut relayer = SimpleRelayer::new(&self.avail_url);
        let receiver = relayer.receiver();
        let start_height = match &self.previous_adapter_proof {
            Some(i) => i.2,
            None => self.starting_block_number,
        };
        if self.previous_adapter_proof.is_none() {
            let header_hash = relayer.get_header_hash(self.starting_block_number).await;
            let nexus_hash: H256 = self.nexus_api.get_header(&header_hash).await?.hash();
            let tx = Transaction {
                signature: TxSignature([0u8; 64]),
                params: TxParams::InitAccount(InitAccount {
                    app_id: AppAccountId::from(self.app_id.clone()),
                    statement: self.elf_id.clone(),
                    start_nexus_hash: nexus_hash,
                }),
            };

            let client = reqwest::Client::new();

            let response = client
                .post("http://127.0.0.1:7000/tx")
                .json(&tx)
                .send()
                .await?;

            // Check if the request was successful
            if response.status().is_success() {
                let body = response.text().await?;
                println!("✅ Initiated rollup {}", body);
            } else {
                println!("❌ Failed to initiate rollup {}", response.status());
            }
        }

        let relayer_handle = tokio::spawn(async move {
            println!("Start height {}", start_height);
            //TODO: Should be able to start from last processed height.
            relayer.start(start_height).await;
        });

        let queue_clone = self.queue.clone();
        let db_clone = self.db.clone();
        let db_clone_2 = self.db.clone();

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

                //Storing the queue in storage.
                db_clone
                    .lock()
                    .await
                    .store_last_known_queue(&queue)
                    .unwrap();
            }
        });
        let nexus_api_clone = self.nexus_api.clone();
        let submission_handle = tokio::spawn(async move {
            let nexus_api = nexus_api_clone;
            Self::manage_submissions(db_clone_2, &nexus_api).await
        });

        match self.process_queue().await {
            Ok(_) => (),
            Err(e) => println!("Exiting because of error: {:?}", e),
        };

        tokio::try_join!(avail_syncer_handle, relayer_handle, submission_handle)
            .unwrap()
            .2
            .unwrap();

        Ok(())
    }

    async fn manage_submissions(
        db: Arc<Mutex<DB<P, ZP>>>,
        nexus_api: &NexusAPI,
    ) -> Result<P, Error> {
        loop {
            thread::sleep(Duration::from_secs(2));

            let latest_proof = {
                let db_lock = db.lock().await;

                let last_proof = match db_lock.get_last_proof()? {
                    Some(i) => i,
                    None => continue,
                };

                last_proof
            };

            println!(
                "👨‍💻 Latest proof current for avail height : {:?}, state root: {:?}",
                &latest_proof.2, &latest_proof.1.state_root
            );

            let response = reqwest::get("http://127.0.0.1:7000/range").await?;

            // Check if the request was successful
            if !response.status().is_success() {
                println!(
                    "⛔️ Request to nexus failed with status {}. Nexus must be down",
                    response.status()
                );

                // Deserialize the response body into a Vec<H256>
                continue;
            }

            let range: Vec<H256> = match nexus_api.get_range().await {
                Ok(i) => i,
                Err(e) => continue,
            };

            let mut is_in_range = false;

            for h in range.iter() {
                if h.clone() == latest_proof.1.nexus_hash {
                    is_in_range = true;
                }
            }

            if is_in_range {
                println!("INside match");
                let client = reqwest::Client::new();
                let tx = Transaction {
                    signature: TxSignature([0u8; 64]),
                    params: TxParams::SubmitProof(SubmitProof {
                        proof: match latest_proof.0.try_into() {
                            Ok(i) => i,
                            Err(e) => return Err(anyhow!(e)),
                        },
                        height: latest_proof.1.height,
                        nexus_hash: latest_proof.1.nexus_hash,
                        state_root: latest_proof.1.state_root,
                        app_id: latest_proof.1.app_id,
                        data: None,
                    }),
                };

                let response = client
                    .post("http://127.0.0.1:7000/tx")
                    .json(&tx)
                    .send()
                    .await?;

                // Check if the request was successful
                if response.status().is_success() {
                    let body = response.text().await?;
                    println!(
                        "✅ Posted proof for avail height: {:?}, state root: {:?}",
                        &latest_proof.2, &latest_proof.1.state_root
                    );
                } else {
                    println!(
                        "❌ Request failed with status: {}, for avail height: {:?}, state root: {:?}",
                        response.status(),
                        &latest_proof.2,
                        &latest_proof.1.state_root
                    );
                }
            } else {
                println!("⏳ Not in range yet. Rollup at height: {}", latest_proof.2);
            }
        }
    }

    async fn process_queue(&mut self) -> Result<ZP, Error>
    where
        ZP: ZKVMProof
            + DebugTrait
            + Clone
            + DeserializeOwned
            + Serialize
            + Send
            + TryInto<Proof>
            + Debug,
    {
        loop {
            let queue_item = {
                let queue_lock = self.queue.lock().await;
                let item = queue_lock.front().cloned();
                item
            };

            if queue_item.is_none() {
                thread::sleep(Duration::from_secs(2));

                continue; // Restart the loop
            };
            let queue_item = queue_item.unwrap();
            if queue_item.blob.is_some() && queue_item.proof.is_none() {
                thread::sleep(Duration::from_secs(10));

                continue; // Restart the loop
            }

            let receipt = match self.verify_and_generate_proof(&queue_item).await {
                Err(e) => {
                    return Err(e);
                }
                Ok(i) => i,
            };
            // TODO]
            // let adapter_pi: AdapterPublicInputs = from_slice(&receipt.journal.bytes)?;
            // self.queue.lock().await.pop_front();

            // self.db.lock().await.store_last_proof(&(
            //     receipt.clone(),
            //     adapter_pi.clone(),
            //     queue_item.header.number,
            // ))?;
            // self.previous_adapter_proof = Some((
            //     receipt.clone(),
            //     adapter_pi.clone(),
            //     queue_item.header.number,
            // ));
        }
    }

    pub async fn add_proof(&mut self, proof: RollupProofWithPublicInputs<P>) -> Result<(), Error> {
        let mut queue = self.queue.lock().await;

        let mut updated_proof: bool = false;
        for height in queue.iter_mut() {
            //Check if blob hash matches the blob hash in PI,
            match height.blob.clone() {
                Some(value) => {
                    //If found, then set updated_proof to true and then reset the proof field from None to the given proof.
                    if value.0 == proof.public_inputs.blob_hash {
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
    async fn verify_and_generate_proof(
        &mut self,
        queue_item: &QueueItem<P>,
        // TODO change the return type to ZP
    ) -> Result<(ZP), Error>
    where
        ZP: ZKVMProof
            + DebugTrait
            + Clone
            + DeserializeOwned
            + Serialize
            + Send
            + TryInto<Proof>
            + Debug,
    {
        let nexus_header: NexusHeader =
            self.nexus_api.get_header(&queue_item.header.hash()).await?;

        let private_inputs = AdapterPrivateInputs {
            nexus_header,
            avail_header: queue_item.header.clone(),
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

        //Add prover mode config
        #[cfg(feature = "native-sp1")]
        let mut zkvm = Sp1Prover::new(self.elf.clone(), self.prover_mode.clone());

        #[cfg(feature = "native-risc0")]
        let mut zkvm = RiscZeroProver::new(self.elf.clone(), self.prover_mode.clone());

        let prev_pi: Option<AdapterPublicInputs> = match prev_pi_and_receipt {
            None => None,
            Some((receipt, pi, _)) => {
                // env_builder.add_assumption(receipt);
                Some(pi)
            }
        };

        zkvm.add_input(&prev_pi);
        zkvm.add_input(&queue_item.proof);
        zkvm.add_input(&private_inputs);
        zkvm.add_input(&self.elf_id);
        zkvm.add_input(&self.vk);

        let zkvm_proof = zkvm.prove();

        let zkvm_proof = match zkvm_proof {
            Ok(i) => {
                let proof: ZP = i.try_into().map_err(|_| anyhow!("Conversion failed"))?;
                return Ok(proof);
            }
            Err(e) => return Err(anyhow!(e)),
        };
    }
}
