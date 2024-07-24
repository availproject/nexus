//! A simple program that aggregates the proofs of multiple programs proven with the zkVM.

#![no_main]
sp1_zkvm::entrypoint!(main);

use sha2::Digest;
use sha2::Sha256;

pub fn main() {
    // Read the verification keys.
    let vkeys = sp1_zkvm::io::read::<Vec<[u32; 8]>>();

    // Read the public values.
    let public_values = sp1_zkvm::io::read::<Vec<Vec<u8>>>();

    // Verify the proofs.
    assert_eq!(vkeys.len(), public_values.len());
    for i in 0..vkeys.len() {
        let vkey = &vkeys[i];
        let public_values = &public_values[i];
        let public_values_digest = Sha256::digest(public_values);
        sp1_zkvm::lib::verify::verify_sp1_proof(vkey, &public_values_digest.into());
    }

    // TODO: Do something interesting with the proofs here.
    //
    // For example, commit to the verified proofs in a merkle tree. For now, we'll just commit to
    // all the (vkey, input) pairs.
    let commitment = commit_proof_pairs(&vkeys, &public_values);
    sp1_zkvm::io::commit_slice(&commitment);
}
