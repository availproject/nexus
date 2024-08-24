use zksync_core::verifier::ZksyncVerifier;

fn main() {
    let verifier = ZksyncVerifier::new();
    verifier.verify();
}