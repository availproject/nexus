use crate::types::ShaHasher;
use crate::types::H256;
use anyhow::Error;
use avail_core::Keccak256;
use binary_merkle_tree::merkle_root;
use libm;

pub fn calculate_data_root(mut data: Vec<Vec<u8>>) -> Result<H256, Error> {
    if data.is_empty() {
        return Ok(H256::zero());
    }

    let card = u32::try_from(data.len())?;
    let next_pow_2 = libm::ceil(libm::log2(card as f64)) as u32;
    let leafs_to_append = usize::try_from(2u32.pow(next_pow_2) - card)?;
    let to_append = vec![H256::zero().as_fixed_slice().to_vec(); leafs_to_append];

    data.extend(to_append);

    let root = merkle_root::<Keccak256, _>(data);

    Ok(H256::from(root.as_fixed_bytes().clone()))
}
