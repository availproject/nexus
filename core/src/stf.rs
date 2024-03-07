use avail_core::{keccak256, AppExtrinsic};

use crate::types::{
    AccountState, AppAccountId, AppId, Header, MachineCallParams, Transaction, TxParams, H256,
};
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
        new_header: Header,
        data_commitments: Vec<(AppId, H256)>,
        txs: &Vec<Transaction>,
        pre_state: &Vec<(AppAccountId, AccountState)>,
    ) -> Result<Vec<(AppAccountId, AccountState)>, anyhow::Error> {
        let mut post_state: Vec<(AppAccountId, AccountState)>;

        post_state = self.execute_tx(
            &Transaction {
                signature: None,
                params: TxParams::MachineCall(MachineCallParams {
                    header: new_header,
                    data_commitments,
                }),
            },
            pre_state,
        )?;

        for tx in txs {
            post_state = self.execute_tx(tx, &post_state)?;
        }

        Ok(post_state)
    }
    pub fn execute_tx(
        &self,
        tx: &Transaction,
        pre_state: &Vec<(AppAccountId, AccountState)>,
    ) -> Result<Vec<(AppAccountId, AccountState)>, anyhow::Error> {
        let post_state = match &tx.params {
            TxParams::MachineCall(params) => {
                self.machine_call(&params.header, &params.data_commitments, pre_state)?
            }
        };

        Ok(post_state)
    }

    fn machine_call(
        &self,
        new_header: &Header,
        data_commitments: &Vec<(AppId, H256)>,
        pre_state: &Vec<(AppAccountId, AccountState)>,
    ) -> Result<Vec<(AppAccountId, AccountState)>, anyhow::Error> {
        new_header.data_root();

        let modified_state: Vec<(AppAccountId, AccountState)> = vec![];

        for commitment in data_commitments {
            let account = pre_state
                .iter()
                .find(|&account| account.0 == AppAccountId::from(commitment.0));

            match account {
                Some(value) => {
                    let mut account = value.1.clone();

                    account.add_hash(commitment.1.as_fixed_slice())
                }
                None => return Err(anyhow::anyhow!("Account not found..")),
            }
        }

        let leafs: Vec<Vec<u8>> = data_commitments
            .iter()
            .map(|commitment| commitment.1.as_fixed_slice().to_vec())
            .collect();

        let root = calculate_data_root(leafs)?;

        Ok(modified_state)
    }
}
