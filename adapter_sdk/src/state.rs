use crate::adapter_zkvm::verify_proof;
// track starting block of the rollup.
// track the last queried block of the rollup
// manage a basic data store for the proof generated with the following data: till_avail_block, proof, receipt

use crate::types::{AdapterPrivateInputs, AdapterPublicInputs, RollupProof};
use anyhow::{anyhow, Error};
use avail_subxt::api::identity::calls::types::SetFee;
use nexus_core::traits::{Proof, RollupPublicInputs};
use nexus_core::types::H256;
use risc0_zkp::core::digest::Digest;
use risc0_zkvm::{default_prover, Receipt};
use risc0_zkvm::{
    serde::{from_slice, to_vec},
    Executor, ExecutorEnv,
};
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
    pub(crate) proof_queue: Arc<Mutex<VecDeque<RollupProof<PI, P>>>>,
    pub blob_data: Arc<Mutex<VecDeque<H256>>>,
    pub elf: Box<[u8]>,
    pub elf_id: Digest,
    pub vk: [u8; 32],
}

impl<PI: RollupPublicInputs, P: Proof<PI>> AdapterState<PI, P> {
    pub fn new(
        public_inputs: AdapterPublicInputs,
        private_inputs: AdapterPrivateInputs,
        vk: [u8; 32],
        zkvm_elf: &[u8],
        zkvm_id: impl Into<Digest>,
    ) -> Self {
        AdapterState {
            starting_block_number: 0,
            last_queried_block_number: 0,
            public_inputs,
            private_inputs,
            proof_queue: Arc::new(Mutex::new(VecDeque::new())),
            blob_data: Arc::new(Mutex::new(VecDeque::new())),
            elf: zkvm_elf.into(),
            elf_id: zkvm_id.into(),
            vk,
        }
    }

    // function triggered by rollup in a loop to pro+cess its proofs.
    pub async fn process_queue(&mut self, rollup_public_inputs: PI) {
        // self.verify_and_generate_proof(rollup_public_inputs).await
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

    pub async fn add_proof(&mut self, proof: RollupProof<PI, P>) {
        // proof input validation ???
        self.proof_queue.lock().await.push_back(proof)
    }

    // function to generate proof against avail data when proof is received and verified from the rollup
    pub async fn verify_and_generate_proof(
        &mut self,
        rollup_public_inputs: PI,
    ) -> Result<Receipt, Error> {
        let proof_lock = self.proof_queue.lock().await;
        let front_proof_ref = proof_lock.front().expect("Queue is empty");
        let front_proof_clone = front_proof_ref.clone(); // Clone the value

        let env = ExecutorEnv::builder()
            .write(&to_vec(&front_proof_clone)?)
            .unwrap()
            .write(&to_vec(&rollup_public_inputs)?)
            .unwrap()
            .write(&to_vec(&self.public_inputs)?)
            .unwrap()
            .write(&to_vec(&self.private_inputs)?)
            .unwrap()
            .write(&to_vec(&self.public_inputs.img_id)?)
            .unwrap()
            .write(&to_vec(&self.vk)?)
            .unwrap()
            .build()
            .unwrap();

        let prover = default_prover();

        let receipt = prover.prove(env, &self.elf);

        receipt
        // let new_public_inputs = verify_proof(
        //     front_proof_clone,
        //     rollup_public_inputs,
        //     Some(self.public_inputs.clone()),
        //     self.private_inputs.clone(),
        //     self.public_inputs.img_id,
        //     self.vk,
        // );
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
