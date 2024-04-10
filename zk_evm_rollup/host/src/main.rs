use adapter_sdk::{
    adapter_zkvm::verify_proof,
    state::AdapterState,
    types::{AdapterConfig, AdapterPrivateInputs, AdapterPublicInputs, RollupProof, RollupPublicInputs},
};
use websocket::client::r#async;
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
use std::sync::{Arc, Mutex};
use std::time::Duration;


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

// use ethabi::{ParamType, Token};
use ethers::contract::{abigen, Contract};
use ethers::prelude::*;
use ethers::utils::hex;
use zk_evm_adapter_host::fetcher::fetch_proof_and_pub_signal;
use num_bigint::*;
use ethers::types::H256 as EthH256;

use queues::*;
use sha256::digest;
use std::convert::{self, TryFrom};
use std::str::FromStr;
// use std::sync::Arc;
use std::{thread::sleep, time, thread};
use tokio;
use tokio::runtime::Runtime;
use tokio::task;


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
    
    let num_bytes = bytes.len().min(32);
    arr[..num_bytes].copy_from_slice(&bytes[..num_bytes]);
    // arr.copy_from_slice(&bytes);

    arr
}

pub fn get_u8_arr_from_fr(fr: Fp256<FrParameters>) -> [u8; 32] {
    let bytes = get_bigint_from_fr(fr).to_bytes_le().1;

    // Convert the bytes to a fixed size array
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&bytes);

    arr
}
// #[tokio::main]
fn main() {
    let mut adapter: AdapterState<ZkEvmProof> = AdapterState::new(
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
    let mut cloned_adapter = adapter.clone();
   
    let th: tokio::task::JoinHandle<()> = rt.spawn(async move {
        cloned_adapter.run().await;
    });

    rt.block_on(async move{
        tokio::time::sleep(Duration::from_secs(3)).await;
    });

    // let rt = tokio::runtime::Runtime::new().unwrap();
    let POLYGON_ZKEVM_PROXY: Address = "0x5132A183E9F3CB7C848b0AAC5Ae0c4f0491B7aB2"
        .parse()
        .expect("Invalid contract address");

    let provider = rt.block_on(async {
        let provider = Provider::<Ws>::connect(
            "wss://eth-mainnet.ws.alchemyapi.io/v2/nrzrNIfp7oG61YHAmoPAICuibjwqeHmN",
        )
        .await
        .unwrap();
        provider
    });

    let http_provider = Provider::<Http>::try_from(
        "https://eth-mainnet.ws.alchemyapi.io/v2/nrzrNIfp7oG61YHAmoPAICuibjwqeHmN",
    )
    .unwrap();

    let mut proof_queues: Queue<ZkEvmProof> = queue![];

    let filter = Filter::new().address(vec![POLYGON_ZKEVM_PROXY]);

    let mut logs = rt.block_on(async {
        let log = provider.subscribe_logs(&filter).await.unwrap();
        log
    });

    println!("Henosis Proof Aggregator Listening for Proofs!!");

    let sample_hash =
        EthH256::from_str("0xed0c28abb022be570305ae3cd454c5c3bb027ede55cfdefe6744bc1b5af90d8a").unwrap();
    let txn_hash = sample_hash;
    // let get_txn_handle = tokio::spawn(http_provider.clone().get_transaction(sample_hash));

    // let tx: Transaction = get_txn_handle.await.unwrap().unwrap().unwrap();
    let tx: Transaction = rt.block_on(async {
        let tx: Transaction = http_provider
            .get_transaction(txn_hash)
            .await
            .unwrap()
            .unwrap();
        tx
    });

    let mut proof_fetched: ZkEvmProof = ZkEvmProof {
        c1_x: [0u8; 32],
        c1_y: [0u8; 32],
        c2_x: [0u8; 32],
        c2_y: [0u8; 32],
        w1_x: [0u8; 32],
        w1_y: [0u8; 32],
        w2_x: [0u8; 32],
        w2_y: [0u8; 32],
        eval_ql: [0u8; 32],
        eval_qr: [0u8; 32],
        eval_qm: [0u8; 32],
        eval_qo: [0u8; 32],
        eval_qc: [0u8; 32],
        eval_s1: [0u8; 32],
        eval_s2: [0u8; 32],
        eval_s3: [0u8; 32],
        eval_a: [0u8; 32],
        eval_b: [0u8; 32],
        eval_c: [0u8; 32],
        eval_z: [0u8; 32],
        eval_zw: [0u8; 32],
        eval_t1w: [0u8; 32],
        eval_t2w: [0u8; 32],
        eval_inv: [0u8; 32],
        pub_signal: [0u8; 32].into(),
    };

    let mut pubSig_fetched: H256 = [0u8; 32].into();

    if tx.to.unwrap() == POLYGON_ZKEVM_PROXY {

        let _proof = rt.block_on(async {
            let proof = fetch_proof_and_pub_signal(txn_hash).await;
            proof
        });

        println!("Proof: {:?}", _proof);
        
        // let _ = proof_queues.add(ProofValue {
        //     proof: _proof.0,
        //     pub_signal: _proof.1,
        // });
        println!("Transaction: {:?}", tx);

        proof_fetched = ZkEvmProof{
            c1_x: get_u8_arr_from_str(_proof.0[0].as_str()),
            c1_y: get_u8_arr_from_str(_proof.0[1].as_str()),
            c2_x: get_u8_arr_from_str(_proof.0[2].as_str()),
            c2_y: get_u8_arr_from_str(_proof.0[3].as_str()),
            w1_x: get_u8_arr_from_str(_proof.0[4].as_str()),
            w1_y: get_u8_arr_from_str(_proof.0[5].as_str()),
            w2_x: get_u8_arr_from_str(_proof.0[6].as_str()),
            w2_y: get_u8_arr_from_str(_proof.0[7].as_str()),
            eval_ql: get_u8_arr_from_str(_proof.0[8].as_str()),
            eval_qr: get_u8_arr_from_str(_proof.0[9].as_str()),
            eval_qm: get_u8_arr_from_str(_proof.0[10].as_str()),
            eval_qo: get_u8_arr_from_str(_proof.0[11].as_str()),
            eval_qc: get_u8_arr_from_str(_proof.0[12].as_str()),
            eval_s1: get_u8_arr_from_str(_proof.0[13].as_str()),
            eval_s2: get_u8_arr_from_str(_proof.0[14].as_str()),
            eval_s3: get_u8_arr_from_str(_proof.0[15].as_str()),
            eval_a: get_u8_arr_from_str(_proof.0[16].as_str()),
            eval_b: get_u8_arr_from_str(_proof.0[17].as_str()),
            eval_c: get_u8_arr_from_str(_proof.0[18].as_str()),
            eval_z: get_u8_arr_from_str(_proof.0[19].as_str()),
            eval_zw: get_u8_arr_from_str(_proof.0[20].as_str()),
            eval_t1w: get_u8_arr_from_str(_proof.0[21].as_str()),
            eval_t2w: get_u8_arr_from_str(_proof.0[22].as_str()),
            eval_inv: get_u8_arr_from_str(_proof.0[23].as_str()),
            pub_signal: get_u8_arr_from_fr(Fr::from_str(&_proof.1.to_string()).unwrap()).into()
        };

        println!("pub signal send {:?}",_proof.1.to_string());
        // pubSig_fetched = get_u8_arr_from_fr(Fr::from_str(&_proof.1.to_string()).unwrap()).into()
    }

    let proof = Some(RollupProof {
        proof: proof_fetched,
        public_inputs: RollupPublicInputs {
            prev_state_root: [0u8; 32].into(),
            post_state_root: [0u8; 32].into(),
            blob_hash: [2u8; 32].into(),
        },
    }).unwrap();
    rt.block_on(adapter.add_proof(proof.clone()));

    rt.block_on( async move {
        th.await.unwrap();
    }
    );



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
