use crate::traits::RollupProof;
use crate::types::{AdapterPrivateInputs, AdapterPublicInputs, RollupProofWithPublicInputs};
use anyhow::{anyhow, Error};
use nexus_core::traits::Hasher;
use nexus_core::types::{
    AppAccountId, AvailHeader, Extension, ShaHasher, StatementDigest, V3Extension, H256,
};
#[cfg(feature = "zkvm-risc0")]
use risc0_zkvm::{
    guest::env::{self, verify},
    serde::to_vec,
    sha::rust_crypto::Digest,
};

// #[cfg(feature = "zkvm-sp1")]
// use digest::Update

use digest::Update;

use serde::Serialize;

/// Verifies a proof against a specified set of public inputs.
///
/// # Arguments
///
/// * `proof`: The proof to be verified, implementing the `Proof` trait by rollup adapter.
/// * `rollup_public_inputs`: The public inputs to the rollup proof, defined by the rollup adapter.
/// * `prev_adapter_public_inputs`: Optional public inputs from the previous proof of adapter. Can be `None` if not applicable.
/// * `private_inputs`: The private inputs to the adapter, implementing the `AdapterPrivateInputs` trait.
/// * `img_id`: The image ID, convertible into a `Digest`.
/// * `vk`: The verification key as a 32-byte array.
///
/// # Returns
///
/// Returns the public inputs to the adapter upon successful verification of the proof.
///
/// # Errors
///
/// Returns an error if the proof verification fails for any reason.
///
/// # Example
///
/// ```rust
/// use my_crate::{Proof, RollupPublicInputs, AdapterPublicInputs, AdapterPrivateInputs, Error};
/// use my_crate::digest::Digest;
///
/// // Define your types for Proof, RollupPublicInputs, AdapterPublicInputs, and AdapterPrivateInputs
///
/// # struct MyProof;
/// # struct MyRollupPublicInputs;
/// # struct MyAdapterPublicInputs;
/// # struct MyAdapterPrivateInputs;
/// # struct MyError;
/// # impl Proof for MyProof {}
/// # impl RollupPublicInputs for MyRollupPublicInputs {}
/// # impl AdapterPublicInputs for MyAdapterPublicInputs {}
/// # impl AdapterPrivateInputs for MyAdapterPrivateInputs {}
/// # type MyDigest = [u8; 32];
///
/// fn verify_proof_wrapper() -> Result<MyAdapterPublicInputs, MyError> {
///     let is_recursive = true;
///     let proof = MyProof;
///     let rollup_public_inputs = MyRollupPublicInputs;
///     let prev_adapter_public_inputs = None;
///     let private_inputs = MyAdapterPrivateInputs;
///     let img_id: MyDigest = [0; 32];
///     let vk: [u8; 32] = [0; 32];
///     
///     verify_proof(is_recursive, proof, rollup_public_inputs, prev_adapter_public_inputs, private_inputs, img_id, vk)
/// }
/// ```
///
/// In this example, `verify_proof_wrapper` attempts to verify a proof given the specified inputs.
///
/// # Note
///
/// Ensure that the types `Proof`, `RollupPublicInputs`, `AdapterPublicInputs`, `AdapterPrivateInputs`, `Error`, and `Digest` are properly defined and implemented.

pub fn verify_proof<P: RollupProof>(
    rollup_proof: Option<RollupProofWithPublicInputs<P>>,
    prev_adapter_public_inputs: Option<AdapterPublicInputs>,
    private_inputs: AdapterPrivateInputs,
    img_id: StatementDigest,
    vk: [u8; 32],
) -> Result<AdapterPublicInputs, Error> {
    /*  Things adapter must check,
    1. Check if first proof or not, for first proof, proof should be at start height - ✅
    2. Check if proof height is sequential as per previous proof provided - ✅
    3. Verify if previous proof is valid - ✅
    4. Check for current height if input is valid, this is checked against header - 😢
    5. Check if current proof is sequential as per last proof - ✅
    6. Verify current proof -  ✅
    7. Hash the header provided - ✅
    8. Allow verification of empty proof - ✅
    */

    let current_avail_hash: H256 = private_inputs.avail_header.hash();
    let nexus_hash: H256 = private_inputs.nexus_header.hash();

    if current_avail_hash != private_inputs.nexus_header.avail_header_hash {
        return Err(anyhow!("Incorrect Nexus Header provided."));
    }
    //TODO: Check inclusion proof for data blob, app index check, and empty block check.
    let mut hasher = ShaHasher::new();

    //hasher.0.update(&private_inputs.app_id.0.to_be_bytes());
    digest::Digest::update(&mut hasher.0, &private_inputs.app_id.0.to_be_bytes());

    let hash: H256 = hasher.finish();
    let app_account_id: AppAccountId = AppAccountId::from(hash);

    let (proof, rollup_public_inputs) = match rollup_proof {
        Some(i) => (i.proof, i.public_inputs),
        None => {
            let app_lookup = match private_inputs.avail_header.extension {
                Extension::V3(extension) => extension.app_lookup,
                _ => unreachable!("Other headers not expected"),
            };

            let mut empty_block: bool = true;

            for appindex in app_lookup.index {
                if appindex.app_id == private_inputs.app_id {
                    empty_block = false;
                }
            }

            if !empty_block {
                return Err(anyhow!("Header not empty, but no proof"));
            }

            return Ok(match prev_adapter_public_inputs {
                Some(i) => AdapterPublicInputs {
                    nexus_hash,
                    height: i.height,
                    state_root: i.state_root,
                    start_nexus_hash: i.start_nexus_hash,
                    app_id: app_account_id,
                    img_id: i.img_id,
                    rollup_hash: i.rollup_hash
                },
                None => AdapterPublicInputs {
                    nexus_hash: nexus_hash.clone(),
                    height: 0,
                    state_root: H256::zero(),
                    start_nexus_hash: nexus_hash,
                    app_id: app_account_id,
                    img_id: img_id.clone(),
                    rollup_hash: Some(H256::zero())
                },
            });
        }
    };

    let prev_state_root: H256 = rollup_public_inputs.prev_state_root;
    let post_state_root: H256 = rollup_public_inputs.post_state_root;

    //TODO: Remove unwrap below.
    //TODO: Allow custom encoding here.
    proof.verify(&vk, &rollup_public_inputs)?;

    let prev_public_input: AdapterPublicInputs = match prev_adapter_public_inputs {
        Some(i) => i,
        None => {
            return {
                if prev_state_root != H256::zero() {
                    Err(anyhow::anyhow!("Previous proof not submitted"))
                } else {
                    Ok(AdapterPublicInputs {
                        height: 0,
                        nexus_hash: nexus_hash.clone(),
                        state_root: post_state_root,
                        start_nexus_hash: nexus_hash,
                        app_id: app_account_id,
                        img_id: img_id.clone(),
                        rollup_hash: Some(H256::zero())
                    })
                }
            }
        }
    };

    if prev_state_root != prev_public_input.state_root {
        return Err(anyhow::anyhow!("Not sequential proof"));
    }

    if prev_public_input.nexus_hash != private_inputs.nexus_header.parent_hash {
        return Err(anyhow::anyhow!(
            "Proof for previous avail height not provided."
        ));
    }

    #[cfg(feature = "zkvm-risc0")]
    match env::verify(img_id.0, &to_vec(&prev_public_input).unwrap()) {
        Ok(()) => {
            println!("Verified proof");
            ()
        }
        Err(e) => return Err(anyhow::anyhow!("Invalid proof")),
    }

    Ok(AdapterPublicInputs {
        nexus_hash,
        height: rollup_public_inputs.height,
        state_root: rollup_public_inputs.post_state_root,
        start_nexus_hash: prev_public_input.start_nexus_hash,
        app_id: app_account_id,
        img_id: img_id.clone(),
        rollup_hash: prev_public_input.rollup_hash
    })
}
