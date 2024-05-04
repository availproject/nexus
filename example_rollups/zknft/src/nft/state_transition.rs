use crate::{
    nft::types::{
        Burn, Future, FutureReceiptData, Mint, Nft, NftTransaction, NftTransactionMessage,
        Transfer, TransferReceiptData, Trigger,
    },
    traits::StateTransition,
    types::{Address, AggregatedBatch, ShaHasher, TransactionReceipt},
};
use anyhow::{anyhow, Error};
use sparse_merkle_tree::traits::Value;
use sparse_merkle_tree::H256;

pub struct NftStateTransition;

impl NftStateTransition {
    pub fn new() -> Self {
        NftStateTransition {}
    }

    fn transfer(&self, params: Transfer, pre_state: Nft) -> Result<Vec<Nft>, Error> {
        if pre_state == Nft::zero() {
            return Err(anyhow!("NFT not minted."));
        }

        if pre_state.owner != params.from {
            return Err(anyhow!("Not owner"));
        }

        let updated_nonce = pre_state.nonce + 1;

        match params.future_commitment {
            None => Ok(vec![Nft {
                id: params.id.clone(),
                owner: params.to.clone(),
                nonce: updated_nonce,
                future: None,
                metadata: pre_state.metadata,
            }]),
            Some(i) => Ok(vec![Nft {
                id: params.id.clone(),
                owner: pre_state.owner,
                future: Some(Future {
                    to: params.to.clone(),
                    commitment: i,
                }),
                metadata: pre_state.metadata,
                nonce: updated_nonce,
            }]),
        }
    }

    fn mint(&self, params: Mint, pre_state: Nft) -> Result<Vec<Nft>, Error> {
        if pre_state != Nft::zero() {
            return Err(anyhow!("Already minted, {:?}", pre_state));
        }

        match params.future_commitment {
            None => Ok(vec![Nft {
                id: params.id.clone(),
                owner: params.to.clone(),
                nonce: 1,
                future: None,
                metadata: params.metadata,
            }]),
            Some(i) => Ok(vec![Nft {
                id: params.id.clone(),
                owner: Address::zero(),
                nonce: 1,
                future: Some(Future {
                    to: params.to.clone(),
                    commitment: i,
                }),
                metadata: params.metadata,
            }]),
        }
    }

    fn burn(&self, params: Burn, pre_state: Nft) -> Result<Vec<Nft>, Error> {
        if pre_state == Nft::zero() {
            return Err(anyhow!("Nft does not exist"));
        }

        if pre_state.owner != params.from {
            return Err(anyhow!("Not owner"));
        }

        let updated_nonce = pre_state.nonce + 1;

        match params.future_commitment {
            None => Ok(vec![Nft {
                id: params.id.clone(),
                owner: Address::zero(),
                nonce: updated_nonce,
                future: None,
                metadata: pre_state.metadata,
            }]),
            Some(i) => Ok(vec![Nft {
                id: params.id.clone(),
                owner: pre_state.owner,
                future: Some(Future {
                    to: Address::zero(),
                    commitment: i,
                }),
                metadata: pre_state.metadata,
                nonce: updated_nonce,
            }]),
        }
    }

    fn trigger(
        &self,
        params: Trigger,
        pre_state: Nft,
        aggregated_proof: AggregatedBatch,
    ) -> Result<Vec<Nft>, Error> {
        if pre_state == Nft::zero() {
            return Err(anyhow!("Nft does not exist."));
        }

        let future = match pre_state.future {
            None => return Err(anyhow!("No future registered.")),
            Some(i) => i,
        };

        match params.merkle_proof.verify::<ShaHasher>(
            &aggregated_proof.receipts_root,
            //Checks both inclusion or non inclusion.
            vec![(future.commitment, params.receipt.to_h256())],
        ) {
            Ok(true) => (),
            Ok(false) => return Err(anyhow!("Invalid merkle proof.")),
            Err(_e) => return Err(anyhow!("Error while verifying merkle")),
        }

        let updated_nonce = pre_state.nonce + 1;

        //Check if the given proof was non inclusion.
        if params.receipt.to_h256() == H256::zero() {
            //Revert transaction if the receipt is not included.
            Ok(vec![Nft {
                id: params.id.clone(),
                owner: pre_state.owner.clone(),
                future: None,
                metadata: pre_state.metadata,
                nonce: updated_nonce,
            }])
        } else {
            Ok(vec![Nft {
                id: params.id.clone(),
                owner: future.to.clone(),
                future: None,
                nonce: updated_nonce,
                metadata: pre_state.metadata,
            }])
        }
    }
}

impl StateTransition<Nft, NftTransaction> for NftStateTransition {
    fn execute_tx(
        &self,
        pre_state: Vec<Nft>,
        params: NftTransaction,
        aggregated_proof: AggregatedBatch,
    ) -> Result<Vec<Nft>, Error> {
        let message: NftTransactionMessage = NftTransactionMessage::try_from(params.clone())?;

        match {
            match message.clone() {
                NftTransactionMessage::Transfer(i) => {
                    i.from.verify_msg(&params.signature, &params.message)
                }
                NftTransactionMessage::Mint(i) => {
                    i.from.verify_msg(&params.signature, &params.message)
                }
                NftTransactionMessage::Burn(i) => {
                    i.from.verify_msg(&params.signature, &params.message)
                }
                NftTransactionMessage::Trigger(i) => {
                    i.from.verify_msg(&params.signature, &params.message)
                }
            }
        } {
            true => (),
            false => return Err(anyhow!("Signature verification failed.")),
        };

        match message {
            NftTransactionMessage::Transfer(i) => self.transfer(i, pre_state[0].clone()),
            NftTransactionMessage::Mint(i) => self.mint(i, pre_state[0].clone()),
            NftTransactionMessage::Burn(i) => self.burn(i, pre_state[0].clone()),
            NftTransactionMessage::Trigger(i) => {
                self.trigger(i, pre_state[0].clone(), aggregated_proof)
            }
        }
    }
}
