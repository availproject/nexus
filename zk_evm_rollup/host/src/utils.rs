use ark_bn254::{
    g1, g1::Parameters, Bn254, Fq, FqParameters, Fr, FrParameters, G1Projective, G2Projective,
};
use ark_bn254::{g2, Fq2, Fq2Parameters, G2Affine};
use ark_ec::short_weierstrass_jacobian::GroupAffine;
use ark_ec::*;
use ark_ff::{
    field_new, Field, Fp256, Fp256Parameters, Fp2ParamsWrapper, One, PrimeField, QuadExtField,
    UniformRand, Zero,
};

use num_bigint::*;

pub fn get_bigint_from_fr(fr: Fp256<FrParameters>) -> BigInt {
    let mut st = fr.to_string();
    let temp = &st[8..8 + 64];
    BigInt::parse_bytes(temp.as_bytes(), 16).unwrap()
}


pub fn get_u8_arr_from_str(num_str: &str) -> [u8; 32] {
    //convert bigint to [u8; 32]
    let bigint = BigInt::parse_bytes(num_str.as_bytes(), 10).unwrap();

    // Get the bytes from the BigInt
    let bytes = bigint.to_bytes_le().1;

    // Convert the bytes to a fixed size array
    let mut arr = [0u8; 32];
    
    let num_bytes = bytes.len().min(32);
    arr[..num_bytes].copy_from_slice(&bytes[..num_bytes]);
    // arr.copy_from_slice(&bytes);

    arr
}

pub fn get_u8_arr_from_fr(fr: Fp256<FrParameters>) -> [u8; 32] {
    let bytes = get_bigint_from_fr(fr).to_bytes_le().1;

    // Convert the bytes to a fixed size array
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&bytes);

    arr
}