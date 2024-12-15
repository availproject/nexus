use std::collections::HashMap;

use crate::state::types::AccountState;
use crate::stf::StateTransitionFunction;
use crate::types::{
    AvailHeader, HeaderStore, NexusHeader, Sha256, StateUpdate, TransactionZKVM, H256,
};
use crate::utils::hasher::{Digest, ShaHasher};
use crate::zkvm::traits::ZKVMEnv;
use jmt::{KeyHash, RootHash};
use serde::Serialize;

pub struct ZKVMStateMachine<Z: ZKVMEnv> {
    stf: StateTransitionFunction<Z>,
}

impl<Z: ZKVMEnv> ZKVMStateMachine<Z> {
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
            first_header.number + 1
        } else {
            0
        };

        let mut pre_state: HashMap<[u8; 32], AccountState> = HashMap::new();
        if !txs.is_empty() {
            //TODO: Implement multiproof to avoid verifying each leaf.
            state_update
                .pre_state
                .iter()
                .enumerate()
                .try_for_each::<_, Result<(), anyhow::Error>>(
                    |(index, (key, (account_state, proof)))| {
                        let value = match account_state {
                            Some(i) => Some(i.encode()),
                            None => None,
                        };

                        pre_state.insert(
                            key.clone(),
                            account_state.clone().unwrap_or_else(AccountState::zero),
                        );

                        proof.verify(
                            RootHash(state_update.pre_state_root.as_fixed_slice().clone()),
                            KeyHash(key.clone()),
                            value,
                        )?;

                        Ok(())
                    },
                )?
        }

        let result = self
            .stf
            .execute_batch(new_avail_header, old_headers, txs, &pre_state)?;

        //TODO verify post state root.

        let txs_encoded: Vec<u8> = parity_scale_codec::Encode::encode(&txs);

        let mut hasher = ShaHasher::new();
        hasher.0.update(&txs_encoded);
        let tx_root = hasher.finish();

        Ok(NexusHeader {
            parent_hash: match old_headers.first() {
                Some(i) => i.hash(),
                None => H256::zero(),
            },
            number,
            tx_root,
            state_root: state_update.post_state_root,
            prev_state_root: state_update.pre_state_root,
            avail_header_hash: H256::from(new_avail_header.hash().as_fixed_slice().clone()),
        })
    }
}
