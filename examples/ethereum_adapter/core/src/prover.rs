use crate::types::ProofInputs;
use crate::ProverInputs;
use alloy_primitives::B256;
use helios_consensus_core::consensus_spec::MainnetConsensusSpec;
use helios_consensus_core::types::{LightClientStore, Update};
use helios_consensus_core::{apply_finality_update, apply_update, verify_finality_update, verify_update};
use nexus_core::types::Proof as NexusProof;
use nexus_core::types::{AppAccountId, NexusRollupPI, StatementDigest, H256};
use nexus_core::utils::hasher::{Digest, ShaHasher};
use serde::Serialize;
use std::sync::{Arc, Mutex};
use tracing::{debug, info};
use tree_hash::TreeHash;

use nexus_core::zkvm::traits::ZKVMEnv;
#[cfg(any(feature = "native"))]
use nexus_core::zkvm::traits::{ZKVMProof, ZKVMProver};

pub fn run<Z: ZKVMEnv>() {
    let inputs: Vec<u8> = Z::read_input::<Vec<u8>>().unwrap();

    let prover_inputs = serde_cbor::from_slice::<ProverInputs>(&inputs).unwrap();

    let ProofInputs {
        sync_committee_updates,
        finality_update,
        expected_current_slot,
        mut store,
        genesis_root,
        forks,
        nexus_hash,
    } = prover_inputs.proof_inputs;

    let (app_id, start_sync_committee_hash, _) = check_private_inputs::<Z>(
        &prover_inputs.prev_pi_option,
        &store,
        &nexus_hash,
        &prover_inputs.app_id_option,
        &sync_committee_updates[0],
        prover_inputs.guest_image_id,
        prover_inputs.journal_bytes,
    );

    // 1. Apply sync committee updates, if any
    for (index, update) in sync_committee_updates.iter().enumerate() {
        println!(
            "Processing update {} of {}. Expected current slot: {}",
            index + 1,
            sync_committee_updates.len(),
            expected_current_slot,
        );

        let update_is_valid = verify_update(update, expected_current_slot, &store, genesis_root, &forks).is_ok();
        if !update_is_valid {
            panic!("⚠️Update {} is invalid!", index + 1);
        }

        let checkpoint = apply_update(&mut store, update);
        println!(
            "Update {} is valid. checkpoint : {:?}",
            index + 1,
            checkpoint
        );
    }

    // 2. Apply finality update
    let finality_update_is_valid = verify_finality_update(
        &finality_update,
        expected_current_slot,
        &store,
        genesis_root,
        &forks,
    )
    .is_ok();
    if !finality_update_is_valid {
        panic!("🚨 Finality update is invalid!");
    }

    println!("✅ Finality update is valid.");
    apply_finality_update(&mut store, &finality_update);

    // 3. Commit new state root, header, and sync committee for usage in the on-chain contract
    let header: B256 = store.finalized_header.beacon().tree_hash_root();
    let sync_committee_hash: B256 = store.current_sync_committee.tree_hash_root();
    let next_sync_committee_hash: B256 = match &mut store.next_sync_committee {
        Some(next_sync_committee) => {
            println!("Found next sync committee hash");
            next_sync_committee.tree_hash_root()
        }
        None => {
            println!("No next sync committee hash");
            B256::ZERO
        }
    };
    let head = store.finalized_header.beacon().slot;

    //Commit public inputs for nexus.
    let current_rollup_hash = {
        let mut hasher = ShaHasher::new();
        hasher.0.update(
            store
                .next_sync_committee
                .expect("next sync committee hash is to be known")
                .tree_hash_root(),
        );
        hasher.0.update(store.finalized_header.beacon().tree_hash_root().as_slice());

        hasher.finish()
    };

    let mut state_root_slice = [0u8; 32];

    state_root_slice.copy_from_slice(
        store
            .finalized_header
            .execution()
            .expect("Execution payload doesn't exist.")
            .state_root()
            .as_slice(),
    );
    let public_inputs = NexusRollupPI {
        app_id,
        rollup_hash: Some(current_rollup_hash),
        height: u32::try_from(head).expect("Block number should be less than u32::MAX for nexus"),
        state_root: H256::from(state_root_slice),
        start_nexus_hash: prover_inputs.start_nexus_hash,
        nexus_hash: nexus_hash.clone(),
        img_id: StatementDigest(prover_inputs.guest_image_id),
    };

    println!(
        "Ethereum head: {:?}  \n next_sync_committee: {:?} \n rollup_hash: {:?} \n current sync committee {:?}",
        store.finalized_header.beacon().tree_hash_root(),
        next_sync_committee_hash,
        current_rollup_hash,
        sync_committee_hash,
    );

    // let proof_outputs = ProofOutputs {
    //     execution_state_root: *store
    //         .finalized_header
    //         .execution()
    //         .expect("Execution payload doesn't exist.")
    //         .state_root(),
    //     new_header: header,
    //     next_sync_committee_hash: next_sync_committee_hash,
    //     new_head: head.into(),
    //     prev_header: prev_header,
    //     prev_head: prev_head.into(),
    //     sync_committee_hash: sync_committee_hash,
    // };

    Z::commit(&public_inputs);
}

pub fn check_private_inputs<Z: ZKVMEnv>(
    prev_pi_option: &Option<NexusRollupPI>,
    store: &LightClientStore<MainnetConsensusSpec>,
    nexus_hash: &H256,
    app_id_option: &Option<AppAccountId>,
    first_update: &Update<MainnetConsensusSpec>,
    guest_image_id: [u32; 8],
    journal_bytes: Option<Vec<u8>>,
) -> (AppAccountId, B256, H256) {
    let prev_header: B256 = store.finalized_header.beacon().tree_hash_root();
    let prev_head = store.finalized_header.beacon().slot;

    if let Some(prev_pi) = prev_pi_option {
        let previous_rollup_hash = prev_pi.rollup_hash.expect("Rollup hash to be stored");
        //TODO: Check if this update verification is necessary, as proof already has this next_sync_committee hash, which means this update should have been applied.
        let start_sync_committee_hash = first_update.next_sync_committee().tree_hash_root();
        if <u32 as Into<u64>>::into(prev_pi.height) != prev_head {
            panic!("Height mismatch!");
        }

        info!(
            "previous header {:?}, sync_committee_hash {:?}",
            prev_header, start_sync_committee_hash
        );
        let calculated_rollup_hash = {
            let mut hasher = ShaHasher::new();
            hasher.0.update(start_sync_committee_hash);
            hasher.0.update(prev_header.as_slice());

            hasher.finish()
        };

        if calculated_rollup_hash != previous_rollup_hash {
            panic!("Rollup hash mismatch!")
        }

        // Verifying the assumption added in the host code
        match Z::verify(guest_image_id, prev_pi) {
            Ok(()) => {
                info!("Assumption verification successful");
            }
            Err(e) => {
                panic!("Verification failed: {:?}", e);
            }
        }

        //let calculated
        (
            prev_pi.app_id.clone(),
            start_sync_committee_hash,
            prev_pi.start_nexus_hash,
        )
    } else {
        (
            app_id_option
                .as_ref()
                .expect("Cannot initialize ethereum adapter without an app id")
                .clone(),
            store.current_sync_committee.tree_hash_root(),
            nexus_hash.clone(),
        )
    }
}
