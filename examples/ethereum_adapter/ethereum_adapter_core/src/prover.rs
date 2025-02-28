use crate::check_private_inputs;
use crate::types::ProofInputs;
use alloy_primitives::B256;
use helios_consensus_core::{apply_finality_update, apply_update, verify_finality_update, verify_update};
use nexus_core::types::{AppAccountId, NexusRollupPI, StatementDigest, H256};
use nexus_core::utils::hasher::{Digest, ShaHasher};
use nexus_core::zkvm::traits::ZKVMEnv;
use tree_hash::TreeHash;

pub fn run<Z: ZKVMEnv>() {
    let inputs: Vec<u8> = Z::read_input::<Vec<u8>>().unwrap();

    let (proof_inputs, prev_pi_option, app_id_option, guest_image_id, journal_bytes, start_nexus_hash) = serde_cbor::from_slice::<(
        ProofInputs,
        Option<NexusRollupPI>,
        Option<AppAccountId>,
        [u32; 8],
        Option<Vec<u8>>,
        H256,
    )>(&inputs)
    .unwrap();

    let ProofInputs {
        sync_committee_updates,
        finality_update,
        expected_current_slot,
        mut store,
        genesis_root,
        forks,
        nexus_hash,
    } = proof_inputs;

    let (app_id, start_sync_committee_hash, _) = check_private_inputs::<Z>(
        &prev_pi_option,
        &store,
        &nexus_hash,
        &app_id_option,
        &sync_committee_updates[0],
        guest_image_id,
        journal_bytes,
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
            panic!("Update {} is invalid!", index + 1);
        }
        println!("Update {} is valid.", index + 1);
        apply_update(&mut store, update);
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
        panic!("Finality update is invalid!");
    }
    println!("Finality update is valid.");

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
        start_nexus_hash,
        nexus_hash: nexus_hash.clone(),
        img_id: StatementDigest(guest_image_id),
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
