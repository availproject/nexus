use std::collections::HashMap;

use crate::state::VmState;
use crate::stf::StateTransitionFunction;
use crate::types::{
    AccountState, AppAccountId, AvailHeader, HeaderStore, StateUpdate, Transaction,
    TransactionZKVM, TxParams, H256,
};
use crate::zkvm::traits::{ZKVMEnv, ZKVMProof};
use anyhow::{anyhow, Error};
use jmt::storage::{NodeBatch, TreeUpdateBatch};
use jmt::Version;
use serde::Serialize;
use std::fmt::Debug as DebugTrait;
use std::marker::PhantomData;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, error, info, instrument, warn};

pub struct StateMachine<Z: ZKVMEnv, P: ZKVMProof + DebugTrait + Clone> {
    stf: StateTransitionFunction<Z>,
    state: Arc<Mutex<VmState>>,
    p: PhantomData<P>, //db: NodeDB,
}

impl<Z: ZKVMEnv, P: ZKVMProof + Serialize + DebugTrait + Clone> StateMachine<Z, P> {
    #[instrument(level = "debug", skip(state))]
    pub fn new(state: Arc<Mutex<VmState>>) -> Self {
        debug!("Creating new StateMachine");
        StateMachine {
            stf: StateTransitionFunction::new(),
            //      db: node_db,
            p: PhantomData,
            state,
        }
    }

    #[instrument(level = "debug", skip(self, state_root))]
    pub async fn commit_state(
        &mut self,
        state_root: &H256,
        node_batch: &NodeBatch,
        batch_number: u32,
    ) -> Result<(), Error> {
        debug!(
            "Committing state in state machine for batch {}",
            batch_number
        );
        let mut state_lock = self.state.lock().await;
        state_lock.commit(node_batch)?;

        let version = match state_lock.get_version(true)? {
            Some(i) => i,
            None => 0,
        };
        let root = state_lock.get_root(version)?;

        //TODO: Can remove as fixed slice from below
        if (root.as_fixed_slice() != state_root.as_fixed_slice()) {
            return Err(anyhow::anyhow!("State roots do not match to commit."));
        }

        info!(
            "State commitment done for batch {}. State root: {:?}",
            batch_number, state_root
        );
        Ok(())
    }

    #[instrument(level = "debug", skip(self, avail_header, old_nexus_headers, txs), fields(num_txs = txs.len(), num_headers = old_nexus_headers.inner().len()))]
    pub async fn execute_batch(
        &mut self,
        avail_header: &AvailHeader,
        old_nexus_headers: &HeaderStore,
        txs: &Vec<Transaction>,
    ) -> Result<(Option<TreeUpdateBatch>, StateUpdate, HashMap<H256, bool>), Error> {
        debug!("Executing batch in state machine");
        //TODO: Increment version for each update.
        let mut pre_state: HashMap<[u8; 32], AccountState> = HashMap::new();

        let result: Result<u64, anyhow::Error> = {
            let state_lock = self.state.lock().await;
            let prev_version = match state_lock.get_version(true)? {
                Some(i) => i,
                None => 0,
            };

            txs.iter().try_for_each(|tx| {
                let app_account_id: AppAccountId = match &tx.params {
                    TxParams::SubmitProof(submit_proof) => submit_proof.app_id.clone(),
                    TxParams::InitAccount(init_account) => {
                        AppAccountId::from(init_account.app_id.clone())
                    }
                };

                let account_state = match state_lock.get(&app_account_id.as_h256(), prev_version) {
                    Ok(Some(account)) => account,
                    Err(e) => return Err(anyhow!("{:?}", e)), // Exit and return the error
                    Ok(None) => AccountState::zero(),
                };

                pre_state.insert(app_account_id.0.clone(), account_state);
                Ok(()) // Continue iterating
            })?;

            Ok(prev_version)
        };

        // Check the result and return an error if necessary
        let prev_version = match result {
            Ok(i) => i,
            Err(e) => return Err(e),
        };

        let version = prev_version + 1;
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
        let (stf_state_result, tx_result) = self.stf.execute_batch_with_results(
            avail_header,
            old_nexus_headers,
            &zkvm_txs,
            &pre_state,
        )?;
        let mut state_lock = self.state.lock().await;

        if !stf_state_result.is_empty() {
            let num_state_changes = stf_state_result.len();

            let result = state_lock.update_set(
                stf_state_result
                    .into_iter()
                    .map(|(key, account_state)| {
                        if account_state == AccountState::zero() {
                            (H256::from(key), None)
                        } else {
                            (H256::from(key), Some(account_state))
                        }
                    })
                    .collect(),
                version,
            )?;

            state_lock.update_version(version)?;

            info!(
                "Pre execution of batch {} successful. State changes count: {}",
                avail_header.number, num_state_changes
            );
            Ok((Some(result.0), result.1, tx_result))
        } else {
            let root = state_lock.get_root(version - 1)?;

            info!(
                "Pre execution of batch {} successful. State changes count: {}",
                avail_header.number,
                stf_state_result.len()
            );
            Ok((
                None,
                StateUpdate {
                    pre_state_root: root,
                    post_state_root: root,
                    pre_state: HashMap::new(),
                },
                tx_result,
            ))
        }
    }
}
