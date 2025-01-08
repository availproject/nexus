use crate::types::H256;
#[cfg(any(feature = "native-risc0", feature = "zkvm-risc0"))]
pub use risc0_zkvm::sha::rust_crypto::Digest;
#[cfg(any(feature = "native-risc0", feature = "zkvm-risc0"))]
pub use risc0_zkvm::sha::rust_crypto::Sha256;
#[cfg(not(any(feature = "native-risc0", feature = "zkvm-risc0")))]
pub use sha2::Digest;
#[cfg(not(any(feature = "native-risc0", feature = "zkvm-risc0")))]
pub use sha2::Sha256;

pub struct ShaHasher(pub Sha256);

impl ShaHasher {
    pub fn new() -> Self {
        Self(Sha256::new())
    }

    pub fn write_h256(&mut self, h: &H256) {
        self.0.update(h.as_slice())
    }

    pub fn write_byte(&mut self, b: u8) {
        self.0.update([b])
    }

    pub fn finish(self) -> H256 {
        let bytes = self.0.finalize();
        let sha2_array: [u8; 32] = bytes
            .as_slice()
            .try_into()
            .expect("Hash should only be 32 bytes");
        H256::from(sha2_array)
    }
}
