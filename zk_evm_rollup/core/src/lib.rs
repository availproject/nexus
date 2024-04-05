use adapter_sdk::traits::{Proof, RollupPublicInputs};
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

    pub c1_x: [u64; 32],
    pub c1_y: [u64; 32],
    pub c2_x: [u64; 32],
    pub c2_y: [u64; 32],
    pub w1_x: [u64; 32],
    pub w1_y: [u64; 32],
    pub w2_x: [u64; 32],
    pub w2_y: [u64; 32],

    pub eval_ql: [u64; 32],
    pub eval_qr: [u64; 32],
    pub eval_qm: [u64; 32],
    pub eval_qo: [u64; 32],
    pub eval_qc: [u64; 32],
    pub eval_s1: [u64; 32],
    pub eval_s2: [u64; 32],
    pub eval_s3: [u64; 32],
    pub eval_a: [u64; 32],
    pub eval_b: [u64; 32],
    pub eval_c: [u64; 32],
    pub eval_z: [u64; 32],
    pub eval_zw: [u64; 32],
    pub eval_t1w: [u64; 32],
    pub eval_t2w: [u64; 32],
    pub eval_inv: [u64; 32],

}

pub fn u64_to_g1(x: [u64; 32], y: [u64; 32]) -> G1Point {
    let _x = <G1Point as AffineCurve>::BaseField::from_str(
        &x.iter().map(|x| x.to_string()).collect::<String>()
    ).unwrap();
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

pub fn u64_to_fp256(x: [u64; 32]) -> Fp256<FrParameters> {
    let _x = Fr::from_str(
        &x.iter().map(|x| x.to_string()).collect::<String>()
    ).unwrap();
    _x
}


impl Proof<ZkEvmRollupPublicInputs> for ZkEvmProof {
    fn verify(
        &self,
        vk: &[u8; 32],
        public_inputs: &ZkEvmRollupPublicInputs,
    ) -> Result<(), anyhow::Error> {
        println!("Here in ZkEvmProof::verify");
        eprintln!("ZkEvmProof::verify");
        eprintln!("vk: {:?}", vk);
        eprintln!("public_inputs: {:?}", public_inputs);
        println!("c1: {:?}", self.c1_x);





        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, Copy)]
pub struct ZkEvmRollupPublicInputs {
    pub prev_state_root: H256,
    pub post_state_root: H256,
    pub blob_hash: H256,
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
