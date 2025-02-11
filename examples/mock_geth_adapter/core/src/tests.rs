use crate::{
    cell::{get_cells_for_block_header, get_cells_for_block_header_multi_proof},
    get_polynomial_kate_proof,
    utils::read_block_data,
};

#[tokio::test]
#[rstest::rstest]
async fn test_number_of_cells() {
    let avail_header = read_block_data();
    let (cells, dimensions) = get_cells_for_block_header(avail_header).await;

    // comparing with data in json
    assert_eq!(dimensions.cols().get(), 512);
    assert_eq!(dimensions.rows().get(), 16);

    // asserting the number of cells
    assert_eq!(cells.len(), 10);
}

#[tokio::test]
#[rstest::rstest]
#[ignore]
async fn test_number_of_cells_multi_proof() {
    let avail_header = read_block_data();
    let (cells, dimensions) = get_cells_for_block_header_multi_proof(avail_header).await;

    println!(">>> Cells Length : {:?}", cells.len());
    println!(">>> Dimensions : {:?}", dimensions);

    // asserting the number of cells
    // assert_eq!(cells.len(), 10);
}

#[tokio::test]
#[rstest::rstest]
async fn test_lagrange_polynomial() {
    let avail_header = read_block_data();
    let polynomial = get_polynomial_kate_proof(avail_header).await;

    // For this current avail header there are 10 cells
    // and this means 10 points thus lagrange poly. should
    // have 10 coefficients (degree 9 polynomial).
    assert_eq!(polynomial.len(), 10);
}
