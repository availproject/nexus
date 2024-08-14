#[cfg(any(feature = "risc0-native", feature = "zkvm-risc0"))]
pub use risc0_zkvm::sha::rust_crypto::Digest;
#[cfg(any(feature = "risc0-native", feature = "zkvm-risc0"))]
pub use risc0_zkvm::sha::rust_crypto::Sha256;
// #[cfg(not(any(feature = "risc0-native", feature = "zkvm-risc0")))]
// pub use sha2::Digest;
// #[cfg(not(any(feature = "risc0-native", feature = "zkvm-risc0")))]
// pub use sha2::Sha256;
