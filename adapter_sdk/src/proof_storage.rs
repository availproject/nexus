use std::any::Any;
use std::fmt::Debug;

pub trait ProofTrait: Debug {
    fn prove(&self) -> String;
}

#[derive(Debug)]
pub struct GenericProof<P: ProofTrait> {
    proofs: Vec<P>,
}

impl<P: ProofTrait> GenericProof<P> {
    pub fn new() -> Self {
        Self { proofs: Vec::new() }
    }

    pub fn add_proof(&mut self, proof: P) {
        self.proofs.push(proof);
    }

    pub fn process_proof(&mut self, proof: P) {}
}
