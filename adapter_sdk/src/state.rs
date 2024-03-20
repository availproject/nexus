// track starting block of the rollup.
// track the last queried block of the rollup
// manage a basic data store for the proof generated with the following data: till_avail_block, proof, receipt
use crate::types::AdapterPublicInputs;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Proof(pub [u8; 32]);

// usage : create an object for this struct and use as a global dependency
#[derive(Debug, Deserialize, Serialize)]
pub struct AdapterState {
    pub starting_block_number: u8,
    pub last_queried_block_number: u8,
    pub public_inputs: AdapterPublicInputs,
    queue: VecDeque<Proof>,
}

impl AdapterState {
    // queries avail block to search for data related to the given app_id
    pub fn query_avail_blocks(&self) {}

    // function to generate proof against avail data when proof is received and verified from the rollup
    fn verify_and_generate_proof(&self) {}

    // function to store the till_avail_block, and the corresponding adapter proof generated in local storage
    fn store_local_state(&self) {}
}
