use serde::{Deserialize, Serialize};
use std::fmt::Debug;

pub trait ProofTrait: Debug + Clone + Serialize + for<'de> Deserialize<'de> {
    fn verify(&self) -> String;
}

#[derive(Debug)]
pub struct GenericProof<P: ProofTrait> {
    proofs: Vec<P>,
}

impl<P: ProofTrait> GenericProof<P> {
    pub fn new() -> Self {
        Self { proofs: Vec::new() }
    }
}

impl<'de, P> Deserialize<'de> for GenericProof<P>
where
    P: ProofTrait,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let proofs = Vec::<P>::deserialize(deserializer)?;
        Ok(Self { proofs })
    }
}
