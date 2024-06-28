#![no_main]
use adapter_sdk::types::AdapterPublicInputs;
use risc0_zkvm::guest::env;

risc0_zkvm::guest::entry!(main);

fn main() {
    let adapter_public_inputs: AdapterPublicInputs = env::read();

    env::commit(&adapter_public_inputs);
}
