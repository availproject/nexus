use adapter_sdk::traits::{Proof, RollupPublicInputs};
use ark_bn254::{Fq, Fq2, G2Affine};
use nexus_core::types::H256;
use serde::{Deserialize, Serialize, Deserializer};

use ark_bn254::{g1, g1::Parameters, Bn254, FqParameters, Fr, FrParameters, G1Projective};
use ark_ec::short_weierstrass_jacobian::GroupAffine;
use ark_ec::*;
use ark_ff::{Field, Fp256, Fp256Parameters, One, PrimeField, UniformRand, Zero};
use std::str::FromStr;
use std::ops::{Add, Mul, Neg, Sub};
pub mod utils;
use utils::*;

pub type G1Point = <Bn254 as PairingEngine>::G1Affine;
pub type G2Point = <Bn254 as PairingEngine>::G2Affine;

use tiny_keccak::{Hasher, Keccak};
use num_bigint::{BigInt, BigUint};




pub struct LISValues {
    pub li_s0_inv: [Fp256<FrParameters>; 8],
    pub li_s1_inv: [Fp256<FrParameters>; 4],
    pub li_s2_inv: [Fp256<FrParameters>; 6],
}

pub struct Challenges {
    pub alpha: Fp256<FrParameters>,
    pub beta: Fp256<FrParameters>,
    pub gamma: Fp256<FrParameters>,
    pub y: Fp256<FrParameters>,
    pub xiSeed: Fp256<FrParameters>,
    pub xiSeed2: Fp256<FrParameters>,
    pub xi: Fp256<FrParameters>,
}

pub struct Roots {
    pub h0w8: [Fp256<FrParameters>; 8],
    pub h1w4: [Fp256<FrParameters>; 4],
    pub h2w3: [Fp256<FrParameters>; 3],
    pub h3w3: [Fp256<FrParameters>; 3],
}

pub struct VerifierProcessedInputs {
    pub c0x: BigInt,
    pub c0y: BigInt,
    pub x2x1: BigInt,
    pub x2x2: BigInt,
    pub x2y1: BigInt,
    pub x2y2: BigInt,
}

fn fr_parameter_to_hex_string(hex_string: String) -> [u8; 32] {
    // Convert the value to a hexadecimal string
    // let hex_string = value.to_string();

    // Extract the desired bits (8 to 72 characters) and prepend "0x"
    let substring = format!("0x{}", &hex_string[8..72]);

    substring.as_bytes().try_into().unwrap()
}

pub fn compute_challenges(
    challenges: &mut Challenges, roots: &mut Roots, mut zh: &mut Fp256<FrParameters>, zhinv: &mut Fp256<FrParameters>, vpi: VerifierProcessedInputs, pubSignals: BigInt
){
    let mut hasher = Keccak::v256();

    let val1 = vpi.c0x.to_bytes_be();
    let val2 = vpi.c0y.to_bytes_be();
    let val3 = pubSignals.to_bytes_be();
    let val4 = get_proog_bigint().c1.0.to_bytes_be();
    let val5 = get_proog_bigint().c1.1.to_bytes_be();

    let mut concatenated = Vec::new();
    concatenated.extend_from_slice(&padd_bytes32(val1.1));
    concatenated.extend_from_slice(&padd_bytes32(val2.1));
    concatenated.extend_from_slice(&padd_bytes32(val3.1));
    concatenated.extend_from_slice(&padd_bytes32(val4.1));
    concatenated.extend_from_slice(&padd_bytes32(val5.1));

    hasher.update(&concatenated);

    let mut out = [0u8; 32];
    hasher.finalize(&mut out);
    let _beta = BigInt::from_bytes_be(num_bigint::Sign::Plus, &out);

    let beta = Fr::from_str(&_beta.to_string()).unwrap();

    //gamma
    hasher = Keccak::v256();

    let _beta_string = beta.to_string();
    let beta_string = &_beta_string[8..8+64];
    let val6 = BigInt::parse_bytes(beta_string.trim_start_matches("0x").as_bytes(), 16).unwrap().to_bytes_be();
    concatenated = Vec::new();
    concatenated.extend_from_slice(&padd_bytes32(val6.1));
    hasher.update(&concatenated);
    out = [0u8; 32];
    hasher.finalize(&mut out);
    let _gamma = BigInt::from_bytes_be(num_bigint::Sign::Plus, &out);
    let gamma = Fr::from_str(&_gamma.to_string()).unwrap();

    //xiseed 
    let mut hasher3 = Keccak::v256();
    let _gamma_string = gamma.to_string();
    let gamma_string = &_gamma_string[8..8+64];
    // println!("gamma_string: {:?}", gamma_string);
    let val7 = BigInt::parse_bytes(gamma_string.as_bytes(), 16).unwrap().to_bytes_be();
    let val8 = get_proog_bigint().c2.0.to_bytes_be();
    let val9 = get_proog_bigint().c2.1.to_bytes_be();

    concatenated = Vec::new();
    concatenated.extend_from_slice(&padd_bytes32(val7.1));
    concatenated.extend_from_slice(&padd_bytes32(val8.1));
    concatenated.extend_from_slice(&padd_bytes32(val9.1));

    hasher3.update(&concatenated);
    out = [0u8; 32];
    hasher3.finalize(&mut out);
    let _xiSeed = BigInt::from_bytes_be(num_bigint::Sign::Plus, &out);
    let xiSeed = Fr::from_str(&_xiSeed.to_string()).unwrap();

    // println!("xiSeed: {:?}", xiSeed.to_string());

    //xiSeed2
    let mut xiSeed2 = xiSeed.mul(xiSeed);
    // println!("xiSeed2: {:?}", xiSeed2.to_string());

    //roots h0w8
    roots.h0w8[0] = xiSeed2.mul(xiSeed);
    roots.h0w8[1] = roots.h0w8[0].mul(get_omegas().w8_1);
    roots.h0w8[2] = roots.h0w8[0].mul(get_omegas().w8_2);
    roots.h0w8[3] = roots.h0w8[0].mul(get_omegas().w8_3);
    roots.h0w8[4] = roots.h0w8[0].mul(get_omegas().w8_4);
    roots.h0w8[5] = roots.h0w8[0].mul(get_omegas().w8_5);
    roots.h0w8[6] = roots.h0w8[0].mul(get_omegas().w8_6);
    roots.h0w8[7] = roots.h0w8[0].mul(get_omegas().w8_7);

    //roots h1w4
    roots.h1w4[0] = roots.h0w8[0].mul(roots.h0w8[0]);
    roots.h1w4[1] = roots.h1w4[0].mul(get_omegas().w4);
    roots.h1w4[2] = roots.h1w4[0].mul(get_omegas().w4_2);
    roots.h1w4[3] = roots.h1w4[0].mul(get_omegas().w4_3);

    //roots h2w3
    roots.h2w3[0] = roots.h1w4[0].mul(xiSeed2);
    roots.h2w3[1] = roots.h2w3[0].mul(get_omegas().w3);
    roots.h2w3[2] = roots.h2w3[0].mul(get_omegas().w3_2);

    //roots h3w3
    roots.h3w3[0] = roots.h2w3[0].mul(get_omegas().wr);
    roots.h3w3[1] = roots.h3w3[0].mul(get_omegas().w3);
    roots.h3w3[2] = roots.h3w3[0].mul(get_omegas().w3_2);


    //zh and zhInv
    let mut xin = roots.h2w3[0].mul(roots.h2w3[0]).mul(roots.h2w3[0]);
    let mut Xin = xin;
    for _ in 0..24{
        xin = xin.mul(xin);
    }

    xin = xin.sub(Fr::one());

    *zh = xin;
    *zhinv = xin;
    // println!("zh: {:?}", zh.to_string());

    // alpha
    let mut hasher4 = Keccak::v256();

    let _xiseed_string = xiSeed.to_string();
    let xiseed_string = &_xiseed_string[8..8+64];
    // let val6 = BigInt::parse_bytes(beta_string.trim_start_matches("0x").as_bytes(), 16).unwrap().to_bytes_be();
    let val10 = BigInt::parse_bytes(xiseed_string.to_string().as_bytes(), 16).unwrap().to_bytes_be();
    
    let val11 = get_proog_bigint().eval_ql.to_bytes_be();
    let val12 = get_proog_bigint().eval_qr.to_bytes_be();
    let val13 = get_proog_bigint().eval_qm.to_bytes_be();
    let val14 = get_proog_bigint().eval_qo.to_bytes_be();
    let val15 = get_proog_bigint().eval_qc.to_bytes_be();
    let val16 = get_proog_bigint().eval_s1.to_bytes_be();
    let val17 = get_proog_bigint().eval_s2.to_bytes_be();
    let val18 = get_proog_bigint().eval_s3.to_bytes_be();
    let val19 = get_proog_bigint().eval_a.to_bytes_be();
    let val20 = get_proog_bigint().eval_b.to_bytes_be();
    let val21 = get_proog_bigint().eval_c.to_bytes_be();
    let val22 = get_proog_bigint().eval_z.to_bytes_be();
    let val23 = get_proog_bigint().eval_zw.to_bytes_be();
    let val24 = get_proog_bigint().eval_t1w.to_bytes_be();
    let val25 = get_proog_bigint().eval_t2w.to_bytes_be();

    concatenated = Vec::new();
    concatenated.extend_from_slice(&padd_bytes32(val10.1));
    concatenated.extend_from_slice(&padd_bytes32(val11.1));
    concatenated.extend_from_slice(&padd_bytes32(val12.1));
    concatenated.extend_from_slice(&padd_bytes32(val13.1));
    concatenated.extend_from_slice(&padd_bytes32(val14.1));
    concatenated.extend_from_slice(&padd_bytes32(val15.1));
    concatenated.extend_from_slice(&padd_bytes32(val16.1));
    concatenated.extend_from_slice(&padd_bytes32(val17.1));
    concatenated.extend_from_slice(&padd_bytes32(val18.1));
    concatenated.extend_from_slice(&padd_bytes32(val19.1));
    concatenated.extend_from_slice(&padd_bytes32(val20.1));
    concatenated.extend_from_slice(&padd_bytes32(val21.1));
    concatenated.extend_from_slice(&padd_bytes32(val22.1));
    concatenated.extend_from_slice(&padd_bytes32(val23.1));
    concatenated.extend_from_slice(&padd_bytes32(val24.1));
    concatenated.extend_from_slice(&padd_bytes32(val25.1));

    hasher4.update(&concatenated);

    out = [0u8; 32];
    hasher4.finalize(&mut out);
    let _alpha = BigInt::from_bytes_be(num_bigint::Sign::Plus, &out);
    let alpha = Fr::from_str(&_alpha.to_string()).unwrap();

    println!("alpha: {:?}", alpha.to_string());
    //y
    let mut hasher5 = Keccak::v256();
    let _alpha_string = alpha.to_string();
    let alpha_string = &_alpha_string[8..8+64];
    let val26 = BigInt::parse_bytes(alpha_string.to_string().as_bytes(), 16).unwrap().to_bytes_be();
    let val27 = get_proog_bigint().w1.0.to_bytes_be();
    let val28 = get_proog_bigint().w1.1.to_bytes_be();

    concatenated = Vec::new();
    concatenated.extend_from_slice(&(val26.1));
    concatenated.extend_from_slice(&(val27.1));
    concatenated.extend_from_slice(&(val28.1));

    hasher5.update(&concatenated);
    out = [0u8; 32];
    hasher5.finalize(&mut out);
    let _y = BigInt::from_bytes_be(num_bigint::Sign::Plus, &out);
    let y = Fr::from_str(&_y.to_string()).unwrap();

    println!("y: {:?}", y.to_string());

    challenges.alpha = alpha;
    challenges.beta = beta;
    challenges.gamma = gamma;
    challenges.y = y;
    challenges.xiSeed = xiSeed;
    challenges.xiSeed2 = xiSeed2;
    challenges.xi = Xin;

} 


pub fn compute_lagrange(
    zh: Fp256<FrParameters>,
    eval_l1: Fp256<FrParameters>,
) -> Fp256<FrParameters> {
    let w = Fr::from_str("1").unwrap();
    eval_l1.mul(zh)
}

pub fn computePi(
    pubSignals: Fp256<FrParameters>,
    eval_l1: Fp256<FrParameters>,
) -> Fp256<FrParameters> {
    let pi = Fr::from_str("0").unwrap();

    let q = Fr::from_str(
        "21888242871839275222246405745257275088548364400416034343698204186575808495617",
    )
    .unwrap();

    q.add(pi.sub(eval_l1.mul(pubSignals)))
}

pub fn calculateInversions(
    y: Fp256<FrParameters>,
    xi: Fp256<FrParameters>,
    zhInv: Fp256<FrParameters>,
    h0w8: Vec<Fp256<FrParameters>>,
    h1w4: Vec<Fp256<FrParameters>>,
    h2w3: Vec<Fp256<FrParameters>>,
    h3w3: Vec<Fp256<FrParameters>>,
) -> (
    Fp256<FrParameters>,
    LISValues,
    Fp256<FrParameters>,
    Fp256<FrParameters>,
) {
    let mut w = y
        .sub(h1w4[0])
        .mul(y.sub(h1w4[1]).mul(y.sub(h1w4[2]).mul(y.sub(h1w4[3]))));
    // println!("w: {}", (w));

    let denH1 = w.clone();

    w = y.sub(h2w3[0]).mul(
        y.sub(h2w3[1])
            .mul(y.sub(h2w3[2]))
            .mul(y.sub(h3w3[0]).mul(y.sub(h3w3[1]).mul(y.sub(h3w3[2])))),
    );

    // println!("w: {}", (w));

    let denH2 = w.clone();

    let mut li_s0_inv = computeLiS0(y, h0w8);

    let mut li_s1_inv = computeLiS1(y, h1w4);

    let mut li_s2_inv = computeLiS2(y, xi, h2w3, h3w3);
    // println!()

    w = Fr::from_str("1").unwrap();

    let mut eval_l1 = get_domain_size().mul(xi.sub(w));

    // println!("eval_l1: {}", eval_l1);

    let invser_arr_resp = inverseArray(
        denH1,
        denH2,
        zhInv,
        li_s0_inv,
        li_s1_inv,
        li_s2_inv,
        &mut eval_l1,
    );

    (
        eval_l1,
        invser_arr_resp.0,
        invser_arr_resp.1,
        invser_arr_resp.2,
    )
}

pub fn computeLiS0(
    y: Fp256<FrParameters>,
    h0w8: Vec<Fp256<FrParameters>>,
) -> [Fp256<FrParameters>; 8] {
    let root0 = h0w8[0];

    let mut den1 = Fr::from_str("1").unwrap();
    den1 = den1
        .mul(root0)
        .mul(root0)
        .mul(root0)
        .mul(root0)
        .mul(root0)
        .mul(root0);

    // println!("den1: {}", den1);

    den1 = den1.mul(Fr::from_str("8").unwrap());

    let mut den2;
    let mut den3;

    let mut li_s0_inv: [Fp256<FrParameters>; 8] = [Fr::zero(); 8];

    let q = Fr::from_str(
        "21888242871839275222246405745257275088548364400416034343698204186575808495617",
    )
    .unwrap();

    for i in 0..8 {
        let coeff = ((i * 7) % 8);
        den2 = h0w8[0 + coeff];
        // println!("den2: {}", den2);
        den3 = y.add(q.sub(h0w8[0 + (i)]));
        // println!("den3: {}", den3);

        li_s0_inv[i] = den1.mul(den2).mul(den3);

        // println!("li_s0_inv: {}", li_s0_inv[i]);
        // println!();
    }
    // println!("li_s0_inv: {}", li_s0_inv[7]);

    li_s0_inv
}

pub fn computeLiS1(
    y: Fp256<FrParameters>,
    h1w4: Vec<Fp256<FrParameters>>,
) -> [Fp256<FrParameters>; 4] {
    let root0 = h1w4[0];
    let mut den1 = Fr::from_str("1").unwrap();
    den1 = den1.mul(root0).mul(root0);

    den1 = den1.mul(Fr::from_str("4").unwrap());

    let q = Fr::from_str(
        "21888242871839275222246405745257275088548364400416034343698204186575808495617",
    )
    .unwrap();

    let mut den2;
    let mut den3;

    let mut li_s1_inv: [Fp256<FrParameters>; 4] = [Fr::zero(); 4];

    for i in 0..4 {
        let coeff = ((i * 3) % 4);
        den2 = h1w4[0 + coeff];
        den3 = y.add(q.sub(h1w4[0 + (i)]));
        li_s1_inv[i] = den1.mul(den2).mul(den3);
    }

    // println!("li_s1_inv: {}", li_s1_inv[3]);
    li_s1_inv
}

pub fn computeLiS2(
    y: Fp256<FrParameters>,
    xi: Fp256<FrParameters>,
    h2w3: Vec<Fp256<FrParameters>>,
    h3w3: Vec<Fp256<FrParameters>>,
) -> [Fp256<FrParameters>; 6] {
    let q = Fr::from_str(
        "21888242871839275222246405745257275088548364400416034343698204186575808495617",
    )
    .unwrap();

    // let den1 := mulmod(mulmod(3,mload(add(pMem, pH2w3_0)),q), addmod(mload(add(pMem, pXi)) ,mod(sub(q, mulmod(mload(add(pMem, pXi)), w1 ,q)), q), q), q)
    let omegas = get_omegas();
    let mut den1 = (Fr::from_str("3").unwrap().mul(h2w3[0])).mul(xi.add(q.sub(xi.mul(omegas.w1))));

    let mut den2;
    let mut den3;

    let mut li_s2_inv: [Fp256<FrParameters>; 6] = [Fr::zero(); 6];

    for i in 0..3 {
        let coeff = ((i * 2) % 3);
        den2 = h2w3[0 + coeff];
        den3 = y.add(q.sub(h2w3[0 + (i)]));
        li_s2_inv[i] = den1.mul(den2).mul(den3);
    }

    den1 = (Fr::from_str("3").unwrap().mul(h3w3[0])).mul(xi.mul(omegas.w1).add(q.sub(xi)));

    for i in 0..3 {
        let coeff = ((i * 2) % 3);
        den2 = h3w3[0 + coeff];
        den3 = y.add(q.sub(h3w3[0 + (i)]));
        li_s2_inv[i + 3] = den1.mul(den2).mul(den3);
    }

    li_s2_inv
}

pub fn inverseArray(
    denH1: Fp256<FrParameters>,
    denH2: Fp256<FrParameters>,
    zhInv: Fp256<FrParameters>,
    li_s0_inv: [Fp256<FrParameters>; 8],
    li_s1_inv: [Fp256<FrParameters>; 4],
    li_s2_inv: [Fp256<FrParameters>; 6],
    eval_l1: &mut Fp256<FrParameters>,
) -> (LISValues, Fp256<FrParameters>, Fp256<FrParameters>) {
    // let mut local_eval_l1 = eval_l1.clone();
    let mut local_den_h1 = denH1.clone();
    let mut local_den_h2 = denH2.clone();
    let mut local_zh_inv = zhInv.clone();
    let mut local_li_s0_inv = li_s0_inv.clone();
    let mut local_li_s1_inv = li_s1_inv.clone();
    let mut local_li_s2_inv = li_s2_inv.clone();

    let mut _acc: Vec<Fp256<FrParameters>> = Vec::new();

    _acc.push(zhInv.clone());

    let mut acc = zhInv.mul(denH1);
    _acc.push(acc.clone());

    acc = acc.mul(denH2);
    _acc.push(acc.clone());

    for i in 0..8 {
        acc = acc.mul(local_li_s0_inv[i]);
        _acc.push(acc);
    }
    for i in 0..4 {
        acc = acc.mul(local_li_s1_inv[i]);
        _acc.push(acc);
    }
    for i in 0..6 {
        acc = acc.mul(local_li_s2_inv[i]);
        _acc.push(acc);
    }
    acc = acc.mul(eval_l1.clone());
    _acc.push(acc);
    // println!("acc: {}", acc);
    // println!("acc wala xeval_l1: {}", eval_l1);

    let mut inv = get_proof().eval_inv;

    // println!("inv: {}", inv);

    let check = inv.mul(acc);
    // println!("check: {}", check);
    assert!(check == Fr::one());

    acc = inv.clone();

    _acc.pop();
    inv = acc.mul(_acc.last().unwrap().clone());
    acc = acc.mul(eval_l1.clone());
    *eval_l1 = inv;
    // println!("herer eval_l1: {}", eval_l1);

    for i in (0..6).rev() {
        _acc.pop();
        inv = acc.mul(_acc.last().unwrap().clone());
        acc = acc.mul(local_li_s2_inv[i]);
        local_li_s2_inv[i] = inv;
    }
    // println!("local_li_s2_inv_0: {}", local_li_s2_inv[0]);

    for i in (0..4).rev() {
        _acc.pop();
        inv = acc.mul(_acc.last().unwrap().clone());
        acc = acc.mul(local_li_s1_inv[i]);
        local_li_s1_inv[i] = inv;
    }

    // println!("local_li_s1_inv_0: {}", local_li_s1_inv[0]);

    for i in (0..8).rev() {
        _acc.pop();
        inv = acc.mul(_acc.last().unwrap().clone());
        acc = acc.mul(local_li_s0_inv[i]);
        local_li_s0_inv[i] = inv;
    }

    // println!("local_li_s0_inv_0: {}", local_li_s0_inv[0]);

    _acc.pop();
    inv = acc.mul(_acc.last().unwrap().clone());
    acc = acc.mul(denH2);
    local_den_h2 = inv;

    _acc.pop();
    inv = acc.mul(_acc.last().unwrap().clone());
    acc = acc.mul(denH1);
    local_den_h1 = inv;

    local_zh_inv = acc;

    let lis_values = LISValues {
        li_s0_inv: local_li_s0_inv,
        li_s1_inv: local_li_s1_inv,
        li_s2_inv: local_li_s2_inv,
    };

    (lis_values, local_den_h1, local_den_h2)
    // println!("local_zh_inv: {}", local_zh_inv);
}


#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct ZkEvmProof{
    // pub c1: G1Point,
    // pub c2: G1Point,
    // pub w1: G1Point,
    // pub w2: G1Point,

    // pub eval_ql: Fp256<FrParameters>,
    // pub eval_qr: Fp256<FrParameters>,
    // pub eval_qm: Fp256<FrParameters>,
    // pub eval_qo: Fp256<FrParameters>,
    // pub eval_qc: Fp256<FrParameters>,
    // pub eval_s1: Fp256<FrParameters>,
    // pub eval_s2: Fp256<FrParameters>,
    // pub eval_s3: Fp256<FrParameters>,
    // pub eval_a: Fp256<FrParameters>,
    // pub eval_b: Fp256<FrParameters>,
    // pub eval_c: Fp256<FrParameters>,
    // pub eval_z: Fp256<FrParameters>,
    // pub eval_zw: Fp256<FrParameters>,
    // pub eval_t1w: Fp256<FrParameters>,
    // pub eval_t2w: Fp256<FrParameters>,
    // pub eval_inv: Fp256<FrParameters>,

    pub c1_x: [u8; 32],
    pub c1_y: [u8; 32],
    pub c2_x: [u8; 32],
    pub c2_y: [u8; 32],
    pub w1_x: [u8; 32],
    pub w1_y: [u8; 32],
    pub w2_x: [u8; 32],
    pub w2_y: [u8; 32],

    pub eval_ql: [u8; 32],
    pub eval_qr: [u8; 32],
    pub eval_qm: [u8; 32],
    pub eval_qo: [u8; 32],
    pub eval_qc: [u8; 32],
    pub eval_s1: [u8; 32],
    pub eval_s2: [u8; 32],
    pub eval_s3: [u8; 32],
    pub eval_a: [u8; 32],
    pub eval_b: [u8; 32],
    pub eval_c: [u8; 32],
    pub eval_z: [u8; 32],
    pub eval_zw: [u8; 32],
    pub eval_t1w: [u8; 32],
    pub eval_t2w: [u8; 32],
    pub eval_inv: [u8; 32],

}

pub fn u8_to_g1(a: [u8; 32], y: [u8; 32]) -> G1Point {
    let _x = <G1Point as AffineCurve>::BaseField::from_str(
        &a.iter().map(|y| y.to_string()).collect::<String>()).unwrap();
    let _y = <G1Point as AffineCurve>::BaseField::from_str(
        &y.iter().map(|y| y.to_string()).collect::<String>()
    ).unwrap();
    G1Projective::new(
        _x,
        _y,
        <G1Projective as ProjectiveCurve>::BaseField::one(),
    )
    .into_affine()
}

pub fn u8_to_fp256(x: [u8; 32]) -> Fp256<FrParameters> {
    let _x = Fr::from_str(
        &x.iter().map(|x| x.to_string()).collect::<String>()
    ).unwrap();
    _x
}

pub fn get_bignint_from_h256(h256: H256) -> BigInt {
    // Create a BigInt from the byte array
    BigInt::from_bytes_le(num_bigint::Sign::Plus, h256.as_fixed_slice())
}

impl Proof<ZkEvmRollupPublicInputs> for ZkEvmProof {
    fn verify(
        &self,
        vk: &[[u8; 32]; 6],
        public_inputs: &ZkEvmRollupPublicInputs,
    ) -> Result<(), anyhow::Error> {
        println!("Here in ZkEvmProof::verify");
        eprintln!("ZkEvmProof::verify");
        eprintln!("vk: {:?}", vk);
        eprintln!("public_inputs: {:?}", public_inputs);
        println!("c1: {:?}", self.c1_x);

        let proof = Prooff{
            c1: u8_to_g1(self.c1_x, self.c1_y),
            c2: u8_to_g1(self.c2_x, self.c2_y),
            w1: u8_to_g1(self.w1_x, self.w1_y),
            w2: u8_to_g1(self.w2_x, self.w2_y),
            eval_ql: u8_to_fp256(self.eval_ql),
            eval_qr: u8_to_fp256(self.eval_qr),
            eval_qm: u8_to_fp256(self.eval_qm),
            eval_qo: u8_to_fp256(self.eval_qo),
            eval_qc: u8_to_fp256(self.eval_qc),
            eval_s1: u8_to_fp256(self.eval_s1),
            eval_s2: u8_to_fp256(self.eval_s2),
            eval_s3: u8_to_fp256(self.eval_s3),
            eval_a: u8_to_fp256(self.eval_a),
            eval_b: u8_to_fp256(self.eval_b),
            eval_c: u8_to_fp256(self.eval_c),
            eval_z: u8_to_fp256(self.eval_z),
            eval_zw: u8_to_fp256(self.eval_zw),
            eval_t1w: u8_to_fp256(self.eval_t1w),
            eval_t2w: u8_to_fp256(self.eval_t2w),
            eval_inv: u8_to_fp256(self.eval_inv),    
        };

        let pub_signal = get_pubSignals();
        let mut challenges = Challenges {
            alpha: Fr::zero(),
            beta: Fr::zero(),
            gamma: Fr::zero(),
            y: Fr::zero(),
            xiSeed: Fr::zero(),
            xiSeed2: Fr::zero(),
            xi: Fr::zero(),
    
        };
        let mut roots = Roots {
            h0w8: [Fr::zero(); 8],
            h1w4: [Fr::zero(); 4],
            h2w3: [Fr::zero(); 3],
            h3w3: [Fr::zero(); 3],
        };
        let mut vpi = VerifierProcessedInputs {
            c0x: BigInt::from_bytes_le(num_bigint::Sign::Plus, &vk[0]),
            c0y: BigInt::from_bytes_le(num_bigint::Sign::Plus, &vk[1]),
            x2x1: BigInt::from_bytes_le(num_bigint::Sign::Plus, &vk[2]),
            x2x2: BigInt::from_bytes_le(num_bigint::Sign::Plus, &vk[3]),
            x2y1: BigInt::from_bytes_le(num_bigint::Sign::Plus, &vk[4]),
            x2y2: BigInt::from_bytes_le(num_bigint::Sign::Plus, &vk[5]),
        };
        println!("vk loaded");
        println!("c0x: {:?}", vpi.c0x);
    
        let pubSignalBigInt = get_bignint_from_h256(public_inputs.pub_signal);
        // BigInt::parse_bytes(b"14516932981781041565586298118536599721399535462624815668597272732223874827152", 10).unwrap();
        
        let mut zh: &mut Fp256<FrParameters> = &mut Fr::zero();
    
        let mut zhinv: &mut Fp256<FrParameters> = &mut Fr::zero();
    
        compute_challenges(&mut challenges, &mut roots, &mut zh, &mut zhinv, vpi, pubSignalBigInt);


        let alpha: Fp256<FrParameters> = challenges.alpha;

        let beta: Fp256<FrParameters> = challenges.beta;
    
        let gamma: Fp256<FrParameters> = challenges.gamma;
    
        let xiseed: Fp256<FrParameters> = challenges.xiSeed;
    
        let xiseed2: Fp256<FrParameters> = challenges.xiSeed2;
    
        let mut y: Fp256<FrParameters> = challenges.y;
    
        let mut xi: Fp256<FrParameters> = challenges.xi;
    
        // let zh: Fp256<FrParameters> = 
    
        // let mut zhinv: Fp256<FrParameters> = Fr::from_str(
        //     "8663234610000964594764035144827003258323335914482598945994186647593190381653",
        // )
        // .unwrap();
    
        // it is similar to zhinv just more updated value
        let zinv = zhinv.clone();
    
        let g1_x = <G1Point as AffineCurve>::BaseField::from_str("1").unwrap();
    
        let g1_y = <G1Point as AffineCurve>::BaseField::from_str("2").unwrap();
    
        let g1_affine = G1Projective::new(
            g1_x,
            g1_y,
            <G1Projective as ProjectiveCurve>::BaseField::one(),
        )
        .into_affine();
    
        let h0w8: Vec<Fp256<FrParameters>> = roots.h0w8.to_vec();
    
        let h1w4: Vec<Fp256<FrParameters>> = roots.h1w4.to_vec();
    
        let h2w3: Vec<Fp256<FrParameters>> = roots.h2w3.to_vec();
    
        let h3w3: Vec<Fp256<FrParameters>> = roots.h3w3.to_vec();
    
        let mut inv_tuple = calculateInversions(
            y,
            xi,
            *zhinv,
            h0w8.clone(),
            h1w4.clone(),
            h2w3.clone(),
            h3w3.clone(),
        );
        let mut eval_l1 = inv_tuple.0;
        let lis_values = inv_tuple.1;
        let denH1 = inv_tuple.2;
        let denH2 = inv_tuple.3;
    
        eval_l1 = compute_lagrange(*zh, eval_l1);
    
        let pi = computePi(pub_signal, eval_l1);

        println!("Verifying proof...");

        let R0 = calculateR0(xi, proof.clone(), y, h0w8.clone(), lis_values.li_s0_inv);
        let R1 = calculateR1(
            xi,
            proof.clone(),
            y,
            pi,
            h1w4.clone(),
            lis_values.li_s1_inv,
            zinv,
        );
        let R2 = calculateR2(
            xi,
            gamma,
            beta,
            proof.clone(),
            y,
            eval_l1,
            zinv,
            h2w3.clone(),
            h3w3.clone(),
            lis_values.li_s2_inv,
        );
        let points = computeFEJ(
            y,
            h0w8.clone(),
            denH1,
            denH2,
            alpha,
            proof.clone(),
            g1_affine,
            R0,
            R1,
            R2,
        );
    
        let F = points.0;
        let E = points.1;
        let J = points.2;
    
        let W2 = proof.w2;
    
        // first pairing value
        let p1 = F.add(-E).add(-J).add(W2.mul(y).into_affine());
    
        let g2x1 = Fq::from_str(
            "10857046999023057135944570762232829481370756359578518086990519993285655852781",
        )
        .unwrap();
        let g2x2 = Fq::from_str(
            "11559732032986387107991004021392285783925812861821192530917403151452391805634",
        )
        .unwrap();
        let g2y1 =
            Fq::from_str("869093939501355406318588453775243436758538662501260653214950591532352435323")
                .unwrap();
        let g2y2 = Fq::from_str(
            "4082367875863433681332203403145435568316851327593401208105741076214120093531",
        )
        .unwrap();
    
        // second pairing value
        let g2_val = G2Affine::new(Fq2::new(g2x1, g2x2), Fq2::new(g2y1, g2y2), true);
    
        // third pairing value
        let p3 = -W2;
    
        // fourth pairing value
        let x2x1 = Fq::from_str(
            "21831381940315734285607113342023901060522397560371972897001948545212302161822",
        )
        .unwrap();
        let x2x2 = Fq::from_str(
            "17231025384763736816414546592865244497437017442647097510447326538965263639101",
        )
        .unwrap();
        let x2y1 = Fq::from_str(
            "2388026358213174446665280700919698872609886601280537296205114254867301080648",
        )
        .unwrap();
        let x2y2 = Fq::from_str(
            "11507326595632554467052522095592665270651932854513688777769618397986436103170",
        )
        .unwrap();
    
        println!("Doing Pairing Check!");
        let x2_val = G2Affine::new(Fq2::new(x2x1, x2x2), Fq2::new(x2y1, x2y2), true);

            // let R2 = calculateR2(xi);
        let pairing1 = Bn254::pairing(p1, g2_val);
        let pairing2 = Bn254::pairing(p3, x2_val);

        if pairing1 == pairing2 {
            println!("Proof Verified!");
            // return true;
        } 

        println!("Proof verification failed!");
        
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, Copy)]
pub struct ZkEvmRollupPublicInputs {
    pub prev_state_root: H256,
    pub post_state_root: H256,
    pub blob_hash: H256,
    pub pub_signal: H256,
}

impl RollupPublicInputs for ZkEvmRollupPublicInputs {
    fn prev_state_root(&self) -> H256 {
        self.prev_state_root.clone()
    }
    fn post_state_root(&self) -> H256 {
        self.post_state_root.clone()
    }
    fn blob_hash(&self) -> H256 {
        self.blob_hash.clone()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ZkEvmVerificationKey{
        pub c0x: BigInt,
        pub c0y: BigInt,
        pub x2x1: BigInt,
        pub x2x2: BigInt,
        pub x2y1: BigInt,
        pub x2y2: BigInt,
    
}


fn calculateR0(
    xi: Fp256<FrParameters>,
    proof: Prooff,
    y: Fp256<FrParameters>,
    h0w8: Vec<Fp256<FrParameters>>,
    li_s0_inv: [Fp256<FrParameters>; 8],
) -> Fp256<FrParameters> {
    let Prooff {
        eval_ql,
        eval_qr,
        eval_qm,
        eval_qo,
        eval_qc,
        eval_s1,
        eval_s2,
        eval_s3,
        ..
    } = proof;

    let mut num = Fr::from_str("1").unwrap();
    let y__8 = y.pow([8]);
    num = num.mul(y__8);
    num = num.add(-xi);

    let mut h0w80 = h0w8[0];
    let pH0w8_1_term = h0w8[1];
    let pH0w8_2_term = h0w8[2];
    let pH0w8_3_term = h0w8[3];
    let pH0w8_4_term = h0w8[4];
    let pH0w8_5_term = h0w8[5];
    let pH0w8_6_term = h0w8[6];
    let pH0w8_7_term = h0w8[7];

    let pLiS0Inv_term = li_s0_inv[0];
    let pLiS0Inv_32_term = li_s0_inv[1];
    let pLiS0Inv_64_term = li_s0_inv[2];
    let pLiS0Inv_96_term = li_s0_inv[3];
    let pLiS0Inv_128_term = li_s0_inv[4];
    let pLiS0Inv_160_term = li_s0_inv[5];
    let pLiS0Inv_192_term = li_s0_inv[6];
    let pLiS0Inv_224_term = li_s0_inv[7];

    let mut c0Value = eval_ql.add(h0w80.mul(eval_qr));

    let mut h0w8i = h0w80.mul(h0w80);
    c0Value = c0Value.add(eval_qo.mul(h0w8i));

    h0w8i = h0w8i.mul(h0w80);
    c0Value = c0Value.add(eval_qm.mul(h0w8i));

    h0w8i = h0w8i.mul(h0w80);
    c0Value = c0Value.add(eval_qc.mul(h0w8i));

    h0w8i = h0w8i.mul(h0w80);
    c0Value = c0Value.add(eval_s1.mul(h0w8i));

    h0w8i = h0w8i.mul(h0w80);
    c0Value = c0Value.add(eval_s2.mul(h0w8i));

    h0w8i = h0w8i.mul(h0w80);
    c0Value = c0Value.add(eval_s3.mul(h0w8i));

    let res = c0Value.mul(num.mul(pLiS0Inv_term));

    h0w80 = pH0w8_1_term;
    c0Value = eval_ql.add(h0w80.mul(eval_qr));

    h0w8i = h0w80.mul(h0w80);
    c0Value = c0Value.add(eval_qo.mul(h0w8i));

    h0w8i = h0w8i.mul(h0w80);
    c0Value = c0Value.add(eval_qm.mul(h0w8i));

    h0w8i = h0w8i.mul(h0w80);
    c0Value = c0Value.add(eval_qc.mul(h0w8i));

    h0w8i = h0w8i.mul(h0w80);
    c0Value = c0Value.add(eval_s1.mul(h0w8i));

    h0w8i = h0w8i.mul(h0w80);
    c0Value = c0Value.add(eval_s2.mul(h0w8i));

    h0w8i = h0w8i.mul(h0w80);
    c0Value = c0Value.add(eval_s3.mul(h0w8i));

    let res_2 = res.add(c0Value.mul(num.mul(pLiS0Inv_32_term)));

    h0w80 = pH0w8_2_term;
    c0Value = eval_ql.add(h0w80.mul(eval_qr));

    h0w8i = h0w80.mul(h0w80);
    c0Value = c0Value.add(eval_qo.mul(h0w8i));

    h0w8i = h0w8i.mul(h0w80);
    c0Value = c0Value.add(eval_qm.mul(h0w8i));

    h0w8i = h0w8i.mul(h0w80);
    c0Value = c0Value.add(eval_qc.mul(h0w8i));

    h0w8i = h0w8i.mul(h0w80);
    c0Value = c0Value.add(eval_s1.mul(h0w8i));

    h0w8i = h0w8i.mul(h0w80);
    c0Value = c0Value.add(eval_s2.mul(h0w8i));

    h0w8i = h0w8i.mul(h0w80);
    c0Value = c0Value.add(eval_s3.mul(h0w8i));

    let res_3 = res_2.add(c0Value.mul(num.mul(pLiS0Inv_64_term)));

    h0w80 = pH0w8_3_term;
    c0Value = eval_ql.add(h0w80.mul(eval_qr));

    h0w8i = h0w80.mul(h0w80);
    c0Value = c0Value.add(eval_qo.mul(h0w8i));

    h0w8i = h0w8i.mul(h0w80);
    c0Value = c0Value.add(eval_qm.mul(h0w8i));

    h0w8i = h0w8i.mul(h0w80);
    c0Value = c0Value.add(eval_qc.mul(h0w8i));

    h0w8i = h0w8i.mul(h0w80);
    c0Value = c0Value.add(eval_s1.mul(h0w8i));

    h0w8i = h0w8i.mul(h0w80);
    c0Value = c0Value.add(eval_s2.mul(h0w8i));

    h0w8i = h0w8i.mul(h0w80);
    c0Value = c0Value.add(eval_s3.mul(h0w8i));

    let res_4 = res_3.add(c0Value.mul(num.mul(pLiS0Inv_96_term)));

    h0w80 = pH0w8_4_term;
    c0Value = eval_ql.add(h0w80.mul(eval_qr));

    h0w8i = h0w80.mul(h0w80);
    c0Value = c0Value.add(eval_qo.mul(h0w8i));

    h0w8i = h0w8i.mul(h0w80);
    c0Value = c0Value.add(eval_qm.mul(h0w8i));

    h0w8i = h0w8i.mul(h0w80);
    c0Value = c0Value.add(eval_qc.mul(h0w8i));

    h0w8i = h0w8i.mul(h0w80);
    c0Value = c0Value.add(eval_s1.mul(h0w8i));

    h0w8i = h0w8i.mul(h0w80);
    c0Value = c0Value.add(eval_s2.mul(h0w8i));

    h0w8i = h0w8i.mul(h0w80);
    c0Value = c0Value.add(eval_s3.mul(h0w8i));

    let res_5 = res_4.add(c0Value.mul(num.mul(pLiS0Inv_128_term)));

    h0w80 = pH0w8_5_term;
    c0Value = eval_ql.add(h0w80.mul(eval_qr));

    h0w8i = h0w80.mul(h0w80);
    c0Value = c0Value.add(eval_qo.mul(h0w8i));

    h0w8i = h0w8i.mul(h0w80);
    c0Value = c0Value.add(eval_qm.mul(h0w8i));

    h0w8i = h0w8i.mul(h0w80);
    c0Value = c0Value.add(eval_qc.mul(h0w8i));

    h0w8i = h0w8i.mul(h0w80);
    c0Value = c0Value.add(eval_s1.mul(h0w8i));

    h0w8i = h0w8i.mul(h0w80);
    c0Value = c0Value.add(eval_s2.mul(h0w8i));

    h0w8i = h0w8i.mul(h0w80);
    c0Value = c0Value.add(eval_s3.mul(h0w8i));

    let res_6 = res_5.add(c0Value.mul(num.mul(pLiS0Inv_160_term)));

    h0w80 = pH0w8_6_term;
    c0Value = eval_ql.add(h0w80.mul(eval_qr));

    h0w8i = h0w80.mul(h0w80);
    c0Value = c0Value.add(eval_qo.mul(h0w8i));

    h0w8i = h0w8i.mul(h0w80);
    c0Value = c0Value.add(eval_qm.mul(h0w8i));

    h0w8i = h0w8i.mul(h0w80);
    c0Value = c0Value.add(eval_qc.mul(h0w8i));

    h0w8i = h0w8i.mul(h0w80);
    c0Value = c0Value.add(eval_s1.mul(h0w8i));

    h0w8i = h0w8i.mul(h0w80);
    c0Value = c0Value.add(eval_s2.mul(h0w8i));

    h0w8i = h0w8i.mul(h0w80);
    c0Value = c0Value.add(eval_s3.mul(h0w8i));

    let res_7 = res_6.add(c0Value.mul(num.mul(pLiS0Inv_192_term)));

    h0w80 = pH0w8_7_term;
    c0Value = eval_ql.add(h0w80.mul(eval_qr));

    h0w8i = h0w80.mul(h0w80);
    c0Value = c0Value.add(eval_qo.mul(h0w8i));

    h0w8i = h0w8i.mul(h0w80);
    c0Value = c0Value.add(eval_qm.mul(h0w8i));

    h0w8i = h0w8i.mul(h0w80);
    c0Value = c0Value.add(eval_qc.mul(h0w8i));

    h0w8i = h0w8i.mul(h0w80);
    c0Value = c0Value.add(eval_s1.mul(h0w8i));

    h0w8i = h0w8i.mul(h0w80);
    c0Value = c0Value.add(eval_s2.mul(h0w8i));

    h0w8i = h0w8i.mul(h0w80);
    c0Value = c0Value.add(eval_s3.mul(h0w8i));

    let res_8 = res_7.add(c0Value.mul(num.mul(pLiS0Inv_224_term)));

    res_8
}

fn calculateR1(
    xi: Fp256<FrParameters>,
    proof: Prooff,
    y: Fp256<FrParameters>,
    pi: Fp256<FrParameters>,
    h1w4: Vec<Fp256<FrParameters>>,
    li_s1_inv: [Fp256<FrParameters>; 4],
    zinv: Fp256<FrParameters>,
) -> Fp256<FrParameters> {
    let mut num = Fr::from_str("1").unwrap();
    let Prooff {
        eval_a,
        eval_b,
        eval_c,
        eval_ql,
        eval_qc,
        eval_qr,
        eval_qo,
        eval_qm,
        ..
    } = proof;

    let H1w4_0 = h1w4[0];
    let H1w4_1 = h1w4[1];
    let H1w4_2 = h1w4[2];
    let H1w4_3 = h1w4[3];

    let pLiS1Inv_0_term = li_s1_inv[0];
    let pLiS1Inv_32_term = li_s1_inv[1];
    let pLiS1Inv_64_term = li_s1_inv[2];
    let pLiS1Inv_96_term = li_s1_inv[3];

    let y__4 = y.pow([4]);
    num = num.mul(y__4);
    num = num.add(-xi);

    let mut t0 = eval_ql.mul(eval_a);
    t0 = t0.add(eval_qr.mul(eval_b));
    t0 = t0.add(eval_qm.mul(eval_a.mul(eval_b)));
    t0 = t0.add(eval_qo.mul(eval_c));
    t0 = t0.add(eval_qc);
    t0 = t0.add(pi);
    t0 = t0.mul(zinv);

    let mut c1Value = eval_a;
    c1Value = c1Value.add(H1w4_0.mul(eval_b));
    let mut square = H1w4_0.mul(H1w4_0);
    c1Value = c1Value.add(eval_c.mul(square));
    c1Value = c1Value.add(t0.mul(square.mul(H1w4_0)));

    let res_1 = c1Value.mul(num.mul(pLiS1Inv_0_term));

    c1Value = eval_a;
    c1Value = c1Value.add(H1w4_1.mul(eval_b));
    let mut square = H1w4_1.mul(H1w4_1);
    c1Value = c1Value.add(eval_c.mul(square));
    c1Value = c1Value.add(t0.mul(square.mul(H1w4_1)));

    let res_2 = res_1.add(c1Value.mul(num.mul(pLiS1Inv_32_term)));
    // pLiS1Inv_32_term

    c1Value = eval_a;
    c1Value = c1Value.add(H1w4_2.mul(eval_b));
    let mut square = H1w4_2.mul(H1w4_2);
    c1Value = c1Value.add(eval_c.mul(square));
    c1Value = c1Value.add(t0.mul(square.mul(H1w4_2)));

    let res_3 = res_2.add(c1Value.mul(num.mul(pLiS1Inv_64_term)));

    c1Value = eval_a;
    c1Value = c1Value.add(H1w4_3.mul(eval_b));
    let mut square = H1w4_3.mul(H1w4_3);
    c1Value = c1Value.add(eval_c.mul(square));
    c1Value = c1Value.add(t0.mul(square.mul(H1w4_3)));

    let res_4 = res_3.add(c1Value.mul(num.mul(pLiS1Inv_96_term)));

    res_4
}

fn calculateR2(
    xi: Fp256<FrParameters>,
    gamma: Fp256<FrParameters>,
    beta: Fp256<FrParameters>,
    proof: Prooff,
    y: Fp256<FrParameters>,
    eval_l1: Fp256<FrParameters>,
    zinv: Fp256<FrParameters>,
    h2w3: Vec<Fp256<FrParameters>>,
    h3w3: Vec<Fp256<FrParameters>>,
    li_s2_inv: [Fp256<FrParameters>; 6],
) -> Fp256<FrParameters> {
    let Prooff {
        eval_a,
        eval_b,
        eval_c,
        eval_z,
        eval_s1,
        eval_s2,
        eval_s3,
        eval_zw,
        eval_t1w,
        eval_t2w,
        ..
    } = proof;

    let w1 = get_omegas().w1;
    let mut num = Fr::from_str("1").unwrap();

    let betaxi = beta.mul(xi);

    let y__6 = y.pow([6]);
    let k1 = Fr::from_str("2").unwrap();
    let k2 = Fr::from_str("3").unwrap();

    let h2w3_0 = h2w3[0];
    let h2w3_1 = h2w3[1];
    let h2w3_2 = h2w3[2];
    let h3w3_0 = h3w3[0];
    let h3w3_1 = h3w3[1];
    let h3w3_2 = h3w3[2];

    let pLiS2Inv_0_term = li_s2_inv[0];
    let pLiS2Inv_32_term = li_s2_inv[1];
    let pLiS2Inv_64_term = li_s2_inv[2];
    let pLiS2Inv_96_term = li_s2_inv[3];
    let pLiS2Inv_128_term = li_s2_inv[4];
    let pLiS2Inv_160_term = li_s2_inv[5];

    num = num.mul(y__6);

    let mut num2 = Fr::one();
    num2 = num2.mul(y.pow([3]));

    num2 = num2.mul(xi.add(xi.mul(w1)));

    num = num.sub(num2);

    num2 = xi.mul(xi.mul(w1));
    num = num.add(num2);

    let mut t2 = eval_a.add(betaxi.add(gamma));
    t2 = t2.mul(eval_b.add(gamma.add(betaxi.mul(k1))));
    t2 = t2.mul(eval_c.add(gamma.add(betaxi.mul(k2))));
    t2 = t2.mul(eval_z);

    let mut t1 = eval_a.add(gamma.add(beta.mul(eval_s1)));
    t1 = t1.mul(eval_b.add(gamma.add(beta.mul(eval_s2))));
    t1 = t1.mul(eval_c.add(gamma.add(beta.mul(eval_s3))));
    t1 = t1.mul(eval_zw);

    t2 = t2.sub(t1);
    t2 = t2.mul(zinv);

    t1 = eval_z.sub(Fr::one());
    t1 = t1.mul(eval_l1);
    t1 = t1.mul(zinv);

    let mut gamma_r2 = Fr::zero();
    let mut hw = h2w3_0;
    let mut c2Value = eval_z.add(hw.mul(t1));
    c2Value = c2Value.add(t2.mul(hw.mul(hw)));
    gamma_r2 = gamma_r2.add(c2Value.mul(num.mul(pLiS2Inv_0_term)));

    hw = h2w3_1;
    c2Value = eval_z.add(hw.mul(t1));
    c2Value = c2Value.add(t2.mul(hw.mul(hw)));
    gamma_r2 = gamma_r2.add(c2Value.mul(num.mul(pLiS2Inv_32_term)));

    hw = h2w3_2;
    c2Value = eval_z.add(hw.mul(t1));
    c2Value = c2Value.add(t2.mul(hw.mul(hw)));
    gamma_r2 = gamma_r2.add(c2Value.mul(num.mul(pLiS2Inv_64_term)));

    hw = h3w3_0;
    c2Value = eval_zw.add(hw.mul(eval_t1w));
    c2Value = c2Value.add(eval_t2w.mul(hw.mul(hw)));
    gamma_r2 = gamma_r2.add(c2Value.mul(num.mul(pLiS2Inv_96_term)));

    hw = h3w3_1;
    c2Value = eval_zw.add(hw.mul(eval_t1w));
    c2Value = c2Value.add(eval_t2w.mul(hw.mul(hw)));
    gamma_r2 = gamma_r2.add(c2Value.mul(num.mul(pLiS2Inv_128_term)));

    hw = h3w3_2;
    c2Value = eval_zw.add(hw.mul(eval_t1w));
    c2Value = c2Value.add(eval_t2w.mul(hw.mul(hw)));
    gamma_r2 = gamma_r2.add(c2Value.mul(num.mul(pLiS2Inv_160_term)));

    gamma_r2
}

fn computeFEJ(
    y: Fp256<FrParameters>,
    h0w8: Vec<Fp256<FrParameters>>,
    denH1: Fp256<FrParameters>,
    denH2: Fp256<FrParameters>,
    alpha: Fp256<FrParameters>,
    proof: Prooff,
    g1: GroupAffine<Parameters>,
    R0: Fp256<FrParameters>,
    R1: Fp256<FrParameters>,
    R2: Fp256<FrParameters>,
) -> (
    GroupAffine<Parameters>,
    GroupAffine<Parameters>,
    GroupAffine<Parameters>,
) {
    let mut numerator = y.sub(h0w8[0]);
    numerator = numerator.mul(y.sub(h0w8[1]));
    numerator = numerator.mul(y.sub(h0w8[2]));
    numerator = numerator.mul(y.sub(h0w8[3]));
    numerator = numerator.mul(y.sub(h0w8[4]));
    numerator = numerator.mul(y.sub(h0w8[5]));
    numerator = numerator.mul(y.sub(h0w8[6]));
    numerator = numerator.mul(y.sub(h0w8[7]));

    let c1 = proof.c1;
    let c2 = proof.c2;
    let w1 = proof.w1;

    let mut quotient1 = alpha.mul(numerator.mul(denH1));
    let mut quotient2 = alpha.mul(alpha.mul(numerator.mul(denH2)));

    let c0_x = <G1Point as AffineCurve>::BaseField::from_str(
        "7005013949998269612234996630658580519456097203281734268590713858661772481668",
    )
    .unwrap();

    let c0_y = <G1Point as AffineCurve>::BaseField::from_str(
        "869093939501355406318588453775243436758538662501260653214950591532352435323",
    )
    .unwrap();

    let c0_affine = G1Projective::new(
        c0_x,
        c0_y,
        <G1Projective as ProjectiveCurve>::BaseField::one(),
    )
    .into_affine();
    // pf -> c0x
    // pf + 32 -> c0y
    // pf, pc1, quotient1

    // min -> c1x
    // min + 32 -> c1y
    // min + 64 -> quotient1

    // multiply c1 * quotient1

    // min + 64 -> c0x
    // min + 96 -> c0y

    // adding points c1 * quotient1 + c0

    // print!("Quotient 1: {:?}", quotient1.to_string());
    // print!("Quotient 2: {:?}", quotient2.to_string());

    let c1_agg = c0_affine.add(c1.mul(quotient1).into_affine());

    let c2_agg = c1_agg.add(c2.mul(quotient2).into_affine()); //  F point
                                                              // println!("c2_agg: {:?}", c2_agg.x.to_string());
                                                              // println!("c2_agg: {:?}", c2_agg.y.to_string());

    let r_agg = R0.add(quotient1.mul(R1).add(quotient2.mul(R2)));

    let g1_acc = g1.mul(r_agg).into_affine(); // E point
                                              // println!("g1_acc: {:?}", g1_acc.x.to_string());
                                              // println!("g1_acc: {:?}", g1_acc.y.to_string());

    let w1_agg = w1.mul(numerator).into_affine(); // J Point

    // println!("w1_agg: {:?}", w1_agg.x.to_string());
    // println!("w1_agg: {:?}", w1_agg.y.to_string());
    // pE, g1x, g1y, r_agg
    // min -> g1x
    // min + 32 -> g1y
    // min + 64 -> r_agg

    // multiply g1 * r_agg

    // min + 64 -> 0
    // min + 96 -> 0
    (c2_agg, g1_acc, w1_agg)
}

