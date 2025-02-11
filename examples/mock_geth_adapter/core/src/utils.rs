use avail_rust::avail::runtime_types::avail_core::header::extension::HeaderExtension;
use avail_rust::kate_recovery::matrix::Dimensions;
use avail_rust::kate_recovery::matrix::Position;
use avail_rust::subxt_signer::bip39::rand::{thread_rng, Rng};
use avail_rust::{AvailHeader, H256};
use dusk_plonk::commitment_scheme::kzg10::PublicParameters;
use std::collections::HashSet;

/// Read Public Params
pub fn read_public_params() -> PublicParameters {
    let pp_bytes = include_bytes!("./assets/pp.data");
    PublicParameters::from_slice(pp_bytes)
        .expect("Deserializing of public parameters should work for serialised pp")
}

/// Read the call.json file containing a block with data submissions
pub fn read_block_data() -> AvailHeader {
    let json_file = include_bytes!("./assets/call.json");
    serde_json::from_slice(json_file).expect("Deserialization failed")
}

// ========================================
// Header Extraction Utils
// ========================================

// Value taken from : https://github.com/availproject/avail-light/blob/35f939093b436b24ef33af3a42c2581009cf7a9a/core/src/network/rpc.rs#L246
pub const CELL_COUNT_99_99: u32 = 14;

// Function from : https://github.com/availproject/avail-light/blob/2832aaa41cd8c1fe4086bbc4978a63a366a52766/core/src/utils.rs#L64
pub fn extract_kate_from_header_extension(
    extension: &HeaderExtension,
) -> Option<(u16, u16, H256, Vec<u8>)> {
    match extension {
        HeaderExtension::V3(header) => {
            let kate = &header.commitment;
            Some((
                kate.rows,
                kate.cols,
                kate.data_root,
                kate.commitment.clone(),
            ))
        }
    }
}

// Function from : https://github.com/availproject/avail-light/blob/35f939093b436b24ef33af3a42c2581009cf7a9a/core/src/network/rpc.rs#L249
pub fn cell_count_for_confidence(confidence: f64) -> u32 {
    let mut cell_count: u32;
    if !(50.0..=100f64).contains(&confidence) {
        cell_count = (-((1f64 - (99.3f64 / 100f64)).log2())).ceil() as u32;
    } else {
        cell_count = (-((1f64 - (confidence / 100f64)).log2())).ceil() as u32;
    }
    if cell_count <= 1 {
        cell_count = 1;
    } else if cell_count > CELL_COUNT_99_99 {
        cell_count = CELL_COUNT_99_99;
    }
    cell_count
}

// Function taken from :
// https://github.com/availproject/avail-light/blob/35f939093b436b24ef33af3a42c2581009cf7a9a/core/src/network/rpc.rs#L224
pub fn generate_random_cells(dimensions: Dimensions, cell_count: u32) -> Vec<Position> {
    let max_cells = dimensions.extended_size();
    let count = if max_cells < cell_count {
        max_cells
    } else {
        cell_count
    };
    let mut rng = thread_rng();
    let mut indices = HashSet::new();
    while (indices.len() as u16) < count as u16 {
        let col = rng.gen_range(0..dimensions.cols().into());
        let row = rng.gen_range(0..dimensions.extended_rows());
        indices.insert(Position { row, col });
    }

    indices.into_iter().collect::<Vec<_>>()
}
