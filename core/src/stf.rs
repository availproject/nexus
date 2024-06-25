use std::marker::PhantomData;

use crate::{
    types::{
        AccountState, AppAccountId, AvailHeader, HeaderStore, InitAccount, StatementDigest,
        SubmitProof, TransactionZKVM, TxParamsV2, H256,
    },
    zkvm::traits::ZKVMEnv,
};
use anyhow::{anyhow, Error};
use sparse_merkle_tree::traits::Value;

pub struct StateTransitionFunction<Z: ZKVMEnv> {
    z: PhantomData<Z>,
}

//Notes
//How to check if for a list of data hashes, if the right app ID was updated?
//1. AppID -> Hash should be a public input. This can be done in with a vector of app id being exported in the same order of the data root.
//2. All extrinsics have to be processed inside zkvm, and extrinsics root has to be generated.
//3.

impl<Z: ZKVMEnv> StateTransitionFunction<Z> {
    pub fn new() -> Self {
        StateTransitionFunction { z: PhantomData }
    }
    pub fn execute_batch(
        &self,
        new_header: &AvailHeader,
        prev_headers: &mut HeaderStore,
        txs: &Vec<TransactionZKVM>,
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
        prev_headers.push_front(new_header);

        let mut post_state = vec![];
        for (index, tx) in txs.iter().enumerate() {
            post_state.push(self.execute_tx(tx, &pre_state[index], prev_headers)?);
        }

        Ok(post_state)
    }

    pub fn execute_tx(
        &self,
        tx: &TransactionZKVM,
        pre_state: &(AppAccountId, AccountState),
        headers: &HeaderStore,
    ) -> Result<(AppAccountId, AccountState), anyhow::Error> {
        //TODO: Signature verification
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
        if public_inputs.app_id != pre_state.0.clone() {
            return Err(anyhow!("Incorrect app account id"));
        }

        if public_inputs.avail_start_hash != H256::from(pre_state.1.start_avail_hash) {
            return Err(anyhow!("Not a recursive proof from registered start hash."));
        }

        let mut last_header_hash_in_loop: H256 = H256::zero();
        let mut found_match: bool = false;

        for header in headers.inner().iter() {
            if last_header_hash_in_loop == public_inputs.header_hash {
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

        public_inputs.check_consistency(&pre_state.1.statement)?;

        #[cfg(not(feature = "native"))]
        match Z::verify(public_inputs.img_id.0, &public_inputs) {
            Ok(_) => (),
            Err(e) => return Err(anyhow!("Invalid proof")),
        };

        let post_state: AccountState = AccountState {
            statement: pre_state.1.statement.clone(),
            start_avail_hash: pre_state.1.start_avail_hash,
            state_root: public_inputs.state_root.as_fixed_slice().clone(),
        };

        Ok((public_inputs.app_id.clone(), post_state))
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

        post_account.statement = params.statement.clone();
        post_account.start_avail_hash = params.avail_start_hash.as_fixed_slice().clone();

        Ok((pre_state.0.clone(), post_account))
    }
}
