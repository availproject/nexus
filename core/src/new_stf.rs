use crate::types::{
    AccountState, AppAccountId, AvailHeader, HeaderStore, InitAccount, RollupPublicInputs,
    SubmitProof, TransactionV2, TxParamsV2, H256,
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
        new_header: &AvailHeader,
        prev_headers: &mut HeaderStore,
        txs: &Vec<TransactionV2>,
        pre_state: &Vec<(AppAccountId, AccountState)>,
    ) -> Result<(Vec<(AppAccountId, AccountState)>), anyhow::Error> {
        //TODO: Need to remove is empty part. Should not be empty as this opens an attack vector.
        if !prev_headers.is_empty() {
            if new_header.parent_hash.as_fixed_slice()
                != prev_headers.first().hash().as_fixed_slice()
            {
                return Err(anyhow!("Wrong fork, or Avail header skipped."));
            }
        }
        //TODO: Should not clone here, should be cloned outside.
        prev_headers.push_front(new_header);

        let mut post_state = vec![];
        for (index, tx) in txs.iter().enumerate() {
            post_state.push(self.execute_tx(tx, &pre_state[index], prev_headers)?);
        }

        Ok(post_state)
    }

    pub fn execute_tx(
        &self,
        tx: &TransactionV2,
        pre_state: &(AppAccountId, AccountState),
        headers: &HeaderStore,
    ) -> Result<(AppAccountId, AccountState), anyhow::Error> {
        let post_state = match &tx.params {
            TxParamsV2::SubmitProof(params) => self.submit_proof(&params, &pre_state, headers)?,
            TxParamsV2::InitAccount(params) => self.init_account(params, pre_state)?,
        };

        Ok(post_state)
    }

    fn submit_proof(
        &self,
        params: &SubmitProof,
        pre_state: &(AppAccountId, AccountState),
        headers: &HeaderStore,
    ) -> Result<(AppAccountId, AccountState), Error> {
        if pre_state.1 == AccountState::zero() {
            return Err(anyhow!("Invalid transaction, account not initiated."));
        }
        let public_inputs = &params.public_inputs;
        // let journal_vec = match to_vec(&params.public_inputs) {
        //     Ok(i) => i.iter().flat_map(|&x| x.to_le_bytes().to_vec()).collect(),
        //     Err(e) => return Err(anyhow!(e)),
        // };
        //let proof: Receipt = Receipt::new(params.proof.clone(), journal_vec);
        if public_inputs.app_account_id != pre_state.0.clone() {
            return Err(anyhow!("Incorrect app account id"));
        }

        //TODO: Check if you can compare references instead of cloning.
        if public_inputs.pre_state_root.as_fixed_slice().clone() != pre_state.1.state_root {
            return Err(anyhow!("Proof not as per prev state root"));
        }

        if public_inputs.start_avail_hash != H256::from(pre_state.1.last_avail_block_hash) {
            return Err(anyhow!("Missing Avail block"));
        }

        let mut last_header_hash_in_loop: H256 = H256::zero();
        let mut found_match: bool = false;

        for header in headers.inner().iter() {
            if last_header_hash_in_loop == public_inputs.proof_at_avail_hash {
                found_match = true;

                break;
            }

            if last_header_hash_in_loop != H256::zero() {
                if header.hash() != last_header_hash_in_loop {
                    return Err(anyhow!("Incorrect header list given by sequencer."));
                }
            }
            last_header_hash_in_loop = header.parent_hash;

            continue;
        }

        if !found_match {
            return Err(anyhow!("Not right fork, or against last 32 blocks"));
        }

        // match proof.verify(pre_state.1.statement) {
        //     Ok(()) => (),
        //     Err(e) => return Err(anyhow!(e)),
        // };
        let post_state: AccountState = AccountState {
            statement: pre_state.1.statement,
            last_avail_block_hash: public_inputs.proof_at_avail_hash.as_fixed_slice().clone(),
            state_root: public_inputs.next_state_root.as_fixed_slice().clone(),
        };

        Ok((public_inputs.app_account_id.clone(), post_state))
    }

    fn init_account(
        &self,
        params: &InitAccount,
        pre_state: &(AppAccountId, AccountState),
    ) -> Result<(AppAccountId, AccountState), Error> {
        if pre_state.1 != AccountState::zero() {
            return Err(anyhow!("Account already initiated."));
        }

        let mut post_account = AccountState::zero();

        post_account.statement = params.statement;

        Ok((pre_state.0.clone(), post_account))
    }
}
