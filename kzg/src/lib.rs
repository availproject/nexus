use std::num::NonZeroUsize;

use anyhow::anyhow;
use ark_ff::Field;
use bls12_381::{multi_miller_loop, G1Affine, G1Projective, G2Affine, G2Prepared, G2Projective, Gt, Scalar};
use ff::derive::sbb;
use kate::gridgen::AsBytes;
use kate::pmp::ark_poly::univariate::{DenseOrSparsePolynomial, DensePolynomial};
use kate::pmp::ark_poly::DenseUVPolynomial;
use kate::pmp::ark_serialize::CanonicalSerialize;
use kate::pmp::m1_blst::{Fr, G1Affine as ArkG1Affine, M1NoPrecomp, Proof};
use kate::pmp::traits::KZGProof;
use kate::{
    couscous,
    gridgen::ArkScalar,
    pmp::ark_poly::{EvaluationDomain, GeneralEvaluationDomain},
};
use kate_recovery::data::Cell;

pub use kate;
pub use kate_recovery;

fn poly_div_q_r<F: Field>(num: DenseOrSparsePolynomial<F>, denom: DenseOrSparsePolynomial<F>) -> Result<(Vec<F>, Vec<F>), anyhow::Error> {
    if denom.is_zero() {
        return Err(anyhow!("Divisor is zero"));
    }
    let (q, r) = num.divide_with_q_and_r(&denom).expect("Cannot return none");
    Ok((q.coeffs, r.coeffs))
}

pub fn compute_row_proof(blob: &Vec<[u8; 32]>) -> Result<[u8; 48], anyhow::Error> {
    let public_params = couscous::multiproof_params();
    let row_eval_poly: Vec<Fr> = blob
        .iter()
        .map(|cell| {
            //TODO: add error handling
            let y = Fr::from_bytes(cell).unwrap();

            y
        })
        .collect();

    let domain = GeneralEvaluationDomain::<ArkScalar>::new(blob.len())
        .ok_or(anyhow!("Domain size invalid"))
        .unwrap();
    let co_eff_polynomial = domain.ifft(&row_eval_poly);

    //TODO: Change to actuall challenge calculations.
    let evaluation_challenge = {
        let fr = Fr::from_bytes(&blob[0]);

        fr.map_err(|_| anyhow!("Failed to convert Scalar to Fr"))?
    };

    let challenge_proof = compute_kzg_proof(&public_params, co_eff_polynomial, evaluation_challenge)?;
    let mut challenge_proof_serialized = [0u8; 48];

    challenge_proof
        .serialize_compressed(&mut challenge_proof_serialized[..])
        .map_err(|_| anyhow!("serialization of g1 failed"))?;

    Ok(challenge_proof_serialized)
}

pub fn compute_kzg_proof(srs: &M1NoPrecomp, row_poly: Vec<Fr>, challenge_evaluation: Fr) -> Result<Proof, anyhow::Error> {
    use ark_ff::One;
    let poly = DensePolynomial::from_coefficients_vec(row_poly);
    let divisor = DensePolynomial::from_coefficients_vec(vec![-challenge_evaluation, Fr::one()]);
    let (q, _r) = poly_div_q_r(poly.into(), divisor.into())?;

    let witness = q;

    Ok(KZGProof::open(srs, witness).or_else(|_| Err(anyhow!("Failed to open proof")))?)
}

pub fn verify_row_kzg(row: &Vec<[u8; 32]>, commitment: &[u8; 48], proof_bytes: &[u8; 48]) -> Result<bool, anyhow::Error> {
    let commitment = safe_g1_affine_from_bytes(commitment)?;
    let proof = safe_g1_affine_from_bytes(proof_bytes)?;

    let polynomial: Vec<Scalar> = row
        .iter()
        .map(|cell| {
            //TODO: add error handling
            let y = Scalar::from_bytes(&cell).unwrap();

            y
        })
        .collect();

    //TODO: Change to actuall challenge calculations.
    let evaluation_challenge = Scalar::from_bytes(&row[0]).unwrap();

    let y = evaluate_polynomial_in_evaluation_form(polynomial, evaluation_challenge);

    verify_kzg_proof_impl(
        commitment,
        evaluation_challenge,
        y.unwrap(),
        proof,
        G2Affine::from_compressed(&G2_POINT).expect("Failed to form G2Affine point"),
    )
}

pub fn safe_g1_affine_from_bytes(bytes: &[u8; 48]) -> Result<G1Affine, anyhow::Error> {
    let g1 = G1Affine::from_compressed(&bytes);
    if g1.is_none().into() {
        return Err(anyhow!("Failed to parse G1Affine from bytes".to_string(),));
    }
    Ok(g1.unwrap())
}

pub fn evaluate_polynomial_in_evaluation_form(polynomial: Vec<Scalar>, x: Scalar) -> Result<Scalar, anyhow::Error> {
    let eval_domain = GeneralEvaluationDomain::<ArkScalar>::new(polynomial.len()).unwrap();

    let eval_domain_in_scalar: Vec<Scalar> = eval_domain
        .elements()
        .map(|e| {
            let scalar = Scalar::from_bytes(&e.to_bytes().expect("Failed to convert element to bytes")).unwrap();

            scalar
        })
        .collect();

    //TODO: Length check
    // if polynomial.len() != NUM_FIELD_ELEMENTS_PER_BLOB {
    //     return Err(KzgError::InvalidBytesLength(
    //         "The polynomial length is incorrect".to_string(),
    //     ));
    // }

    let mut inverses_in = vec![Scalar::default(); polynomial.len()];
    let mut inverses = vec![Scalar::default(); polynomial.len()];
    for i in 0..polynomial.len() {
        if x == eval_domain_in_scalar[i] {
            return Ok(polynomial[i]);
        }
        inverses_in[i] = x - eval_domain_in_scalar[i];
    }

    batch_inversion(
        &mut inverses,
        &inverses_in,
        NonZeroUsize::new(polynomial.len()).unwrap(),
    )?;

    let mut out = Scalar::zero();

    for i in 0..polynomial.len() {
        out += (inverses[i] * eval_domain_in_scalar[i]) * polynomial[i];
    }

    out *= Scalar::from(polynomial.len() as u64).invert().unwrap();
    out *= x.pow(&[polynomial.len() as u64, 0, 0, 0]) - Scalar::one();

    Ok(out)
}

fn batch_inversion(out: &mut [Scalar], a: &[Scalar], len: NonZeroUsize) -> Result<(), anyhow::Error> {
    if a == out {
        return Err(anyhow!("Destination is the same as source".to_string(),));
    }

    // Compute the product of all the elements:
    //
    // \[
    // P = x_1 \times x_2 \times \dots \times x_n
    // \]

    let mut accumulator = Scalar::one();

    for i in 0..len.into() {
        out[i] = accumulator;
        accumulator = accumulator.mul(&a[i]);
    }

    if accumulator == Scalar::zero() {
        return Err(anyhow!("Zero input".to_string()));
    }

    // Compute the inverse of the product \( P \):
    //
    // \[
    // P^{-1} = \text{inverse}(P)
    // \]
    accumulator = accumulator.invert().unwrap();

    // Compute the inverse of each element \( x_i^{-1} \) by using the precomputed product and its inverse:
    //
    // \[
    // x_i^{-1} = P^{-1} \times \left(\prod_{j \neq i} x_j \right)
    // \]
    for i in (0..len.into()).rev() {
        out[i] *= accumulator;
        accumulator *= a[i];
    }

    Ok(())
}

pub fn verify_kzg_proof_impl(
    commitment: G1Affine,
    z: Scalar,
    y: Scalar,
    proof: G1Affine,
    trusted_setup_g2_point: G2Affine,
) -> Result<bool, anyhow::Error> {
    let x = G2Projective::generator() * z;
    let x_minus_z = trusted_setup_g2_point - x;

    let y = G1Projective::generator() * y;
    let p_minus_y = commitment - y;

    // Verify: P - y = Q * (X - z)
    Ok(pairings_verify(
        p_minus_y.into(),
        G2Projective::generator().into(),
        proof,
        x_minus_z.into(),
    ))
}

pub fn pairings_verify(a1: G1Affine, a2: G2Affine, b1: G1Affine, b2: G2Affine) -> bool {
    multi_miller_loop(&[(&-a1, &G2Prepared::from(a2)), (&b1, &G2Prepared::from(b2))]).final_exponentiation() == Gt::identity()
}

pub fn scalar_from_bytes_unchecked(bytes: [u8; 32]) -> Scalar {
    scalar_from_u64_array_unchecked([
        u64::from_be_bytes(<[u8; 8]>::try_from(&bytes[0..8]).unwrap()),
        u64::from_be_bytes(<[u8; 8]>::try_from(&bytes[8..16]).unwrap()),
        u64::from_be_bytes(<[u8; 8]>::try_from(&bytes[16..24]).unwrap()),
        u64::from_be_bytes(<[u8; 8]>::try_from(&bytes[24..32]).unwrap()),
    ])
}

pub fn scalar_from_u64_array_unchecked(array: [u64; 4]) -> Scalar {
    // Try to subtract the modulus
    let (_, borrow) = sbb(array[0], MODULUS[0], 0);
    let (_, borrow) = sbb(array[1], MODULUS[1], borrow);
    let (_, borrow) = sbb(array[2], MODULUS[2], borrow);
    let (_, _borrow) = sbb(array[3], MODULUS[3], borrow);

    Scalar::from_raw([array[3], array[2], array[1], array[0]])
}

/// Constant representing the modulus
/// q = 0x73eda753299d7d483339d80809a1d80553bda402fffe5bfeffffffff00000001
pub const MODULUS: [u64; 4] = [0xffff_ffff_0000_0001, 0x53bd_a402_fffe_5bfe, 0x3339_d808_09a1_d805, 0x73ed_a753_299d_7d48];

pub const G2_POINT: [u8; 96] = [
    177, 96, 16, 241, 179, 211, 58, 165, 243, 246, 8, 195, 133, 64, 9, 121, 237, 54, 220, 63, 171, 83, 211, 98, 111, 81, 156, 116, 215, 225, 235,
    239, 98, 201, 173, 53, 121, 100, 4, 206, 47, 195, 235, 180, 33, 103, 181, 161, 5, 10, 194, 124, 53, 39, 7, 43, 129, 175, 15, 169, 166, 131, 226,
    38, 218, 121, 39, 95, 41, 133, 201, 232, 154, 13, 185, 105, 183, 110, 11, 107, 233, 79, 238, 135, 76, 72, 88, 103, 55, 173, 85, 136, 207, 30, 16,
    65,
];
