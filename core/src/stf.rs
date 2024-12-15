use crate::traits::NexusTransaction;
use crate::{
    types::{
        AccountState, AppAccountId, AvailHeader, HeaderStore, InitAccount, NexusRollupPI,
        SubmitProof, TransactionZKVM, TxParams, H256,
    },
    zkvm::traits::ZKVMEnv,
};
use anyhow::{anyhow, Error};
use std::collections::HashMap;
use std::marker::PhantomData;
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
    pub fn execute_batch_common<F>(
        &self,
        new_avail_header: &AvailHeader,
        prev_headers: &HeaderStore,
        txs: &Vec<TransactionZKVM>,
        pre_state: &HashMap<[u8; 32], AccountState>,
        mut on_tx_result: F,
    ) -> Result<HashMap<[u8; 32], AccountState>, anyhow::Error>
    where
        F: FnMut(H256, Result<(), anyhow::Error>),
    {
        if let Some(last_header) = prev_headers.first() {
            if new_avail_header.parent_hash != last_header.avail_header_hash {
                return Err(anyhow!("Previous Nexus header not valid as per Avail Header for which block is being built. Last header hash: {:?} Current header hash: {:?}, Current header number {}", last_header.avail_header_hash, new_avail_header.parent_hash, new_avail_header.number));
            }
        }

        let mut post_state: HashMap<[u8; 32], AccountState> = pre_state.clone();

        for tx in txs.iter() {
            let state_key = match &tx.params {
                TxParams::SubmitProof(params) => params.app_id.clone(),
                TxParams::InitAccount(params) => params.app_id.clone(),
            };

            let pre_state = match post_state.get(&state_key.0) {
                None => return Err(anyhow!("Incorrect pre state provided by host.")),
                Some(i) => i,
            };

            let result = self.execute_tx(tx, (&state_key, pre_state), prev_headers);

            let (app_account_id, account_state) = match result {
                Ok((i, j)) => {
                    on_tx_result(tx.hash(), Ok(()));
                    (i, j)
                }
                Err(e) => {
                    on_tx_result(tx.hash(), Err(e));

                    continue;
                }
            };

            post_state.insert(app_account_id.0, account_state);
        }

        Ok(post_state)
    }

    pub fn execute_batch(
        &self,
        new_avail_header: &AvailHeader,
        prev_headers: &HeaderStore,
        txs: &Vec<TransactionZKVM>,
        pre_state: &HashMap<[u8; 32], AccountState>,
    ) -> Result<HashMap<[u8; 32], AccountState>, anyhow::Error> {
        self.execute_batch_common(new_avail_header, prev_headers, txs, pre_state, |_, _| {})
    }

    pub fn execute_batch_with_results(
        &self,
        new_avail_header: &AvailHeader,
        prev_headers: &HeaderStore,
        txs: &Vec<TransactionZKVM>,
        pre_state: &HashMap<[u8; 32], AccountState>,
    ) -> Result<(HashMap<[u8; 32], AccountState>, HashMap<H256, bool>), anyhow::Error> {
        let mut tx_results = HashMap::new();
        let post_state = self.execute_batch_common(
            new_avail_header,
            prev_headers,
            txs,
            pre_state,
            |tx_hash, result| {
                tx_results.insert(tx_hash, result.is_ok());
            },
        )?;
        Ok((post_state, tx_results))
    }

    pub fn execute_tx(
        &self,
        tx: &TransactionZKVM,
        pre_state: (&AppAccountId, &AccountState),
        headers: &HeaderStore,
    ) -> Result<(AppAccountId, AccountState), anyhow::Error> {
        //TODO: Signature verification
        let post_state = match &tx.params {
            TxParams::SubmitProof(params) => self.submit_proof(params, pre_state, headers)?,
            TxParams::InitAccount(params) => self.init_account(params, pre_state)?,
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

        let public_inputs: NexusRollupPI = NexusRollupPI {
            app_id: params.app_id.clone(),
            nexus_hash: params.nexus_hash.clone(),
            height: params.height,
            start_nexus_hash: H256::from(pre_state.1.start_nexus_hash.clone()),
            state_root: params.state_root.clone(),
            img_id: pre_state.1.statement.clone(),
            rollup_hash: params.data,
        };

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
        {
            match Z::verify(public_inputs.img_id.0, &public_inputs) {
                Ok(_) => (),
                Err(e) => return Err(anyhow!("Invalid proof")),
            }
        }

        let post_state: AccountState = AccountState {
            statement: pre_state.1.statement.clone(),
            start_nexus_hash: pre_state.1.start_nexus_hash,
            state_root: params.state_root.as_fixed_slice().clone(),
            height: params.height,
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
