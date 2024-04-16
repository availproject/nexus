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
use zk_evm_adapter_host::fetcher::{fetch_proof_and_pub_signal, get_vk_fflonk};
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

use zk_evm_adapter_host::utils::*;

// #[tokio::main]
fn main() {
    let mut adapter: AdapterState<ZkEvmProof, ZkEvmVerificationKey> = AdapterState::new(
        String::from("adapter_store"),
        AdapterConfig {
            app_id: AppId(100),
            elf: ADAPTER_ELF.to_vec(),
            adapter_elf_id: StatementDigest(ADAPTER_ID),
            vk: get_vk_fflonk(),
            rollup_start_height: 606460,
        },
    );

    let rt = tokio::runtime::Runtime::new().unwrap();
    let mut cloned_adapter = adapter.clone();
   
    let th: tokio::task::JoinHandle<()> = rt.spawn(async move {
        cloned_adapter.run().await;
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

    let filter = Filter::new().address(vec![POLYGON_ZKEVM_PROXY]);

    let mut logs = rt.block_on(async {
        let log = provider.subscribe_logs(&filter).await.unwrap();
        log
    });

    while let Some(mut txn_hash) = rt.block_on(async {
        let hash = logs.next().await.unwrap().transaction_hash;
        hash
    }) {
        
        let tx: Transaction = rt.block_on(async {
            let tx: Transaction = http_provider
                .get_transaction(txn_hash)
                .await
                .unwrap()
                .unwrap();
            tx
        });

        let mut proof_fetched: ZkEvmProof = ZkEvmProof::default();

        let mut pubSig_fetched: H256 = [0u8; 32].into();

        if tx.to.unwrap() == POLYGON_ZKEVM_PROXY {

            let _proof = rt.block_on(async {
                let proof = fetch_proof_and_pub_signal(txn_hash).await;
                proof
            });

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
                //TODO: Change this to the actual blob hash
                blob_hash: [2u8; 32].into(),
            },
        }).unwrap();
        rt.block_on(adapter.add_proof(proof.clone()));
    }

    rt.block_on( async move {
        th.await.unwrap();
    }
    );


}
