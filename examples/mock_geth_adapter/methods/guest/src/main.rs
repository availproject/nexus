#![no_main]
use adapter_sdk::types::AdapterPublicInputs;
use nexus_core::zkvm::{risczero::ZKVM, traits::ZKVMEnv};

risc0_zkvm::guest::entry!(main);

fn main() {
    run::<ZKVM>();
}

fn run<Z: ZKVMEnv>() {
    let adapter_public_inputs = Z::read_input::<AdapterPublicInputs>().expect("msg");
    Z::commit(&adapter_public_inputs);
}
