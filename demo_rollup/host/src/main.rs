use adapter_sdk::{
    adapter_zkvm::verify_proof,
    types::{AdapterPrivateInputs, AdapterPublicInputs},
};
use demo_rollup_core::{DemoProof, DemoRollupPublicInputs};
use methods::{ADAPTER_ELF, ADAPTER_ID};
use nexus_core::types::{AppAccountId, AppId, AvailHeader, Header, H256};
use risc0_zkvm::{default_prover, ExecutorEnv};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;

fn main() {
    let proof: DemoProof = DemoProof(());
    let rollup_pi: DemoRollupPublicInputs = DemoRollupPublicInputs {
        prev_state_root: H256::zero(),
        post_state_root: H256::from([1u8; 32]),
        blob_hash: H256::zero(),
    };
    let prev_adapter_public_inputs: Option<AdapterPublicInputs> = None;

    // Open the JSON file
    let mut file = File::open("header.json").expect("Unable to open file");

    // Read the contents of the file into a string
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Unable to read file");

    // Deserialize the JSON into a Header object
    let current_header: AvailHeader = AvailHeader::from(
        &serde_json::from_str::<Header>(&contents).expect("Unable to parse JSON"),
    );

    let private_inputs: AdapterPrivateInputs = AdapterPrivateInputs {
        header: current_header.clone(),
        avail_start_hash: current_header.hash(),
        app_id: AppAccountId::from(AppId(0)),
    };

    let env = ExecutorEnv::builder()
        .write(&proof)
        .unwrap()
        .write(&rollup_pi)
        .unwrap()
        .write(&prev_adapter_public_inputs)
        .unwrap()
        .write(&ADAPTER_ID)
        .unwrap()
        .write(&private_inputs)
        .unwrap()
        .write(&[0u8; 32])
        .unwrap()
        .build()
        .unwrap();

    let receipt = default_prover().prove(env, ADAPTER_ELF).unwrap();

    let new_adapter_pi: AdapterPublicInputs = receipt.journal.decode().unwrap();

    println!("First proof {:?}", new_adapter_pi);

    let new_rollup_pi: DemoRollupPublicInputs = DemoRollupPublicInputs {
        prev_state_root: H256::from([1u8; 32]),
        post_state_root: H256::from([2u8; 32]),
        blob_hash: H256::zero(),
    };
    let new_header = AvailHeader {
        parent_hash: current_header.hash(),
        number: current_header.number + 1,
        state_root: current_header.state_root,
        extrinsics_root: current_header.extrinsics_root,
        digest: current_header.digest,
        extension: current_header.extension,
    };

    let new_private_inputs = AdapterPrivateInputs {
        header: new_header.clone(),
        avail_start_hash: private_inputs.avail_start_hash.clone(),
        app_id: AppAccountId::from(AppId(0)),
    };

    let env = ExecutorEnv::builder()
        .add_assumption(receipt)
        .write(&proof)
        .unwrap()
        .write(&new_rollup_pi)
        .unwrap()
        .write(&Some(new_adapter_pi))
        .unwrap()
        .write(&ADAPTER_ID)
        .unwrap()
        .write(&new_private_inputs)
        .unwrap()
        .write(&[0u8; 32])
        .unwrap()
        .build()
        .unwrap();

    let receipt_2 = default_prover().prove(env, ADAPTER_ELF).unwrap();
    let latest_adapter_pi: AdapterPublicInputs = receipt_2.journal.decode().unwrap();

    println!(
        "Second proof {:?}, length: {:?}",
        latest_adapter_pi, receipt_2.inner
    );
}
