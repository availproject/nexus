// track starting block of the rollup.
// track the last queried block of the rollup
// manage a basic data store for the proof generated with the following data: till_avail_block, proof, receipt

use crate::db::DB;
use crate::traits::Proof;
use crate::types::{
    AdapterConfig, AdapterPrivateInputs, AdapterPublicInputs, RollupProof, RollupPublicInputs,
};
use anyhow::{anyhow, Error};
use nexus_core::types::{
    AppAccountId, AppId, AvailHeader, InitAccount, StatementDigest, SubmitProof, TransactionV2,
    TxParamsV2, TxSignature, H256,
};
use relayer::Relayer;
use risc0_zkvm::{default_prover, ExecutorEnvBuilder, Journal, Prover, Receipt, ReceiptClaim};
use risc0_zkvm::{serde::from_slice, ExecutorEnv};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::rc::Rc;
use std::sync::Arc;
use std::{clone, thread};
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};

#[cfg(feature = "sp1")]
use sp1_sdk::{utils, ProverClient, SP1PublicValues, SP1Stdin};

#[cfg(feature = "sp1")]
const ELF: &[u8] = include_bytes!("../../demo_rollup/program/elf/riscv32im-succinct-zkvm-elf");

#[derive(Debug, Clone, Serialize, Deserialize)]
struct InclusionProof(pub Vec<u8>);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct QueueItem<P: Proof + Clone> {
    proof: Option<RollupProof<P>>,
    blob: Option<(H256, InclusionProof)>,
    header: AvailHeader,
}


pub trait ZkVmReceipt {
    fn dummy(&self) -> Result<(), anyhow::Error> {
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZkVmReceiptRisc0 {
    receipt: Receipt,
}


impl ZkVmReceipt for ZkVmReceiptRisc0 {
    fn dummy(&self) -> Result<(), anyhow::Error> {
        Ok(())
    }
}


trait ZkVmAdapter {
    fn add_input<T: Serialize>(&mut self, input: T) -> Result<(), anyhow::Error>;

    fn prove(&mut self) -> Result<(), anyhow::Error>;
  
}

pub struct RiscZeroZKVM<'a> {
    env_builder: ExecutorEnvBuilder<'a>,
    default_prover: Rc<dyn Prover>,
    elf: Vec<u8>,
    receipt: Option<Receipt>,
    
}

#[cfg(feature = "sp1")]
pub struct SpOneZKVM {
    std_in: SP1Stdin,
    prover_client: ProverClient,
}

impl ZkVmAdapter for RiscZeroZKVM<'_> {
    fn add_input<T: Serialize>(&mut self, input: T) -> Result<(), anyhow::Error> {
        self.env_builder.write(&input).unwrap();
        // Ok(env)
        Ok(())
    }

    fn prove(&mut self) -> Result<(), anyhow::Error> {
        self.default_prover = default_prover();
        self.receipt = Some(self.default_prover.prove(self.env_builder.build().unwrap(), &self.elf)?);
        Ok(())
    }

    // fn add_assumption<T: Serialize>(&mut self, input: T) -> Result<(), anyhow::Error> {
    //     let env = self.env_builder.add_assumption(Receipt::);
    //     // Ok(env)
    //     Ok(())
    // }
}

#[cfg(feature = "sp1")]
impl ZkVmAdapter for SpOneZKVM {
    fn add_input<T: Serialize>(&mut self, input: T) -> Result<(), anyhow::Error> {
        // let env = self.prover_client.add_input(&input).unwrap();
        self.std_in.write(&input);
        // Ok(env)
        Ok(())
    }
}


// usage : create an object for this struct and use as a global dependency
pub struct AdapterState<P: Proof + Clone + DeserializeOwned + Serialize + 'static> {
    pub starting_block_number: u32,
    pub queue: Arc<Mutex<VecDeque<QueueItem<P>>>>,
    pub previous_adapter_proof: Option<(Receipt, AdapterPublicInputs, u32)>,
    pub elf: Vec<u8>,
    pub elf_id: StatementDigest,
    pub vk: [u8; 32],
    pub app_id: AppId,
    pub db: Arc<Mutex<DB<P>>>,
}

impl<P: Proof + Clone + DeserializeOwned + Serialize + Send> AdapterState<P> {
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

        if self.previous_adapter_proof.is_none() {
            let header_hash = relayer.get_header_hash(self.starting_block_number).await;
            let tx = TransactionV2 {
                signature: TxSignature([0u8; 64]),
                params: TxParamsV2::InitAccount(InitAccount {
                    app_id: AppAccountId::from(self.app_id.clone()),
                    statement: self.elf_id.clone(),
                    avail_start_hash: header_hash,
                }),
                proof: None,
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

        let submission_handle = tokio::spawn(Self::manage_submissions(db_clone_2));

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

    async fn manage_submissions(db: Arc<Mutex<DB<P>>>) -> Result<Receipt, Error> {
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

            let range: Vec<H256> = response.json().await?;

            let mut is_in_range = false;

            for h in range.iter() {
                if h.clone() == latest_proof.1.header_hash {
                    is_in_range = true;
                }
            }

            if is_in_range {
                println!("INside match");
                let client = reqwest::Client::new();
                let tx = TransactionV2 {
                    signature: TxSignature([0u8; 64]),
                    params: TxParamsV2::SubmitProof(SubmitProof {
                        public_inputs: latest_proof.1.clone(),
                    }),
                    proof: Some(latest_proof.0.inner),
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

    async fn process_queue(&mut self) -> Result<Receipt, Error> {
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

            //todo
            // let receipt = 
            match self.verify_and_generate_proof(&queue_item) {
                Err(e) => {
                    return Err(e);
                }
                Ok(i) => i,
            }
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

    pub async fn add_proof(&mut self, proof: RollupProof<P>) -> Result<(), Error> {
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
    fn verify_and_generate_proof(&mut self, queue_item: &QueueItem<P>) -> Result<(), Error> {
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

        let prover = default_prover();

        // let zkvm;
        #[cfg(feature = "sp1")]
        let zkvm = SpOneZKVM {
            std_in: SP1Stdin::new(),
            prover_client: ProverClient::new(),
        };

        // #[cfg(feature = "sp1")]
        // zkvm_sp1.prover_client.setup(ELF);
        // let mut env_builder = ExecutorEnv::builder();
        #[cfg(feature = "risc0")]
        let mut zkvm = RiscZeroZKVM {
            env_builder: ExecutorEnv::builder(),
            default_prover: default_prover(),
            elf: self.elf.clone(),
            receipt: None,
        };

        let prev_pi: Option<AdapterPublicInputs> = match prev_pi_and_receipt {
            None => None,
            Some((receipt, pi, _)) => {
                // env_builder.add_assumption(receipt);

                Some(pi)
            }
        };

        #[cfg(feature = "risc0")]
        {zkvm.add_input(&prev_pi);
        zkvm.add_input(&queue_item.proof);
        zkvm.add_input(&private_inputs);
        zkvm.add_input(&self.elf_id);
        zkvm.add_input(&self.vk);

        zkvm.prove();}

        Ok(())

        // let env = zkvm.env_builder
        //     .write(&prev_pi)
        //     .unwrap()
        //     .write(&queue_item.proof)
        //     .unwrap()
        //     .write(&private_inputs)
        //     .unwrap()
        //     .write(&self.elf_id)
        //     .unwrap()
        //     .write(&self.vk)
        //     .unwrap()
        //     .build()
        //     .unwrap();

        // let prover = zkvm.default_prover;

        // let receipt = zkvm.prove(env, &self.elf);

        // let receipt = prover.prove(env, &self.elf);

        //return
        // Ok(ZkVmReceipt::<T>::new(zkvm.receipt.unwrap()))
    }
}
