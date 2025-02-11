use avail_core::{BlockLengthColumns, BlockLengthRows};
use avail_rust::avail::runtime_types::frame_system::limits::BlockLength;
use avail_rust::avail_core::kate::COMMITMENT_SIZE;
use avail_rust::primitives::kate::{GProof, GRawScalar};
use avail_rust::subxt::ext::codec::Encode;
use avail_rust::{AvailHeader, H256};
use dusk_bytes::Serializable;
use dusk_plonk::bls12_381::{BlsScalar, G1Affine};
use dusk_plonk::commitment_scheme::kzg10::commitment::Commitment;
use dusk_plonk::commitment_scheme::kzg10::proof::Proof;
use dusk_plonk::commitment_scheme::PublicParameters;
use dusk_plonk::fft::EvaluationDomain;
use kate::gridgen::{ArkScalar, AsBytes, EvaluationGrid};
use kate::M1NoPrecomp;
use kate_recovery::com::*;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::num::NonZeroU16;

/// To get the header hash from avail header
pub fn get_header_hash(avail_header: &AvailHeader) -> H256 {
    Encode::using_encoded(avail_header, blake2_256).into()
}

/// To calculate blake2_256 from a data slice
#[allow(dead_code)]
pub fn blake2_256(data: &[u8]) -> [u8; 32] {
    avail_rust::sp_core::blake2_256(data)
}

/// Verify the KZG commitment for a cell
pub fn verify(
    public_parameters: &PublicParameters,
    width: usize,
    commitment: &[u8; COMMITMENT_SIZE],
    cell: &avail_rust::kate_recovery::data::Cell,
) -> bool {
    let commitment_to_witness = G1Affine::from_bytes(&cell.proof())
        .map(Commitment::from)
        .expect("Deserialization failed from bytes");
    let evaluated_point =
        BlsScalar::from_bytes(&cell.data()).expect("Deserialization failed from bytes");
    let commitment_to_polynomial = G1Affine::from_bytes(commitment)
        .map(Commitment::from)
        .expect("Deserialization failed from bytes");

    let proof = Proof {
        commitment_to_witness,
        evaluated_point,
        commitment_to_polynomial,
    };

    let point = EvaluationDomain::new(width)
        .expect("Failed to create evaluation domain")
        .elements()
        .nth(cell.position.col.into())
        .expect("Failed to create evaluation domain");

    public_parameters.opening_key().check(point, proof)
}

/// To calculate lagrange interpolation polynomial from point and their evaluations.
pub fn lagrange_interpolation_poly(
    points: Vec<BlsScalar>,
    values: Vec<BlsScalar>,
) -> Vec<BlsScalar> {
    if points.len() != values.len() {
        panic!("Number of points and values must be the same length")
    }

    let mut result = vec![BlsScalar::zero(); values.len()];

    for i in 0..points.len() {
        let mut numerator = vec![BlsScalar::one()];
        let mut denominator = BlsScalar::one();

        for j in 0..points.len() {
            if i == j {
                continue;
            }

            numerator = mul(&numerator, &[-points[j], BlsScalar::one()]);
            denominator *= points[i] - points[j];
        }

        let denominator_inv = denominator.invert().unwrap();
        let term: Vec<BlsScalar> = numerator
            .iter()
            .map(|&a| a * values[i] * denominator_inv)
            .collect();

        result = add(&result, &term);
    }

    result
}

// =================================
// Polynomial Helper Functions
// =================================

// helper function for polynomial multiplication
fn mul(p1: &[BlsScalar], p2: &[BlsScalar]) -> Vec<BlsScalar> {
    let mut result = vec![BlsScalar::zero(); p1.len() + p2.len() - 1];
    for (i, &coeff1) in p1.iter().enumerate() {
        for (j, &coeff2) in p2.iter().enumerate() {
            result[i + j] += coeff1 * coeff2;
        }
    }
    result
}

// helper function for polynomial addition
pub fn add(p1: &[BlsScalar], p2: &[BlsScalar]) -> Vec<BlsScalar> {
    let mut result = vec![BlsScalar::zero(); std::cmp::max(p1.len(), p2.len())];
    for (i, &coeff) in p1.iter().enumerate() {
        result[i] += coeff;
    }
    for (i, &coeff) in p2.iter().enumerate() {
        result[i] += coeff;
    }
    result
}

// =================================
// Multi Proof Implementation
// Taken from avail-core, avail, avail-rust PRs
// =================================

pub const SEED: [u8; 32] = [100; 32];
pub const MIN_WIDTH: usize = 4;
pub type GMultiProof = (Vec<GRawScalar>, GProof);

pub fn get_multi_proof(
    extrinsics: Vec<avail_core::AppExtrinsic>,
    block_len: BlockLength,
    _seed: [u8; 32],
    cells: Vec<(u32, u32)>,
) -> color_eyre::Result<Vec<GMultiProof>> {
    let multi_proof_params: M1NoPrecomp = kate::couscous::multiproof_params();
    let (max_width, max_height) = to_width_height(&block_len);
    let grid = EvaluationGrid::from_extrinsics(extrinsics, MIN_WIDTH, max_width, max_height, SEED)
        .expect("Error creating grid")
        .extend_columns(NonZeroU16::new(2).expect("2>0"))
        .expect("Error creating grid");
    let poly = grid
        .make_polynomial_grid()
        .expect("Error creating polynomial grid");

    let proofs = cells
        .into_par_iter()
        .map(|(row, col)| -> color_eyre::Result<GMultiProof> {
            let cell = kate::com::Cell::new(BlockLengthRows(row), BlockLengthColumns(col));

            println!(">>> cell : {:?}", cell);

            let target_dimensions =
                kate_recovery::matrix::Dimensions::new(16, 64).expect("16,64 > 0");
            if cell.row.0 >= grid.dims().height() as u32 || cell.col.0 >= grid.dims().width() as u32
            {
                panic!("Missing Cell");
            }
            let mp = poly
                .multiproof(&multi_proof_params, &cell, &grid, target_dimensions)
                .expect("Failed to poly multi proof");
            let data = mp
                .evals
                .into_iter()
                .flatten()
                .map(|e: ArkScalar| e.to_bytes().map(GRawScalar::from))
                .collect::<color_eyre::Result<Vec<GRawScalar>, _>>()?;

            let proof = mp.proof.to_bytes().map(GProof)?;

            Ok((data, proof))
        })
        .collect::<color_eyre::Result<Vec<GMultiProof>, _>>()?;

    Ok(proofs)
}

fn to_width_height(block_len: &BlockLength) -> (usize, usize) {
    // even if we run on a u16 target this is fine
    let width = block_len.cols.0.saturated_into();
    let height = block_len.rows.0.saturated_into();
    (width, height)
}
