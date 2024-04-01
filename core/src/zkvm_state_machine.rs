use crate::stf::StateTransitionFunction;
use crate::types::{
    AvailHeader, HeaderStore, NexusHeader, ShaHasher, StateUpdate, TransactionZKVM, H256,
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
        new_header: &AvailHeader,
        old_headers: &mut HeaderStore,
        txs: &Vec<TransactionZKVM>,
        state_update: StateUpdate,
    ) -> Result<NexusHeader, anyhow::Error> {
        if !txs.is_empty() {
            if let Some(proof) = state_update.proof.clone() {
                match proof.verify::<ShaHasher>(
                    &state_update.pre_state_root,
                    state_update
                        .pre_state
                        .iter()
                        .map(|v| (v.0.as_h256(), v.1.to_h256()))
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

        let result =
            self.stf
                .execute_batch(new_header, old_headers, txs, &state_update.pre_state)?;

        if !txs.is_empty() {
            if let Some(proof) = state_update.proof {
                match proof.verify::<ShaHasher>(
                    &state_update.post_state_root,
                    result
                        .iter()
                        .map(|v| {
                            println!(
                                "Modified account after batch : {:?} -- AccountID: {:?}",
                                v.1, v.0
                            );
                            (v.0.as_h256(), v.1.to_h256())
                        })
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
        Ok(NexusHeader {
            state_root: state_update.post_state_root,
            prev_state_root: state_update.pre_state_root,
            avail_header_hash: H256::from(new_header.hash().as_fixed_slice().clone()),
        })
    }
}
