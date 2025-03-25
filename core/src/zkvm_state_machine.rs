use std::collections::HashMap;

use crate::state::types::AccountState;
use crate::stf::StateTransitionFunction;
use crate::types::{
    AvailHeader, Blob, BlobProof, CompactDataLookup, HeaderExtension, HeaderStore, NexusHeader, Sha256, StateUpdate, Transaction, TransactionZKVM,
    H256,
};
use crate::utils::hasher::{Digest, ShaHasher};
use crate::zkvm::traits::ZKVMEnv;
use anyhow::anyhow;
use jmt::{KeyHash, RootHash};
use kzg::verify_row_kzg;

pub struct ZKVMStateMachine<Z: ZKVMEnv> {
    stf: StateTransitionFunction<Z>,
}

impl<Z: ZKVMEnv> ZKVMStateMachine<Z> {
    pub fn new() -> Self {
        Self { stf: StateTransitionFunction::new() }
    }

    pub fn execute_batch(
        &self,
        new_avail_header: &AvailHeader,
        old_headers: &HeaderStore,
        blobs: &Vec<Blob>,
        blob_proofs: &Vec<BlobProof>,
        state_update: StateUpdate,
        app_id: u32,
    ) -> Result<NexusHeader, anyhow::Error> {
        let number: u32 = if let Some(first_header) = old_headers.first() {
            first_header.number + 1
        } else {
            0
        };

        let mut txs: Vec<Transaction> = Vec::new();

        for blob in blobs {
            let blob_txs: Vec<Transaction> = bincode::deserialize(&blob.get_data()).map_err(|e| anyhow!("blob deserialization error: {:?}", e))?;
            txs.extend(blob_txs);
        }

        let commitments: Vec<[u8; 48]> = {
            let (app_lookup, commitments): (CompactDataLookup, Vec<[u8; 48]>) = match &new_avail_header.extension {
                HeaderExtension::V3(extension) => {
                    let commitment_chunks: Vec<[u8; 48]> = extension
                        .commitment
                        .commitment
                        .chunks_exact(48)
                        .map(|chunk| {
                            let mut arr = [0u8; 48];
                            arr.copy_from_slice(chunk);
                            arr
                        })
                        .collect();
                    (extension.app_lookup.clone(), commitment_chunks)
                }
                _ => return Err(anyhow!("Header extension not supported")),
            };

            let mut filtered_commitments: Vec<[u8; 48]> = Vec::new();

            for (idx, current) in app_lookup.index.iter().enumerate() {
                if current.app_id.0 == app_id {
                    let start = current.start as usize;
                    let end = if idx + 1 < app_lookup.index.len() {
                        app_lookup.index[idx + 1].start as usize
                    } else {
                        commitments.len()
                    };

                    filtered_commitments.extend_from_slice(&commitments[start..end]);
                }
            }

            filtered_commitments
        };

        for (i, blob) in blobs.iter().enumerate() {
            let verification_result = verify_row_kzg(&blob.0, &commitments[i], &blob_proofs[i].0)?;
            if !verification_result {
                return Err(anyhow!("KZG verification failed for blob {}", i));
            }
        }

        let mut pre_state: HashMap<[u8; 32], AccountState> = HashMap::new();
        if !txs.is_empty() {
            //TODO: Implement multiproof to avoid verifying each leaf.
            state_update
                .pre_state
                .iter()
                .enumerate()
                .try_for_each::<_, Result<(), anyhow::Error>>(|(index, (key, (account_state, proof)))| {
                    let value = match account_state {
                        Some(i) => Some(i.encode()),
                        None => None,
                    };

                    pre_state.insert(
                        key.clone(),
                        account_state.clone().unwrap_or_else(AccountState::zero),
                    );

                    proof.verify(
                        RootHash(state_update.pre_state_root.as_fixed_slice().clone()),
                        KeyHash(key.clone()),
                        value,
                    )?;

                    Ok(())
                })?
        }

        let result = self.stf.execute_batch(new_avail_header, old_headers, &txs, &pre_state)?;

        //TODO verify post state root.

        let txs_encoded: Vec<u8> = parity_scale_codec::Encode::encode(&txs);

        let mut hasher = ShaHasher::new();
        hasher.0.update(&txs_encoded);
        let tx_root = hasher.finish();

        Ok(NexusHeader {
            parent_hash: match old_headers.first() {
                Some(i) => i.hash(),
                None => H256::zero(),
            },
            number,
            // tx_root,
            state_root: state_update.post_state_root,
            prev_state_root: state_update.pre_state_root,
            avail_header_hash: H256::from(new_avail_header.hash().as_fixed_slice().clone()),
        })
    }
}
