use alloy_primitives::private::serde::Serialize;
use alloy_primitives::B256;
use helios_consensus_core::consensus_spec::MainnetConsensusSpec;
use helios_consensus_core::types::{LightClientStore, Update};
use nexus_core::types::Proof as NexusProof;
use nexus_core::types::{AppAccountId, NexusRollupPI, H256};
use nexus_core::utils::hasher::{Digest, ShaHasher};
use nexus_core::zkvm::traits::{ZKVMEnv};

#[cfg(any(feature = "native"))]
use nexus_core::zkvm::traits::{ZKVMProof, ZKVMProver};

use nexus_core::zkvm::ProverMode;
use sp1_helios_primitives::types::ProofInputs;
use tree_hash::TreeHash;

pub mod prover;

#[cfg(any(feature = "native", feature = "risc0", feature = "sp1"))]
pub fn create_proof<Z: ZKVMProver<P>, P: ZKVMProof + Serialize + Clone + TryFrom<NexusProof>>(
    elf: Vec<u8>,
    prover_mode: ProverMode,
    prev_proof: Option<P>,
    proof_inputs: ProofInputs,
    prev_pi_option: Option<NexusRollupPI>,
    app_id_option: Option<AppAccountId>,
    guest_image_id: [u32; 8],
    journal_bytes: Option<Vec<u8>>,
    start_nexus_hash: H256,
) -> Result<P, anyhow::Error>
where
    <P as TryFrom<NexusProof>>::Error: std::fmt::Debug,
{
    println!(">>> Prover init >>>");
    let mut prover = Z::new(elf.clone(), prover_mode.clone());

    // If previous proof available, add it in assumption
    if let Some(proof) = prev_proof {
        prover.add_proof_for_recursion(proof)?;
    }

    // Adding inputs
    let inputs_vec: Vec<u8> = serde_cbor::to_vec(&(
        proof_inputs,
        prev_pi_option,
        app_id_option,
        guest_image_id,
        journal_bytes,
        start_nexus_hash
    ))
        .expect("Failed to serialize inputs");
    println!("Serialized inputs: {}", inputs_vec.len());

    prover.add_input(&inputs_vec)?;
    println!(">>> Inputs Added >>>");

    // Run the prover
    prover.prove()
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
        let start_sync_committee_hash = first_update.next_sync_committee.tree_hash_root();
        if <u32 as Into<u64>>::into(prev_pi.height) != prev_head {
            panic!("Height mismatch!");
        }

        println!(
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
        match Z::verify(guest_image_id, &journal_bytes.unwrap()) {
            Ok(()) => {
                println!("Assumption verification successful");
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
