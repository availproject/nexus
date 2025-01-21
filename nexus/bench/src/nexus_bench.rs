use adapter_sdk::{api::NexusAPI, types::AdapterConfig};
use geth_methods::{ADAPTER_ELF, ADAPTER_ID};
use nexus_core::{
    db::NodeDB,
    state::vm_state::VmState,
    state_machine::StateMachine,
    types::{
        AppAccountId, AppId, AvailHeader, DataLookup, Digest, DigestItem, Extension, HeaderStore,
        InitAccount, KateCommitment, NexusHeader, NexusRollupPI, StatementDigest, SubmitProof,
        Transaction, TxParams, TxSignature, V3Extension, H256,
    },
    zkvm::ProverMode,
};
use nexus_host::execute_batch;
use rocksdb::Options;
use serde::{Deserialize, Serialize};
use serde_json::from_reader;
use std::env;
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;

#[cfg(feature = "risc0")]
use nexus_core::zkvm::risczero::{RiscZeroProof as Proof, RiscZeroProver as Prover, ZKVM};
#[cfg(feature = "sp1")]
use nexus_core::zkvm::sp1::{Sp1Proof as Proof, Sp1Prover as Prover, SP1ZKVM as ZKVM};

#[cfg(any(feature = "sp1"))]
use env_logger;

#[cfg(any(feature = "sp1"))]
use log;

#[derive(Clone, Serialize, Deserialize)]
struct AdapterStateData {
    last_height: u32,
    adapter_config: AdapterConfig,
}

//@TODO : use mockproofs for bench
fn create_mock_data() -> (
    Vec<Transaction>,
    StateMachine<ZKVM, Proof>,
    Vec<AvailHeader>,
    HeaderStore,
) {
    let _node_db: NodeDB = NodeDB::from_path(&String::from("./db/node_db"));
    let mut db_options = Options::default();
    db_options.create_if_missing(true);

    let state = Arc::new(Mutex::new(VmState::new(&String::from("./db/runtime_db"))));
    let mut txs: Vec<Transaction> = Vec::new();
    let state_machine = StateMachine::<ZKVM, Proof>::new(state.clone());

    // Read all file names from the transactions directory
    let dir_path = "src/transactions";
    let mut files = fs::read_dir(dir_path)
        .unwrap()
        .filter_map(|entry| {
            entry.ok().and_then(|e| {
                let path = e.path();
                if path.is_file() && path.extension().unwrap_or_default() == "json" {
                    path.file_name()
                        .map(|name| name.to_string_lossy().into_owned())
                } else {
                    None
                }
            })
        })
        .collect::<Vec<_>>();

    // Sort files to ensure consistent ordering
    files.sort();

    let num_txns = 300; // desired number of transactions
    let files_len = files.len(); // actual number of files available

    for tx in 0..num_txns {
        // Use modulo to loop back to start when we run out of files
        let file_index = tx % files_len;
        let file_path = format!("{}/{}", dir_path, files[file_index]);
        let tx_file = File::open(&file_path).unwrap();
        let tx_reader = BufReader::new(tx_file);
        let tx: Transaction = from_reader(tx_reader).unwrap();
        txs.push(tx);
    }

    let avail_header = File::open("src/avail_header.json").unwrap();
    let avail_header_reader = BufReader::new(avail_header);
    let avail_headers: Vec<AvailHeader> = from_reader(avail_header_reader).unwrap();

    let header_store = File::open("src/header_store.json").unwrap();
    let header_store_reader = BufReader::new(header_store);
    let header_store: HeaderStore = from_reader(header_store_reader).unwrap();

    (txs, state_machine, avail_headers, header_store)
}

async fn create_mock_transactions() {
    let (_, mut state_machine, avail_headers, mut header_store) = create_mock_data();
        let mock_txs: Vec<Transaction> = Vec::new();
        let prover_mode = ProverMode::NoAggregation;

        let (_, header, _, _) = execute_batch::<Prover, Proof, ZKVM>(
            &mock_txs,
            &mut state_machine,
            &avail_headers[0],
            &mut header_store,
            prover_mode.clone(),
        )
        .await
        .unwrap();

        let json_string = serde_json::to_string_pretty(&header).unwrap();
        fs::write("src/headers/nexus_header_1.json", json_string).unwrap();

        let nexus_header_1 = fs::read_to_string("src/headers/nexus_header_1.json").unwrap();

        // making 100 init transactions with diff app account id and store the results
        let header_1: NexusHeader = serde_json::from_str(&nexus_header_1).unwrap();
        println!("Nexus header 1 {:?}", header_1);
        let nexus_api = NexusAPI::new(&"http://127.0.0.1:7001");

        for txn_index in 0..100 {
            let tx = Transaction {
                signature: TxSignature([0u8; 64]),
                params: TxParams::InitAccount(InitAccount {
                    app_id: AppAccountId::from(AppId(txn_index as u32)),
                    statement: StatementDigest(ADAPTER_ID),
                    start_nexus_hash: header_1.hash(),
                }),
            };
            let json_string = serde_json::to_string_pretty(&tx).unwrap();
            let file_name = format!("src/init_account_transactions/init_account_txn_{}.json", txn_index);
            fs::write(file_name, json_string).unwrap();
            match nexus_api.send_tx(tx).await {
                Ok(i) => {
                    println!("Completed init account for app id {:?}", txn_index);
                }
                Err(e) => {
                    println!("Error when iniating account: {:?}", e);
                    continue;
                }
            }
        }

        let (_, header, _, _) = execute_batch::<Prover, Proof, ZKVM>(
            &mock_txs,
            &mut state_machine,
            &avail_headers[1],
            &mut header_store,
            prover_mode.clone(),
        )
        .await
        .unwrap();

        let json_string = serde_json::to_string_pretty(&header).unwrap();
        fs::write("src/headers/nexus_header_2.json", json_string).unwrap();

        let nexus_header_2 = fs::read_to_string("src/headers/nexus_header_2.json").unwrap();
        let header_2: NexusHeader = serde_json::from_str(&nexus_header_2).unwrap();
        println!("Nexus header 2 {:?}", header_2);

        // making 100 submit proof transactions
        let submit_proof_file = File::open("src/submit_proof_transactions/submit_proof_txn.json").unwrap();
        let submit_proof_txn_reader = BufReader::new(submit_proof_file);
        let submit_proof_txn: Transaction = from_reader(submit_proof_txn_reader).unwrap();

        let random_proof = match submit_proof_txn.params {
            TxParams::SubmitProof(proof) => proof.proof,
            _ => panic!("Invalid transaction"),
        };

        for txn_index in 0..100 {
            let public_inputs = NexusRollupPI {
                nexus_hash: header_2.hash(),
                state_root: H256::from(header_2.state_root.as_fixed_slice().clone()),
                //TODO: remove unwrap
                height: header_2.number,
                start_nexus_hash: H256::zero(), // for now
                app_id: AppAccountId::from(AppId(txn_index as u32)),
                img_id: StatementDigest(ADAPTER_ID),
                rollup_hash: Some(H256::zero()),
            };

            let tx = Transaction {
                signature: TxSignature([0u8; 64]),
                params: TxParams::SubmitProof(SubmitProof {
                    app_id: AppAccountId::from(AppId(txn_index as u32)),
                    nexus_hash: header_2.hash(),
                    state_root: H256::from(header_2.state_root.as_fixed_slice().clone()),
                    proof: random_proof.clone(), // need to generate the actual proof but for now using a random proof
                    height: header_2.number,
                    data: None,
                }),
            };

            let json_string = serde_json::to_string_pretty(&tx).unwrap();
            let file_name = format!("src/submit_proof_transactions/submit_proof_txn_{}.json", txn_index);
            fs::write(file_name, json_string).unwrap();

            match nexus_api.send_tx(tx).await {
                Ok(i) => {
                    println!(
                        "Submitted proof to update state root on nexus. Response: {:?} Stateroot: {:?}",
                        i, &public_inputs.state_root
                    )
                }
                Err(e) => {
                    println!("Error when iniating account: {:?}", e);

                    continue;
                }
            }
        }
}

#[tokio::main]
async fn main() {
    #[cfg(any(feature = "sp1"))]
    env_logger::Builder::from_env("RUST_LOG")
        .filter_level(log::LevelFilter::Info)
        .init();

    let prover_modes = vec![ProverMode::NoAggregation]; // for now only NoAggregation Mode
    let mut init_account_transactions: Vec<Transaction> = vec![];
    let mut submit_proof_transactions: Vec<Transaction> = vec![];

    for txn_index in 0..100 {
        let file_name = format!("src/submit_proof_transactions/submit_proof_txn_{}.json", txn_index);
        let file_content = fs::read_to_string(file_name).unwrap();
        let tx: Transaction = serde_json::from_str(&file_content).unwrap();
        init_account_transactions.push(tx);
    }

    for txn_index in 0..100 {
        let file_name = format!("src/init_account_transactions/init_account_txn_{}.json", txn_index);
        let file_content = fs::read_to_string(file_name).unwrap();
        let tx: Transaction = serde_json::from_str(&file_content).unwrap();
        submit_proof_transactions.push(tx);
    }

    for mode in 0..prover_modes.len() {
        let (_, mut state_machine, avail_headers, mut header_store) = create_mock_data();
        let prover_mode = &prover_modes[mode.clone()];

        {
            let start = Instant::now();
    
            let (proof, _, _, _) = execute_batch::<Prover, Proof, ZKVM>(
                &init_account_transactions,
                &mut state_machine,
                &avail_headers[0],
                &mut header_store,
                prover_mode.clone(),
            )
            .await
            .unwrap();
    
            let duration = start.elapsed();
            println!("Proof generation took: {:?}", duration);
    
            let current_dir = env::current_dir().unwrap();
            let mut out_sr_path = PathBuf::from(current_dir);
            #[cfg(feature = "risc0")]
            out_sr_path.push("succinct_receipt_risc0.bin");
    
            #[cfg(feature = "sp1")]
            out_sr_path.push("succinct_receipt_sp1.bin");
            let serialized_data = bincode::serialize(&proof).unwrap();
            let _ = fs::write(out_sr_path.clone(), serialized_data).unwrap();
    
            let metadata = fs::metadata(&out_sr_path).unwrap();
            let file_size = metadata.len();
            println!("Size of the binary file: {} bytes", file_size);
        }

        {
            let start = Instant::now();
    
            let (proof, header, tx_result, tree_update_batch) = execute_batch::<Prover, Proof, ZKVM>(
                &submit_proof_transactions,
                &mut state_machine,
                &avail_headers[0],
                &mut header_store,
                prover_mode.clone(),
            )
            .await
            .unwrap();
    
            let duration = start.elapsed();
            println!("Proof generation took: {:?}", duration);
    
            let current_dir = env::current_dir().unwrap();
            let mut out_sr_path = PathBuf::from(current_dir);
            #[cfg(feature = "risc0")]
            out_sr_path.push("succinct_receipt_risc0.bin");
    
            #[cfg(feature = "sp1")]
            out_sr_path.push("succinct_receipt_sp1.bin");
            let serialized_data = bincode::serialize(&proof).unwrap();
            let _ = fs::write(out_sr_path.clone(), serialized_data).unwrap();
    
            let metadata = fs::metadata(&out_sr_path).unwrap();
            let file_size = metadata.len();
            println!("Size of the binary file: {} bytes", file_size);
        }
    }
}
