#![no_main]
// If you want to try std support, also update the guest Cargo.toml file
// std support is experimental

use adapter_sdk::types::AdapterPrivateInputs;
use adapter_sdk::types::AdapterPublicInputs;
use demo_rollup_core::DemoRollupPublicInputs;
use risc0_zkvm::guest::env;
risc0_zkvm::guest::entry!(main);
use adapter_sdk::adapter_zkvm::verify_proof;
use demo_rollup_core::DemoProof;
use risc0_zkvm::sha::Digest;

fn main() {
    let start = env::cycle_count();
    eprintln!("Start cycle {}", start);

    let proof: DemoProof = env::read();
    let rollup_public_inputs: DemoRollupPublicInputs = env::read();
    let prev_adapter_public_inputs: Option<AdapterPublicInputs> = env::read();
    let img_id: Digest = env::read();
    let private_inputs: AdapterPrivateInputs = env::read();
    let vk: [u8; 32] = env::read();

    let result = verify_proof(
        proof,
        rollup_public_inputs,
        prev_adapter_public_inputs,
        private_inputs,
        img_id,
        vk,
    )
    .unwrap();

    eprintln!("Current cycle count: {}", env::cycle_count());
    env::commit(&result);
}
