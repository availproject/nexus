#![no_main]

use risc0_zkvm::guest::env;
risc0_zkvm::guest::entry!(main);

fn main() {
    // First read the length of the data
    let length: usize = env::read();
    println!("Expected length: {}", length);
    
    // Now allocate a vector of the correct size and read the data
    let mut bytes = vec![0u8; length];
    env::read_slice(&mut bytes);

    println!("Bytes read: {:?}", &bytes);
    env::commit_slice(&bytes);
}
