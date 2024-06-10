use std::collections::HashMap;

use crate::types::{
    AccountState, AppAccountId, AvailHeader, HeaderStore, InitAccount, NexusHeader,
    RollupPublicInputsV2, StatementDigest, SubmitProof, TransactionZKVM, TxParamsV2, H256,
};
use anyhow::{anyhow, Error};
#[cfg(not(feature = "native"))]
use risc0_zkvm::{guest::env::verify, serde::to_vec};
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
        new_avail_header: &AvailHeader,
        prev_headers: &HeaderStore,
        txs: &Vec<TransactionZKVM>,
        pre_state: &HashMap<[u8; 32], AccountState>,
    ) -> Result<HashMap<[u8; 32], AccountState>, anyhow::Error> {
        //Check if this is sequentially the next block.
        if let Some(last_header) = prev_headers.first() {
            if new_avail_header.parent_hash != last_header.avail_header_hash {
                return Err(anyhow!("Previous Nexus header not valid as per Avail Header for which block is being built."));
            }
        }

        let mut post_state: HashMap<[u8; 32], AccountState> = HashMap::new();

        for tx in txs.iter() {
            let state_key = match &tx.params {
                TxParamsV2::SubmitProof(params) => params.app_id.clone(),
                TxParamsV2::InitAccount(params) => params.app_id.clone(),
            };

            let pre_state = match pre_state.get(&state_key.0) {
                None => return Err(anyhow!("Incorrect pre state provided by host.")),
                Some(i) => i,
            };

            let result = self.execute_tx(tx, (&state_key, pre_state), prev_headers)?;
            post_state.insert(result.0 .0, result.1);
        }

        Ok(post_state)
    }

    pub fn execute_tx(
        &self,
        tx: &TransactionZKVM,
        pre_state: (&AppAccountId, &AccountState),
        headers: &HeaderStore,
    ) -> Result<(AppAccountId, AccountState), anyhow::Error> {
        //TODO: Signature verification
        let post_state = match &tx.params {
            TxParamsV2::SubmitProof(params) => self.submit_proof(params, pre_state, headers)?,
            TxParamsV2::InitAccount(params) => self.init_account(params, pre_state)?,
        };

        Ok(post_state)
    }

    fn submit_proof(
        &self,
        params: &SubmitProof,
        pre_state: (&AppAccountId, &AccountState),
        headers: &HeaderStore,
    ) -> Result<(AppAccountId, AccountState), Error> {
        if pre_state.1.clone() == AccountState::zero() {
            return Err(anyhow!("Invalid transaction, account not initiated."));
        }

        let public_inputs: RollupPublicInputsV2 = RollupPublicInputsV2 {
            app_id: params.app_id.clone(),
            nexus_hash: params.nexus_hash.clone(),
            start_nexus_hash: H256::from(pre_state.1.start_nexus_hash.clone()),
            state_root: params.state_root.clone(),
            img_id: pre_state.1.statement.clone(),
        };

        // let journal_vec = match to_vec(&params.public_inputs) {
        //     Ok(i) => i.iter().flat_map(|&x| x.to_le_bytes().to_vec()).collect(),
        //     Err(e) => return Err(anyhow!(e)),
        // };
        //let proof: Receipt = Receipt::new(params.proof.clone(), journal_vec);
        if public_inputs.app_id != pre_state.0.clone() {
            return Err(anyhow!("Incorrect app account id"));
        }

        if public_inputs.start_nexus_hash != H256::from(pre_state.1.start_nexus_hash) {
            return Err(anyhow!("Not a recursive proof from registered start hash."));
        }

        let mut header_hash: H256 = match headers.first() {
            Some(i) => i.hash(),
            None => unreachable!("Node is not expected to accept submit proof txs if first block."),
        };
        let mut found_header_height: Option<u32> = None;

        for header in headers.inner().iter() {
            if header.hash() != header_hash {
                return Err(anyhow!("Incorrect header list given by sequencer."));
            }

            if header_hash == public_inputs.nexus_hash {
                found_header_height = Some(header.number);

                break;
            }

            header_hash = header.parent_hash;

            continue;
        }

        if found_header_height.is_none() {
            return Err(anyhow!("Not right fork, or against last 32 blocks"));
        }

        public_inputs.check_consistency(&pre_state.1.statement)?;

        #[cfg(not(feature = "native"))]
        let public_input_vec = match to_vec(&public_inputs) {
            Ok(i) => i,
            Err(e) => return Err(anyhow!("Could not encode public inputs of rollup.")),
        };

        #[cfg(not(feature = "native"))]
        match verify(public_inputs.img_id.0, &public_input_vec) {
            Ok(_) => (),
            Err(e) => return Err(anyhow!("Invalid proof")),
        };

        let post_state: AccountState = AccountState {
            statement: pre_state.1.statement.clone(),
            start_nexus_hash: pre_state.1.start_nexus_hash,
            state_root: params.state_root.as_fixed_slice().clone(),
            //Okay to do unwrap as we check above if it is None.
            last_proof_height: found_header_height.unwrap(),
        };

        Ok((public_inputs.app_id.clone(), post_state))
    }

    fn init_account(
        &self,
        params: &InitAccount,
        pre_state: (&AppAccountId, &AccountState),
    ) -> Result<(AppAccountId, AccountState), Error> {
        if pre_state.1.clone() != AccountState::zero() {
            return Err(anyhow!("Account already initiated."));
        }

        let mut post_account = AccountState::zero();

        post_account.statement = params.statement.clone();
        post_account.start_nexus_hash = params.start_nexus_hash.as_fixed_slice().clone();

        Ok((pre_state.0.clone(), post_account))
    }
}
