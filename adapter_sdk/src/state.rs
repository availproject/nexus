// track starting block of the rollup.
// track the last queried block of the rollup
// manage a basic data store for the proof generated with the following data: till_avail_block, proof, receipt

use crate::db::DB;
use crate::traits::{Proof, RollupPublicInputs};
use crate::types::{AdapterConfig, AdapterPrivateInputs, AdapterPublicInputs, RollupProof};
use anyhow::{anyhow, Context, Error};

use avail_core::{data_proof::ProofResponse, AppId as AvailAppID, DataProof};
use avail_subxt::AvailConfig;
use avail_subxt::{
    api, api::data_availability::calls::types::SubmitData, rpc::KateRpcClient, AvailClient,
    BoundedVec,
};
use nexus_core::types::{
    AppAccountId, AppId, AvailHeader, InitAccount, StatementDigest, SubmitProof, TransactionV2,
    TxParamsV2, TxSignature, H256,
};
use relayer::Relayer;
use risc0_zkvm::{default_prover, Receipt};
use risc0_zkvm::{serde::from_slice, ExecutorEnv};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use sp_core::H256 as AvailH256;
use std::{collections::VecDeque, env, sync::Arc, thread};

use subxt::{
    ext::sp_core::sr25519::Pair,
    ext::sp_core::Pair as PairT,
    tx::{PairSigner, Payload},
};

use subxt_signer::{bip39::Mnemonic, sr25519::Keypair};
use tokio::sync::Mutex;
use tokio::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct QueueItem<I: RollupPublicInputs + Clone, P: Proof<I> + Clone> {
    proof: Option<RollupProof<I, P>>,
    blob: Option<(H256, DataProof)>,
    header: AvailHeader,
}

// usage : create an object for this struct and use as a global dependency
pub struct AdapterState<
    PI: RollupPublicInputs + Clone + DeserializeOwned + Serialize + 'static,
    P: Proof<PI> + Clone + DeserializeOwned + Serialize + 'static,
> {
    pub starting_block_number: u32,
    pub(crate) queue: Arc<Mutex<VecDeque<QueueItem<PI, P>>>>,
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

    async fn manage_submissions(db: Arc<Mutex<DB<PI, P>>>) -> Result<Receipt, Error> {
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

    pub async fn process_queue(&mut self) -> Result<Receipt, Error> {
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
            };

            let receipt = match self.verify_and_generate_proof(&queue_item) {
                Err(e) => {
                    return Err(e);
                }
                Ok(i) => i,
            };

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
        if queue_item.blob.is_none() || queue_item.proof.is_none() {
            return Err(anyhow!("Incomplete proof item inside queue"));
        }

        let (hash, blob) = queue_item.blob.as_ref().unwrap();

        let private_inputs = AdapterPrivateInputs {
            header: queue_item.header.clone(),
            app_id: self.app_id.clone(),
            blob: (*hash, blob.clone()),
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
                env_builder.add_assumption(receipt);

                Some(pi)
            }
        };

        let env = env_builder
            .write(&prev_pi)
            .unwrap()
            .write(&queue_item.proof)
            .unwrap()
            .write(&queue_item.blob)
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

    pub async fn store_blob(&mut self, header: H256, blob: &[u8]) -> Result<(), Error> {
        let client = Self::establish_a_connection().await?;

        let mnemonic: String;

        match env::var("MNEMONIC") {
            Ok(value) => mnemonic = value,
            Err(e) => panic!("Couldn't read : {}", e),
        }
        let sender = PairT::from_string_with_seed(mnemonic.as_str(), None).unwrap();
        let signer = PairSigner::<AvailConfig, Pair>::new(sender.0);

        println!("Data submitted...");

        let data = BoundedVec(blob.into());
        let call = api::tx().data_availability().submit_data(data);
        let (block_hash, transaction_index) = self.send_tx(call, &signer, &client).await?;

        let inclusion_proof = self
            .get_inclusion_proof(&client, transaction_index, block_hash)
            .await;
        let blob_hash = inclusion_proof.as_ref().unwrap().data_proof.leaf.clone();
        let data_proof = inclusion_proof.unwrap().data_proof;

        let mut queue = self.queue.lock().await;

        for height in queue.iter_mut() {
            if height.header.hash() == header {
                height.blob = Some((H256::from(blob_hash.to_fixed_bytes()), data_proof));
                break;
            }
        }

        Ok(())
    }

    async fn send_tx(
        &self,
        tx: Payload<SubmitData>,
        signer: &PairSigner<AvailConfig, Pair>,
        client: &AvailClient,
    ) -> Result<(AvailH256, u32), Error> {
        let nonce = client
            .legacy_rpc()
            .system_account_next_index(signer.account_id())
            .await?;

        let e_event = client
            .tx()
            .create_signed_with_nonce(
                &tx,
                signer,
                nonce,
                avail_subxt::primitives::new_params_from_app_id(AvailAppID(self.app_id.0)),
            )?
            .submit_and_watch()
            .await
            .context("Submission failed")
            .unwrap()
            .wait_for_finalized_success()
            .await
            .context("Waiting for success failed")
            .unwrap();
        let block_hash = e_event.block_hash();
        let extrinsic_hash = e_event.extrinsic_index();
        Ok((block_hash, extrinsic_hash))
    }

    async fn get_inclusion_proof(
        &self,
        client: &AvailClient,
        transaction_index: u32,
        block_hash: AvailH256,
    ) -> Result<ProofResponse, Error> {
        let actual_proof = client
            .rpc_methods()
            .query_data_proof(transaction_index, block_hash)
            .await?;
        Ok(actual_proof)
    }

    pub async fn establish_a_connection() -> Result<AvailClient, Error> {
        let ws = String::from("ws://127.0.0.1:9944"); // TODO: update
        let client = AvailClient::new(ws).await?;
        Ok(client)
    }
}
