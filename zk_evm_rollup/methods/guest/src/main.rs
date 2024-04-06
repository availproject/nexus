#![no_main]
// If you want to try std support, also update the guest Cargo.toml file
// std support is experimental

use adapter_sdk::adapter_zkvm::verify_proof;
use adapter_sdk::types::AdapterPrivateInputs;
use adapter_sdk::types::AdapterPublicInputs;
use adapter_sdk::types::RollupProof;
// use demo_rollup_core::DemoProof;
// use demo_rollup_core::DemoRollupPublicInputs;
use zk_evm_rollup_core::ZkEvmProof;
use zk_evm_rollup_core::ZkEvmRollupPublicInputs;
use nexus_core::types::StatementDigest;
use risc0_zkvm::guest::env;
use risc0_zkvm::sha::Digest;

use ark_bn254::{g1, g1::Parameters, Bn254, FqParameters, Fr, FrParameters, G1Projective};
use ark_ec::short_weierstrass_jacobian::GroupAffine;
use ark_ec::*;
use ark_ff::{Field, Fp256, Fp256Parameters, One, PrimeField, UniformRand, Zero};

pub type G1Point = <Bn254 as PairingEngine>::G1Affine;
pub type G2Point = <Bn254 as PairingEngine>::G2Affine;

risc0_zkvm::guest::entry!(main);


fn main() {
    let prev_adapter_public_inputs: Option<AdapterPublicInputs> = env::read();
    let proof: Option<RollupProof<ZkEvmRollupPublicInputs, ZkEvmProof>> = env::read();
    let private_inputs: AdapterPrivateInputs = env::read();
    let img_id: StatementDigest = env::read();
    let vk: [[u8; 32]; 6] = env::read();

    println!("here in guest main");
    let result = verify_proof(
        proof,
        prev_adapter_public_inputs,
        private_inputs,
        img_id,
        vk,
    )
    .unwrap();

    env::commit(&result);
}
