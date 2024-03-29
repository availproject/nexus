#![no_main]
// If you want to try std support, also update the guest Cargo.toml file
// std support is experimental

use adapter_sdk::adapter_zkvm::verify_proof;
use adapter_sdk::types::AdapterPrivateInputs;
use adapter_sdk::types::AdapterPublicInputs;
use adapter_sdk::types::RollupProof;
use demo_rollup_core::DemoProof;
use demo_rollup_core::DemoRollupPublicInputs;
use nexus_core::types::StatementDigest;
use risc0_zkvm::guest::env;
use risc0_zkvm::sha::Digest;

risc0_zkvm::guest::entry!(main);

fn main() {
    let start = env::cycle_count();
    eprintln!("Start cycle {}", start);
    let prev_adapter_public_inputs: Option<AdapterPublicInputs> = env::read();
    let proof: Option<RollupProof<DemoRollupPublicInputs, DemoProof>> = env::read();
    let private_inputs: AdapterPrivateInputs = env::read();
    let img_id: StatementDigest = env::read();
    let vk: [u8; 32] = env::read();

    let result = verify_proof(
        proof,
        prev_adapter_public_inputs,
        private_inputs,
        img_id,
        vk,
    )
    .unwrap();

    eprintln!("Current cycle count: {}", env::cycle_count());
    env::commit(&result);
}
