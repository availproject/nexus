pub mod hasher;

pub fn u32_to_u8_array(input: &[u32; 8]) -> [u8; 32] {
    let mut output = [0u8; 32];
    for i in 0..8 {
        let bytes = input[i].to_le_bytes();
        output[i * 4..(i + 1) * 4].copy_from_slice(&bytes);
    }
    output
}
