#![no_main]
// If you want to try std support, also update the guest Cargo.toml file
// std support is experimental
use nexus_core::types::StatementDigest;
use risc0_zkvm::guest::env;
risc0_zkvm::guest::entry!(main);
use adapter_sdk::types::AdapterPublicInputs;
use methods::ADAPTER_ID;
use risc0_zkvm::serde;
use risc0_zkvm::sha::Digest;

fn main() {
    let start = env::cycle_count();
    eprintln!("Start cycle {}", start);
    let agg_public_inputs: Vec<AdapterPublicInputs> = env::read();

    agg_public_inputs.iter().map(|public_inputs| {
        env::verify(ADAPTER_ID, &serde::to_vec(&public_inputs).unwrap()).unwrap();
    });

    eprintln!("Current cycle count: {}", env::cycle_count());
    env::commit(&agg_public_inputs);
}
