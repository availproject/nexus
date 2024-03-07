use crate::db::NodeDB;
use crate::new_stf::StateTransitionFunction;
use crate::state::VmState;
use crate::types::{
    AccountState, AppAccountId, AppId, AvailHeader, HeaderStore, RollupPublicInputs, StateUpdate,
    SubmitProof, TransactionV2, TxParamsV2, H256,
};
use anyhow::Error;
use avail_subxt::config::Header as HeaderTrait;
use parity_scale_codec::{Decode, Encode};
use sparse_merkle_tree::traits::Value;

pub struct StateMachine {
    stf: StateTransitionFunction,
    state: VmState,
    //db: NodeDB,
}

impl StateMachine {
    pub fn new(root: H256, path: &str) -> Self {
        let state = VmState::new(root, path);
        // let node_db = NodeDB::from_path(String::from(&path));

        StateMachine {
            stf: StateTransitionFunction::new(),
            //      db: node_db,
            state,
        }
    }

    pub fn execute_batch(
        &mut self,
        new_header: &AvailHeader,
        old_headers: &mut HeaderStore,
        txs: &Vec<TransactionV2>,
    ) -> Result<StateUpdate, Error> {
        let pre_state: Vec<(AppAccountId, AccountState)> = txs
            .iter()
            .map(|tx| {
                let app_account_id: AppAccountId = match &tx.params {
                    TxParamsV2::SubmitProof(submit_proof) => {
                        submit_proof.public_inputs.app_account_id.clone()
                    }
                    TxParamsV2::InitAccount(init_account) => {
                        AppAccountId::from(init_account.app_id.clone())
                    }
                };

                let account_state = match self.state.get(&app_account_id.as_h256(), false) {
                    Ok(Some(account)) => account,
                    Err(e) => return Err(anyhow::anyhow!(e)),
                    Ok(None) => AccountState::zero(),
                };

                Ok((app_account_id, account_state))
            })
            .collect::<Result<Vec<_>, _>>()?; //TODO: Need to simplify this part.

        let result = self
            .stf
            .execute_batch(&new_header, old_headers, txs, &pre_state)?;

        if !result.is_empty() {
            Ok(self.state.update_set(
                result
                    .iter()
                    .map(|f| (f.0.as_h256().clone(), f.1.clone()))
                    .collect(),
            )?)
        } else {
            Ok(StateUpdate {
                pre_state_root: self.state.get_root(),
                post_state_root: self.state.get_root(),
                pre_state: vec![],
                post_state: vec![],
                proof: None,
            })
        }
    }
}
