use adapter_sdk::{
    adapter_zkvm::verify_proof,
    state::AdapterState,
    types::{AdapterConfig, AdapterPrivateInputs, AdapterPublicInputs, RollupProof},
};
// use demo_rollup_core::{DemoProof, DemoRollupPublicInputs};
use zk_evm_rollup_core::{ZkEvmProof, ZkEvmRollupPublicInputs, ZkEvmVerificationKey};
use zk_evm_methods::{ADAPTER_ELF, ADAPTER_ID};
use nexus_core::types::{
    AppAccountId, AppId, AvailHeader, Header, StatementDigest, SubmitProof, TransactionV2,
    TxParamsV2, TxSignature, H256,
};
use risc0_zkvm::{default_prover, ExecutorEnv, InnerReceipt};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;
use std::time::Instant;

use ark_bn254::{
    g1, g1::Parameters, Bn254, Fq, FqParameters, Fr, FrParameters, G1Projective, G2Projective,
};
use ark_bn254::{g2, Fq2, Fq2Parameters, G2Affine};
use ark_ec::short_weierstrass_jacobian::GroupAffine;
use ark_ec::*;
use ark_ff::{
    field_new, Field, Fp256, Fp256Parameters, Fp2ParamsWrapper, One, PrimeField, QuadExtField,
    UniformRand, Zero,
};

use std::fmt::{format, Debug, DebugMap, Display};
use std::hash::Hash;
use std::marker::PhantomData;
use std::ops::{Add, Mul, Neg, Sub};
use std::str::FromStr;
use std::vec;
use num_bigint::*;
use num_bigint::BigUint;


pub fn get_bigint_from_fr(fr: Fp256<FrParameters>) -> BigInt {
    let mut st = fr.to_string();
    let temp = &st[8..8 + 64];
    BigInt::parse_bytes(temp.as_bytes(), 16).unwrap()
}


pub fn get_u8_arr_from_str(num_str: &str) -> [u8; 32] {
    //convert bigint to [u8; 32]
    let bigint = BigInt::parse_bytes(num_str.as_bytes(), 10).unwrap();

    // Get the bytes from the BigInt
    let bytes = bigint.to_bytes_le().1;

    // Convert the bytes to a fixed size array
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&bytes);

    arr
}

fn main() {
    let mut adapter: AdapterState<ZkEvmRollupPublicInputs, ZkEvmProof> = AdapterState::new(
        String::from("adapter_store"),
        AdapterConfig {
            app_id: AppId(100),
            elf: ADAPTER_ELF.to_vec(),
            adapter_elf_id: StatementDigest(ADAPTER_ID),
            vk: [0u8; 32],
            rollup_start_height: 606460,
        },
    );
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    // rt.block_on(adapter.run());

    let proof_: ZkEvmProof = ZkEvmProof{
        c1_x: [1u64; 32],
        c1_y: [1u64; 32],
        c2_x: [1u64; 32],
        c2_y: [1u64; 32],
        w1_x: [1u64; 32],
        w1_y: [1u64; 32],
        w2_x: [1u64; 32],
        w2_y: [1u64; 32],
        eval_ql: [1u64; 32],
        eval_qr: [1u64; 32],
        eval_qm: [1u64; 32],
        eval_qo: [1u64; 32],
        eval_qc: [1u64; 32],
        eval_s1: [1u64; 32],
        eval_s2: [1u64; 32],
        eval_s3: [1u64; 32],
        eval_a: [1u64; 32],
        eval_b: [1u64; 32],
        eval_c: [1u64; 32],
        eval_z: [1u64; 32],
        eval_zw: [1u64; 32],
        eval_t1w: [1u64; 32],
        eval_t2w: [1u64; 32],
        eval_inv: [1u64; 32],

    };

    let proof = Some(RollupProof {
        proof: proof_,
        public_inputs: ZkEvmRollupPublicInputs {
            prev_state_root: [0u8; 32].into(),
            post_state_root: [0u8; 32].into(),
            blob_hash: [0u8; 32].into(),
        },
    });


// proof: Option<RollupProof<ZkEvmRollupPublicInputs, ZkEvmProof>>

    // let rollup_pi: DemoRollupPublicInputs = DemoRollupPublicInputs {
    //     prev_state_root: H256::zero(),
    //     post_state_root: H256::from([1u8; 32]),
    //     blob_hash: H256::zero(),
    // };
    let prev_adapter_public_inputs: Option<AdapterPublicInputs> = None;

    // //1. Inclusion proof to the blob root of that block. (Which is maintained by adapter when it finds blob in an Avail Block.)
    // //So this does not have completeness check.
    println!("Reading header.json");
    // // Open the JSON file
    let mut file = File::open("header.json").expect("Unable to open file");

    // // Read the contents of the file into a string
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Unable to read file");

    // // Deserialize the JSON into a Header object
    let current_header: AvailHeader = AvailHeader::from(
        &serde_json::from_str::<Header>(&contents).expect("Unable to parse JSON"),
    );

    let private_inputs: AdapterPrivateInputs = AdapterPrivateInputs {
        header: current_header.clone(),
        app_id: AppId(0),
    };
    println!("priv innputs prepared");

    let vk: [[u8; 32]; 6] = [
        get_u8_arr_from_str("7005013949998269612234996630658580519456097203281734268590713858661772481668"),
        get_u8_arr_from_str("869093939501355406318588453775243436758538662501260653214950591532352435323"),
        get_u8_arr_from_str("21831381940315734285607113342023901060522397560371972897001948545212302161822"),
        get_u8_arr_from_str("17231025384763736816414546592865244497437017442647097510447326538965263639101"),
        get_u8_arr_from_str("2388026358213174446665280700919698872609886601280537296205114254867301080648"),
        get_u8_arr_from_str("11507326595632554467052522095592665270651932854513688777769618397986436103170"),
    ];
    println!("vk prepared");

    let env = ExecutorEnv::builder()
        .write(&prev_adapter_public_inputs)
        .unwrap()
        .write(&proof)
        .unwrap()
        .write(&private_inputs)
        .unwrap()
        .write(&ADAPTER_ID)
        .unwrap()
        .write(&vk)
        .unwrap()
        .build()
        .unwrap();

    println!("sending proof to zkvm");

    // Measure time taken for the first proof
    let start_time_first_proof = Instant::now();
    let receipt = default_prover().prove(env, ADAPTER_ELF).unwrap();
    let end_time_first_proof = Instant::now();
    let time_taken_first_proof = end_time_first_proof.duration_since(start_time_first_proof);
    eprintln!("Time taken for first proof: {:?}", time_taken_first_proof);

    // let new_adapter_pi: AdapterPublicInputs = receipt.journal.decode().unwrap();

    // println!("First proof {:?}", new_adapter_pi);

    // let proof = match receipt.inner {
    //     InnerReceipt::Composite(i) => i,
    //     _ => panic!("Should have received a composite proof"),
    // };

    // let submit_tx: TransactionV2 = TransactionV2 {
    //     params: TxParamsV2::SubmitProof(SubmitProof {
    //         public_inputs: new_adapter_pi,
    //     }),
    //     signature: TxSignature([0u8; 64]),
    //     proof: Some(proof),
    // };

    // let json_data = serde_json::to_string_pretty(&submit_tx).expect("Serialization failed");

    // // Write to file
    // std::fs::write("submit_tx.json", json_data).expect("Failed to write to file");

    // // let new_rollup_pi: DemoRollupPublicInputs = DemoRollupPublicInputs {
    // //     prev_state_root: H256::from([1u8; 32]),
    // //     post_state_root: H256::from([2u8; 32]),
    // //     blob_hash: H256::zero(),
    // // };
    // // let new_header = AvailHeader {
    // //     parent_hash: current_header.hash(),
    // //     number: current_header.number + 1,
    // //     state_root: current_header.state_root,
    // //     extrinsics_root: current_header.extrinsics_root,
    // //     digest: current_header.digest,
    // //     extension: current_header.extension,
    // // };

    // // let new_private_inputs = AdapterPrivateInputs {
    // //     header: new_header.clone(),
    // //     avail_start_hash: private_inputs.avail_start_hash.clone(),
    // //     app_id: AppAccountId::from(AppId(0)),
    // // };

    // // let env = ExecutorEnv::builder()
    // //     .add_assumption(receipt)
    // //     .write(&proof)
    // //     .unwrap()
    // //     .write(&new_rollup_pi)
    // //     .unwrap()
    // //     .write(&Some(new_adapter_pi))
    // //     .unwrap()
    // //     .write(&ADAPTER_ID)
    // //     .unwrap()
    // //     .write(&new_private_inputs)
    // //     .unwrap()
    // //     .write(&[0u8; 32])
    // //     .unwrap()
    // //     .build()
    // //     .unwrap();

    // // // Measure time taken for the second proof
    // // let start_time_second_proof = Instant::now();
    // // let receipt_2 = default_prover().prove(env, ADAPTER_ELF).unwrap();
    // // let end_time_second_proof = Instant::now();
    // // let time_taken_second_proof = end_time_second_proof.duration_since(start_time_second_proof);
    // // println!("Time taken for second proof: {:?}", time_taken_second_proof);

    // // let latest_adapter_pi: AdapterPublicInputs = receipt_2.journal.decode().unwrap();

    // // println!("Second proof {:?}", latest_adapter_pi);
}
