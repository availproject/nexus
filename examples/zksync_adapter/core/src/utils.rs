use zksync_basic_types::{Address, H256, U128, U256};
#[cfg(any(feature = "native"))]
pub use zksync_types::commitment::serialize_commitments;

pub fn read_address(bytes: &[u8], start: usize) -> (Address, U256) {
    let mut address = [0u8; 20];
    address.copy_from_slice(&bytes[start..start + 20]);
    let offset = start + 20;
    (Address::from(address), offset.into())
}

pub fn read_uint256(bytes: &[u8], start: usize) -> (U256, U256) {
    let mut uint256_bytes = [0u8; 32];
    uint256_bytes.copy_from_slice(&bytes[start..start + 32]);

    let mut result = 0;
    for &byte in &uint256_bytes[16..] {
        // Take the last 16 bytes (128 bits)
        result = (result << 8) | (byte as u128);
    }

    let offset = start + 32;
    (U256::from(result), U256::from(offset))
}

pub fn read_bytes32(bytes: &[u8], start: usize) -> (H256, U256) {
    let mut result = [0u8; 32];
    result.copy_from_slice(&bytes[start..start + 32]);
    let offset = start + 32;
    (H256::from(result), offset.into())
}
