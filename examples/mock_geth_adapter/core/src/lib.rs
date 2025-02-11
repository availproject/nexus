use crate::cell::get_cells_for_block_header;
use crate::crypto::lagrange_interpolation_poly;
use adapter_sdk::traits::RollupProof;
use adapter_sdk::types::RollupPublicInputs;
use avail_rust::AvailHeader;
use dusk_bytes::Serializable;
use dusk_plonk::{fft::EvaluationDomain, prelude::BlsScalar};
use nexus_core::types::H256;
use serde::{Deserialize, Serialize};

pub mod cell;
pub mod crypto;
pub mod rpc;
#[cfg(test)]
mod tests;
pub mod utils;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct DemoProof(pub ());

impl RollupProof for DemoProof {
    fn verify(
        &self,
        vk: &[u8; 32],
        public_inputs: &RollupPublicInputs,
    ) -> Result<(), anyhow::Error> {
        Ok(())
    }
}

/// Construct Polynomial from Cells
/// Returns the polynomial vector of BlsScalar points.
pub async fn get_polynomial_kate_proof(avail_header: AvailHeader) -> Vec<BlsScalar> {
    let (cells, dimensions) = get_cells_for_block_header(avail_header).await;
    let mut points: Vec<BlsScalar> = Vec::new();
    let mut evaluations: Vec<BlsScalar> = Vec::new();

    let width: u16 = dimensions.cols().into();
    let cols = usize::from(width);
    for cell in cells {
        // Represents the point on which the pairing is checked
        let point = EvaluationDomain::new(cols)
            .expect("Failed to create evaluation domain")
            .elements()
            .nth(cell.position.col.into())
            .expect("Failed to create evaluation domain");

        // Represents the result of the evaluation of the point in the polynomial
        let evaluated_point =
            BlsScalar::from_bytes(&cell.data()).expect("Deserialization failed from bytes");

        points.push(point);
        evaluations.push(evaluated_point);
    }

    // Implementing lagrange interpolation
    lagrange_interpolation_poly(points, evaluations)
}

/// Construct Polynomial from Cells (Multi Proof)
/// Returns the polynomial vector of BlsScalar points.
pub async fn get_polynomial_kate_multi_proof(avail_header: AvailHeader) -> Vec<BlsScalar> {
    panic!("Not implemented yet.");
}
