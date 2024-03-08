use crate::{
    agg_types::{AggregatedTransaction, InitTransaction},
    types::{
        AppAccountId, AvailHeader, HeaderStore, InitAccount, RollupPublicInputs,
        SimpleAccountState, SubmitProof, TransactionV2, TxParamsV2, H256,
    },
};
use anyhow::{anyhow, Error};
use sparse_merkle_tree::traits::Value;
pub struct StateTransitionFunction;

//Notes
//How to check if for a list of data hashes, if the right app ID was updated?
//1. AppID -> Hash should be a public input. This can be done in with a vector of app id being exported in the same order of the data root.
//2. All extrinsics have to be processed inside zkvm, and extrinsics root has to be generated.
//3.

impl StateTransitionFunction {
    pub fn new() -> Self {
        StateTransitionFunction {}
    }
    pub fn execute_batch(
        &self,
        txs: &Vec<InitTransaction>,
        aggregated_tx: AggregatedTransaction,
        pre_state: &Vec<(AppAccountId, SimpleAccountState)>,
    ) -> Result<Vec<(AppAccountId, SimpleAccountState)>, anyhow::Error> {
        //TODO: Need to remove is empty part. Should not be empty as this opens an attack vector.
        let mut post_state = vec![];
        for (index, tx) in txs.iter().enumerate() {
            post_state.push(self.init_account(&tx.params, &pre_state[index])?);
        }

        for (index, tx) in aggregated_tx.submit_proof_txs.iter().enumerate() {
            post_state.push(self.submit_proof(&tx.params, &pre_state[index])?)
        }

        Ok(post_state)
    }

    fn submit_proof(
        &self,
        params: &SubmitProof,
        pre_state: &(AppAccountId, SimpleAccountState),
    ) -> Result<(AppAccountId, SimpleAccountState), Error> {
        if pre_state.1 == SimpleAccountState::zero() {
            return Err(anyhow!("Invalid transaction, account not initiated."));
        }
        let public_inputs = &params.public_inputs;
        // let journal_vec = match to_vec(&params.public_inputs) {
        //     Ok(i) => i.iter().flat_map(|&x| x.to_le_bytes().to_vec()).collect(),
        //     Err(e) => return Err(anyhow!(e)),
        // };
        //let proof: Receipt = Receipt::new(params.proof.clone(), journal_vec);
        if params.app_account_id != pre_state.0.clone() {
            return Err(anyhow!("Incorrect app account id"));
        }

        //TODO: Check if you can compare references instead of cloning.
        if public_inputs.pre_state_root.as_fixed_slice().clone() != pre_state.1.state_root {
            return Err(anyhow!("Proof not as per prev state root"));
        }

        // match proof.verify(pre_state.1.statement) {
        //     Ok(()) => (),
        //     Err(e) => return Err(anyhow!(e)),
        // };
        let post_state: SimpleAccountState = SimpleAccountState {
            statement: pre_state.1.statement,
            state_root: public_inputs.next_state_root.as_fixed_slice().clone(),
        };

        Ok((params.app_account_id.clone(), post_state))
    }

    fn init_account(
        &self,
        params: &InitAccount,
        pre_state: &(AppAccountId, SimpleAccountState),
    ) -> Result<(AppAccountId, SimpleAccountState), Error> {
        if pre_state.1 != SimpleAccountState::zero() {
            return Err(anyhow!("Account already initiated."));
        }

        let mut post_account = SimpleAccountState::zero();

        post_account.statement = params.statement;

        Ok((pre_state.0.clone(), post_account))
    }
}
