#[cfg(any(feature = "native-risc0", feature = "zkvm-risc0"))]
pub use risc0_zkvm::sha::rust_crypto::Digest;
#[cfg(any(feature = "native-risc0", feature = "zkvm-risc0"))]
pub use risc0_zkvm::sha::rust_crypto::Sha256;
#[cfg(not(any(feature = "native-risc0", feature = "zkvm-risc0")))]
pub use sha2::Digest;
#[cfg(not(any(feature = "native-risc0", feature = "zkvm-risc0")))]
pub use sha2::Sha256;
