use ark_bn254::{Fr, FrParameters};
use ethabi::{ParamType, Token};
use ethers::contract::{abigen, Contract};
use ethers::prelude::*;
use ethers::utils::hex;
use num_bigint::*;
use queues::*;
use sha256::digest;
use zk_evm_rollup_core::ZkEvmVerificationKey;
use std::convert::TryFrom;
use std::str::FromStr;
use std::sync::Arc;


use crate::utils::*;

#[derive(Debug, Clone)]
pub struct ProofValue {
    pub proof: Vec<String>,
    pub pub_signal: String,
}

pub async fn fetch_proof_and_pub_signal(
    txn_hash: H256,
) -> (Vec<String>, String, u32, [u8; 32], [u8; 32]) {
    let provider = Provider::<Ws>::connect(
        "wss://eth-mainnet.ws.alchemyapi.io/v2/nrzrNIfp7oG61YHAmoPAICuibjwqeHmN",
    )
    .await
    .unwrap();

    let http_provider = Provider::<Http>::try_from(
        "https://eth-mainnet.ws.alchemyapi.io/v2/nrzrNIfp7oG61YHAmoPAICuibjwqeHmN",
    )
    .unwrap();

    abigen!(
        PolygonZkevmRollupManagerContract,
        r#"[
            {
                "type": "function",
                "name": "getInputSnarkBytes",
                "inputs": [
                    {
                        "name": "rollupID",
                        "type": "uint32"
                    },
                    {
                        "name": "initNumBatch",
                        "type": "uint64"
                    },
                    {
                        "name": "finalNewBatch",
                        "type": "uint64"
                    },
                    {
                        "name": "newLocalExitRoot",
                        "type": "bytes32"
                    },
                    {
                        "name": "oldStateRoot",
                        "type": "bytes32"
                    },
                    {
                        "name": "newStateRoot",
                        "type": "bytes32"
                    }
                ],
                "outputs": [
                    {
                        "name": "",
                        "type": "bytes"
                    }
                ]
            },
            {
                "type": "function",
                "name": "getRollupBatchNumToStateRoot",
                "inputs": [
                    {
                        "name": "rollupID",
                        "type": "uint32"
                    },
                    {
                        "name": "batchNum",
                        "type": "uint64"
                    }
                ],
                "outputs": [
                    {
                        "name": "",
                        "type": "bytes32"
                    }
                ]
            }
        ]"#,
    );

    let POLYGON_ZKEVM_PROXY: Address = "0x5132A183E9F3CB7C848b0AAC5Ae0c4f0491B7aB2"
        .parse()
        .expect("Invalid contract address");

    let contract = PolygonZkevmRollupManagerContract::new(
        POLYGON_ZKEVM_PROXY,
        Arc::new(http_provider.clone()),
    );

    let filter = Filter::new().address(vec![POLYGON_ZKEVM_PROXY]);

    let mut logs = provider.subscribe_logs(&filter).await.unwrap();

    println!("Listening for the proofs...");

    let tx: Transaction = http_provider
        .get_transaction(txn_hash)
        .await
        .unwrap()
        .unwrap();

    println!("Transaction to : {:?}", tx.to);
    println!("Transaction data: {:?}", tx.input);

    let mut txn_call_data = tx.input.to_string();
    // slice first 9 element of txn_call_data string
    txn_call_data = txn_call_data[10..].to_string(); // slice method id
                                                     // let txn_byte = hex::decode(txn_call_data).unwrap();

    let chunks: Vec<String> = txn_call_data
        .as_bytes()
        .chunks(64)
        .map(|chunk| String::from_utf8_lossy(chunk).into_owned())
        .collect();

    // Display the resulting array of strings
    for chunk in chunks.iter() {
        println!("{}", chunk);
    }

    let rollup_id = chunks[0].parse::<u32>().unwrap();
    let pending_state_num = chunks[1].parse::<u64>().unwrap();
    let init_num_batch = u64::from_str_radix(&chunks[2].trim_start_matches('0'), 16).unwrap();
    let final_new_batch = u64::from_str_radix(&chunks[3].trim_start_matches('0'), 16).unwrap();

    let mut beneficiary = chunks[6].to_string();
    beneficiary = beneficiary[24..].to_string();

    println!("Rollup ID: {:?}", rollup_id);
    println!("Pending State Num: {:?}", pending_state_num);
    println!("Init Num Batch: {:?}", init_num_batch);
    println!("Final New Batch: {:?}", final_new_batch);

    let resp_old_state_root = contract
        .get_rollup_batch_num_to_state_root(rollup_id, init_num_batch)
        .await
        .unwrap();

    let mut old_state_root = hex::encode(&resp_old_state_root);
    println!("State Root: {:?}", old_state_root);

    //add "0x"
    old_state_root = format!("0x{}", old_state_root).to_string();

    // rest are proof chunks

    let new_local_exist_root = hex::decode(chunks[4].clone()).unwrap();
    let new_state_root = hex::decode(chunks[5].clone()).unwrap();

    let nle: [u8; 32] = new_local_exist_root.try_into().unwrap();
    let nsr: [u8; 32] = new_state_root.try_into().unwrap();

    let snark_hash_bytes: Bytes = contract
        .get_input_snark_bytes(
            rollup_id,
            init_num_batch,
            final_new_batch,
            nle,
            resp_old_state_root,
            nsr,
        )
        .await
        .unwrap();

    // getting the pub signal
    println!("Pub Signal: {:?}", snark_hash_bytes);

    let mut snark_hash_string = snark_hash_bytes.to_string();
    snark_hash_string = snark_hash_string[42..].to_string();

    // concat hash string with beneficairy string
    snark_hash_string = format!("{}{}", beneficiary, snark_hash_string);
    // snark_hash_string = format!("{}{}", "0x", snark_hash_string);

    println!("Snark Hash String: {:?}", snark_hash_string);

    let sha_snark_hash_string = digest(hex::decode(snark_hash_string).unwrap());

    let uint_snark_hash =
        U256::from_str(sha_snark_hash_string.as_str()).expect("Invalid hex value");
    println!("U256 value: {}", uint_snark_hash);

    let pub_signal = Fr::from_str(uint_snark_hash.to_string().as_str()).unwrap();

    println!("Public Signal: {:?}", pub_signal.to_string());

    let mut proof_values: Vec<String> = Vec::new();
    // let mut proof_values_ref: Vec<&str> = Vec::new();

    for i in 7..chunks.len() {
        let big_int_val = U256::from_str(chunks[i].as_str()).expect("Invalid hex value");
        let val_str = big_int_val.to_string();
        proof_values.push(val_str);
    }

    // proof_values_ref = proof_values.iter().map(|s| s.as_str()).collect();

    (
        proof_values,
        uint_snark_hash.to_string(),
        rollup_id,
        nle,
        nsr,
    )
}

pub fn get_vk_fflonk() -> ZkEvmVerificationKey {
    let vk: [[u8; 32]; 6] = [
        get_u8_arr_from_str("7005013949998269612234996630658580519456097203281734268590713858661772481668"),
        get_u8_arr_from_str("869093939501355406318588453775243436758538662501260653214950591532352435323"),
        get_u8_arr_from_str("21831381940315734285607113342023901060522397560371972897001948545212302161822"),
        get_u8_arr_from_str("17231025384763736816414546592865244497437017442647097510447326538965263639101"),
        get_u8_arr_from_str("2388026358213174446665280700919698872609886601280537296205114254867301080648"),
        get_u8_arr_from_str("11507326595632554467052522095592665270651932854513688777769618397986436103170"),
    ];
    ZkEvmVerificationKey{
        vk
    }
}