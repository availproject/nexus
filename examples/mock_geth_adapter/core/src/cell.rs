use crate::crypto::get_header_hash;
use crate::rpc::{request_kate_multi_proof, request_kate_proof};
use crate::utils::{
    cell_count_for_confidence, extract_kate_from_header_extension, generate_random_cells,
};
use avail_rust::avail::runtime_types::avail_core::header::extension::HeaderExtension;
use avail_rust::avail_core::kate::COMMITMENT_SIZE;
use avail_rust::kate_recovery::commitments;
use avail_rust::kate_recovery::matrix::Dimensions;
use avail_rust::{kate_recovery::matrix::Position, AvailHeader};

/// Get cells from block header kate single proof
pub async fn get_cells_for_block_header(
    avail_header: AvailHeader,
) -> (Vec<avail_rust::kate_recovery::data::Cell>, Dimensions) {
    let header_hash = get_header_hash(&avail_header);
    let (dimensions, _, positions) = get_params(avail_header.extension);
    let cells = request_kate_proof(header_hash, &positions)
        .await
        .expect("Kate fetch failed");
    (cells, dimensions)
}

/// Get cells from block header kate multi proof
pub async fn get_cells_for_block_header_multi_proof(
    avail_header: AvailHeader,
) -> (Vec<avail_rust::kate_recovery::data::Cell>, Dimensions) {
    let header_hash = get_header_hash(&avail_header);
    let (dimensions, _, positions) = get_params(avail_header.extension);
    let cells = request_kate_multi_proof(header_hash, &positions)
        .await
        .expect("Kate fetch failed");
    (cells, dimensions)
}

/// Get params for rpc request :
/// - Dimensions
/// - Commitments
/// - Positions
pub fn get_params(
    header_extension: HeaderExtension,
) -> (Dimensions, Vec<[u8; COMMITMENT_SIZE]>, Vec<Position>) {
    let (rows, cols, _, commitment) = extract_kate_from_header_extension(&header_extension)
        .expect("Failed to extract kate_from_header_extension");
    let dimensions = Dimensions::new(rows, cols).expect("Failed to create dimensions");
    let commitments = commitments::from_slice(&commitment).expect("Deserialization failed");
    // Assuming a confidence value of 99.9
    // Value taken from test :
    // https://github.com/availproject/avail-light/blob/35f939093b436b24ef33af3a42c2581009cf7a9a/core/src/light_client.rs#L363
    let cell_count = cell_count_for_confidence(99.9);
    let positions = generate_random_cells(dimensions, cell_count);
    (dimensions, commitments, positions)
}
