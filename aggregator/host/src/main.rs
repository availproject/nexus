use adapter_sdk::types::AdapterPublicInputs;
use aggregator_methods::{AGGREGATOR_ELF, AGGREGATOR_ID};
use bincode;
use nexus_core::types::{TxParamsV2, TransactionV2};
use risc0_zkvm::{default_prover, ExecutorEnv, InnerReceipt, Journal, Receipt};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;
use std::time::Instant;

fn main() {
    let mut file = File::open("submit_tx.json").expect("Unable to open file");

    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Unable to read file");

    let submit_tx: TransactionV2 = serde_json::from_str(&contents).expect("Unable to parse JSON");

    let submit_txs: Vec<TransactionV2> = vec![submit_tx.clone(), submit_tx];

    // temporary types needs discussion before finalizing it
    #[derive(Serialize, Deserialize, Clone)]
    struct ReceiptWithPublicInputs {
        receipt: Receipt,
        public_inputs: AdapterPublicInputs,
    }

    let receipts_with_public_inputs: Vec<ReceiptWithPublicInputs> = submit_txs
        .iter()
        .map(|tx| {
            let public_inputs = match &tx.params {
                TxParamsV2::SubmitProof(submit_proof) => Some(submit_proof.public_inputs.clone()).unwrap(),
                _ => panic!("Should have received a submit proof"),
            };

            let serialized_public_inputs = bincode::serialize(&public_inputs).unwrap();

            let receipt = Receipt {
                inner: InnerReceipt::Composite(tx.proof.clone().unwrap()),
                journal: Journal {
                    bytes: serialized_public_inputs,
                },
            };

            ReceiptWithPublicInputs {
                receipt,
                public_inputs,
            }
        })
        .collect();

    let agg_public_inputs = receipts_with_public_inputs.clone()
        .iter()
        .map(|receipt_with_public_inputs| receipt_with_public_inputs.clone().public_inputs)
        .collect::<Vec<AdapterPublicInputs>>();

    // for now we are aggregating two proofs
    let env = ExecutorEnv::builder()
        .add_assumption(receipts_with_public_inputs[0].clone().receipt)
        .add_assumption(receipts_with_public_inputs[1].clone().receipt) 
        .write(&agg_public_inputs)
        .unwrap()
        .build()
        .unwrap();

    let aggregated_receipt = default_prover().prove(env, AGGREGATOR_ELF).unwrap();
    println!("aggregated_receipt: {:?}", aggregated_receipt);

}
