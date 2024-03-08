use crate::agg_types::{AggregatedTransaction, InitTransaction};
use crate::simple_stf::StateTransitionFunction;
use crate::types::{
    AvailHeader, HeaderStore, NexusHeader, ShaHasher, SimpleNexusHeader, SimpleStateUpdate,
    StateUpdate, TransactionV2, H256,
};
use anyhow::anyhow;
use sparse_merkle_tree::traits::Value;

pub struct ZKVMStateMachine {
    stf: StateTransitionFunction,
}

impl ZKVMStateMachine {
    pub fn new() -> Self {
        Self {
            stf: StateTransitionFunction::new(),
        }
    }

    pub fn execute_batch(
        &self,
        txs: &Vec<InitTransaction>,
        aggregated_tx: AggregatedTransaction,
        state_update: SimpleStateUpdate,
    ) -> Result<SimpleNexusHeader, anyhow::Error> {
        if !txs.is_empty() {
            if let Some(proof) = state_update.proof.clone() {
                match proof.verify::<ShaHasher>(
                    &state_update.pre_state_root,
                    state_update
                        .pre_state
                        .iter()
                        .map(|v| {
                            println!("{:?}, {:?}", v.0, v.1);
                            (v.0.as_h256(), v.1.to_h256())
                        })
                        .collect(),
                ) {
                    Ok(true) => {}
                    //TODO - Change to invalid proof error
                    Ok(false) => {
                        return Err(anyhow!("Invalid merkle proof."));
                    }
                    Err(e) => {
                        return Err(anyhow!("Merkle state verification failed. {:?}", e));
                    }
                };
            } else {
                return Err(anyhow!("Merkle proof for stateupdate not provided."));
            }
        }

        let result = self
            .stf
            .execute_batch(txs, aggregated_tx, &state_update.pre_state)?;

        if !txs.is_empty() {
            if let Some(proof) = state_update.proof {
                match proof.verify::<ShaHasher>(
                    &state_update.post_state_root,
                    result
                        .iter()
                        .map(|v| (v.0.as_h256(), v.1.to_h256()))
                        .collect(),
                ) {
                    Ok(true) => (),
                    //TODO - Change to invalid proof error
                    Ok(false) => {
                        return Err(anyhow!("Invalid merkle proof."));
                    }
                    Err(e) => {
                        println!("{:?}", e);
                        return Err(anyhow!("Merkle state verification failed."));
                    }
                };
            } else {
                return Err(anyhow!("Merkle proof for stateupdate not provided."));
            }
        }
        Ok(SimpleNexusHeader {
            state_root: state_update.post_state_root,
            prev_state_root: state_update.pre_state_root,
            //Tx root over all the transactions sent to nexus.
            tx_root: H256::zero(),
            //Tx root over avail blobs that were used for proof aggregation, this needs to be checked to ensure
            //that Avail blob order has been followed, this will be deprecated and replaced with avail header hash.
            avail_blob_root: H256::zero(),
        })
    }
}
