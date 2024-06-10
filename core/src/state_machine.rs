use std::collections::HashMap;

use crate::db::NodeDB;
use crate::state::VmState;
use crate::stf::StateTransitionFunction;
use crate::types::{
    AccountState, AppAccountId, AppId, AvailHeader, HeaderStore, StateUpdate, SubmitProof,
    TransactionV2, TransactionZKVM, TxParamsV2, H256,
};
use anyhow::Error;
use avail_subxt::config::Header as HeaderTrait;
use parity_scale_codec::{Decode, Encode};
use risc0_zkvm::serde::{to_vec, Serializer};
use risc0_zkvm::{Journal, Receipt};
use serde::Serialize;
use sparse_merkle_tree::traits::Value;

pub struct StateMachine {
    stf: StateTransitionFunction,
    state: VmState,
    db: NodeDB,
}

impl StateMachine {
    pub fn new(root: H256, path: &str) -> Self {
        let chain_state_path = format!("{}/chain_state", path);
        let node_storage_path = format!("{}/node_storage", path);
        let state = VmState::new(root, &chain_state_path);
        let node_db = NodeDB::from_path(&node_storage_path);

        StateMachine {
            stf: StateTransitionFunction::new(),
            db: node_db,
            state,
        }
    }

    pub fn commit_state(&mut self, state_root: &H256) -> Result<(), Error> {
        //TODO: Can remove as fixed slice from below
        if (self.state.get_root().as_fixed_slice() != state_root.as_fixed_slice()) {
            return Err(anyhow::anyhow!("State roots do not match to commit."));
        }
        self.state.commit()?;

        Ok(())
    }

    pub fn execute_batch(
        &mut self,
        avail_header: &AvailHeader,
        old_nexus_headers: &HeaderStore,
        txs: &Vec<TransactionV2>,
    ) -> Result<StateUpdate, Error> {
        let mut pre_state: HashMap<[u8; 32], AccountState> = HashMap::new();

        let result = txs.iter().try_for_each(|tx| {
            let app_account_id: AppAccountId = match &tx.params {
                TxParamsV2::SubmitProof(submit_proof) => submit_proof.app_id.clone(),
                TxParamsV2::InitAccount(init_account) => {
                    AppAccountId::from(init_account.app_id.clone())
                }
            };

            let account_state = match self.state.get(&app_account_id.as_h256(), false) {
                Ok(Some(account)) => account,
                Err(e) => return Err(anyhow::anyhow!(e)), // Exit and return the error
                Ok(None) => AccountState::zero(),
            };

            pre_state.insert(app_account_id.0.clone(), account_state);
            Ok(()) // Continue iterating
        });

        // Check the result and return an error if necessary
        match result {
            Ok(()) => { /* Continue with the rest of your code */ }
            Err(e) => return Err(e),
        }

        //TODO: Need to simplify this part.
        let zkvm_txs: Vec<TransactionZKVM> = txs
            .iter()
            .map(|tx| {
                return TransactionZKVM {
                    params: tx.params.clone(),
                    signature: tx.signature.clone(),
                };
            })
            .collect();
        let result =
            self.stf
                .execute_batch(avail_header, old_nexus_headers, &zkvm_txs, &pre_state)?;

        if !result.is_empty() {
            Ok(self.state.update_set(
                result
                    .iter()
                    .map(|f| (H256::from(f.0.clone()), f.1.clone()))
                    .collect(),
            )?)
        } else {
            Ok(StateUpdate {
                pre_state_root: self.state.get_root(),
                post_state_root: self.state.get_root(),
                pre_state: HashMap::new(),
                post_state: HashMap::new(),
                proof: None,
            })
        }
    }
}
