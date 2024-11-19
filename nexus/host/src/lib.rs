use anyhow::{anyhow, Context, Error};
 
use nexus_core::{
    agg_types::{AggregatedTransaction, InitTransaction, SubmitProofTransaction},
    db::NodeDB,
    mempool::Mempool,
    state::{merkle_store, MerkleStore, VmState},
    state_machine::StateMachine,
    types::{
        AvailHeader, HeaderStore, NexusHeader, Proof as NexusProof, RollupPublicInputsV2,
        TransactionV2, TransactionZKVM, TxParamsV2, H256,
    },
    zkvm::{
        traits::{ZKVMEnv, ZKVMProof, ZKVMProver},
        ProverMode,
    },
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AvailToNexusPointer {
    pub number: u32,
    pub nexus_hash: H256,
}
 
#[cfg(any(feature = "risc0"))]
use prover::{NEXUS_RUNTIME_ELF, NEXUS_RUNTIME_ID};
 
use serde::{Deserialize, Serialize};
 
use std::net::SocketAddr;
use std::str::FromStr;
use std::{collections::HashMap, sync::Arc};
use std::{env::args, fmt::Debug as DebugTrait};
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};
use warp::Filter;

use crate::rpc::routes;

mod rpc;
// mod lib;

pub async fn execute_batch<
    Z: ZKVMProver<P>,
    P: ZKVMProof + Serialize + Clone + DebugTrait + TryFrom<NexusProof>,
    E: ZKVMEnv,
>(
    txs: &Vec<TransactionV2>,
    state_machine: &mut StateMachine<E, P>,
    header: &AvailHeader,
    header_store: &mut HeaderStore,
    prover_mode: ProverMode,
) -> Result<(P, NexusHeader), Error>
where
    <P as TryFrom<NexusProof>>::Error: std::fmt::Debug,
{
    let state_update = state_machine
        .execute_batch(&header, header_store, &txs)
        .await?;

    let (proof, result) = {
        #[cfg(any(feature = "sp1"))]
        let NEXUS_RUNTIME_ELF: &[u8] =
            include_bytes!("../../prover/sp1-guest/elf/riscv32im-succinct-zkvm-elf");

        let mut zkvm_prover = Z::new(NEXUS_RUNTIME_ELF.to_vec(), prover_mode);

        let zkvm_txs: Result<Vec<TransactionZKVM>, anyhow::Error> = txs
            .iter()
            .map(|tx| {
                if let TxParamsV2::SubmitProof(submit_proof_tx) = &tx.params {
                    //TODO: Remove transactions that error out from mempool
                    let proof = submit_proof_tx.proof.clone();
                    let receipt: P = P::try_from(proof).unwrap();
                    // let pre_state = match state_update.1.pre_state.get(&submit_proof_tx.app_id.0) {
                    //     Some(i) => i,
                    //     None => {
                    //         return Err(anyhow!(
                    //      "Incorrect StateUpdate computed. Cannot find state for AppAccountId: {:?}",
                    //      submit_proof_tx.app_id
                    //  ))
                    //     }
                    // };

                    zkvm_prover.add_proof_for_recursion(receipt).unwrap();
                }

                Ok(TransactionZKVM {
                    signature: tx.signature.clone(),
                    params: tx.params.clone(),
                })
            })
            .collect();

        let zkvm_txs = zkvm_txs?;

        zkvm_prover.add_input(&zkvm_txs).unwrap();
        zkvm_prover.add_input(&state_update.1).unwrap();
        zkvm_prover.add_input(&header).unwrap();
        zkvm_prover.add_input(&header_store).unwrap();
        let mut proof = zkvm_prover.prove()?;

        let result: NexusHeader = proof.public_inputs()?;
        (proof, result)
    };

    header_store.push_front(&result);

    match state_update.0 {
        Some(i) => {
            state_machine
                .commit_state(&result.state_root, &i.node_batch)
                .await?;
        }
        None => (),
    }

    Ok((proof, result))
}