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
        new_avail_header: &AvailHeader,
        old_headers: &HeaderStore,
        txs: &Vec<TransactionZKVM>,
        state_update: StateUpdate,
    ) -> Result<NexusHeader, anyhow::Error> {
        let number: u32 = if let Some(first_header) = old_headers.first() {
            first_header.number
        } else {
            0
        };

        if !txs.is_empty() {
            if let Some(proof) = state_update.proof.clone() {
                match proof.verify::<ShaHasher>(
                    &state_update.pre_state_root,
                    state_update
                        .pre_state
                        .iter()
                        .map(|v| (H256::from(v.0.clone()), v.1.to_h256()))
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
                .execute_batch(new_avail_header, old_headers, txs, &state_update.pre_state)?;

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
                            (H256::from(v.0.clone()), v.1.to_h256())
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
            parent_hash: match old_headers.first() {
                Some(i) => i.hash(),
                None => H256::zero(),
            },
            number,
            state_root: state_update.post_state_root,
            prev_state_root: state_update.pre_state_root,
            avail_header_hash: H256::from(new_avail_header.hash().as_fixed_slice().clone()),
        })
    }
}
