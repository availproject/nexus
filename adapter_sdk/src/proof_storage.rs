use anyhow::Error;
use nexus_core::traits::{Proof, RollupPublicInputs};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::marker::PhantomData;

#[derive(Debug, Clone, Copy)]
pub struct GenericProof<PI, P> {
    proof: P,
    _marker: PhantomData<PI>,
}

impl<PI, P> Proof<PI> for GenericProof<PI, P>
// Ensure we specify `GenericProof<PI, P>` here
where
    PI: RollupPublicInputs,
    P: Proof<PI>, // This ensures `P` implements the Proof trait for some PI
{
    fn verify(&self, vk: &[u8; 32], public_inputs: &PI) -> Result<(), Error> {
        // Forward the call to P's verify method
        self.proof.verify(vk, public_inputs)
    }
}
