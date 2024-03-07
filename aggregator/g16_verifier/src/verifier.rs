use std::str::FromStr;

use ark_ec::{AffineCurve, PairingEngine, ProjectiveCurve};
use ark_groth16::{prepare_inputs, prepare_verifying_key, verify_proof, Groth16, PreparedVerifyingKey, Proof, VerifyingKey};
use anyhow::{anyhow, Error, Result};
use ark_bn254::{g1::G1_GENERATOR_X, g2, Fq, Fq2, Fr, G1Affine, G2Affine};
// use ark_ff::{ MontFp, QuadExtConfig};

// use ark_serialize::CanonicalDeserialize;
use num_bigint::{BigInt, Sign};
use ark_bn254::{Bn254, G1Projective};

use ark_ff::{
    field_new, Field, Fp256, Fp256Parameters, Fp2ParamsWrapper, One, PrimeField, QuadExtField,
    UniformRand, Zero,
};

use ark_bn254::{
    g1, g1::Parameters, FqParameters, FrParameters, G2Projective,
};

pub type G1Point = <Bn254 as PairingEngine>::G1Affine;
pub type G2Point = <Bn254 as PairingEngine>::G2Affine;

const ALPHA_X: &str =
    "20491192805390485299153009773594534940189261866228447918068658471970481763042";
    const ALPHA_Y: &str =
        "9383485363053290200918347156157836566562967994039712273449902621266178545958";
    const BETA_X1: &str =
        "4252822878758300859123897981450591353533073413197771768651442665752259397132";
    const BETA_X2: &str =
        "6375614351688725206403948262868962793625744043794305715222011528459656738731";
    const BETA_Y1: &str =
        "21847035105528745403288232691147584728191162732299865338377159692350059136679";
    const BETA_Y2: &str =
        "10505242626370262277552901082094356697409835680220590971873171140371331206856";
    const GAMMA_X1: &str =
        "11559732032986387107991004021392285783925812861821192530917403151452391805634";
    const GAMMA_X2: &str =
        "10857046999023057135944570762232829481370756359578518086990519993285655852781";
    const GAMMA_Y1: &str =
        "4082367875863433681332203403145435568316851327593401208105741076214120093531";
    const GAMMA_Y2: &str =
        "8495653923123431417604973247489272438418190587263600148770280649306958101930";
    const DELTA_X1: &str =
        "20637939757332191985219466750514112514830176492003070298908178796582256423445";
    const DELTA_X2: &str =
        "21015870987554935578856562994563796394452175083269944606559673949460277152483";
    const DELTA_Y1: &str =
        "7308971620370004609743038871778988943972318366181842608509263947408591078846";
    const DELTA_Y2: &str =
        "19578762133483017273429849028797807252406479590275449312036317638112265649126";

    const IC0_X: &str = "4595639739788529313135927846153489513260052783364743523344328896305419933627";
    const IC0_Y: &str = "13577843718844184042346095806470311065274840502864234728407198439361979518223";
    const IC1_X: &str = "19125733112813331880180112762042920784001527126678496097978721184513458499861";
    const IC1_Y: &str = "470495054354753477176064253439657941845200056447070007550476843795069859530";
    const IC2_X: &str = "9798632009143333403145042225641105799474060066926099950339875153142594918323";
    const IC2_Y: &str = "15467851970301286525906423722646678659414362276892586739627188622113917076355";
    const IC3_X: &str = "4677856832410602822119633312864839150180396112709578634305606190993420950086";
    const IC3_Y: &str = "21413789555508871663216491538642005537595601774930793267108872091881334409985";
    const IC4_X: &str = "17622463197037705164686879153818888337611670039316323149958751021262085916949";
    const IC4_Y: &str = "10546326028888365743245970980969672597991412490319907398941581639510925080455";


pub fn verify(){
    let temp_a : &[Vec<u8>] = &[vec![29, 81, 12, 222, 49, 79, 63, 66, 226, 208, 219, 255, 73, 50, 241, 196, 116, 140, 85, 176, 155, 85, 9, 6, 32, 28, 107, 25, 85, 36, 145, 178], vec![11, 108, 199, 221, 70, 204, 169, 139, 238, 134, 92, 246, 5, 111, 214, 166, 17, 60, 191, 250, 228, 126, 166, 145, 135, 141, 147, 235, 28, 142, 94, 150]];
    let temp_b : &Vec<Vec<Vec<u8>>> = &vec![vec![vec![23, 92, 194, 108, 252, 68, 128, 186, 196, 33, 159, 82, 8, 217, 233, 176, 190, 104, 183, 61, 201, 101, 226, 57, 16, 84, 72, 239, 167, 0, 150, 14], vec![17, 38, 194, 210, 94, 65, 180, 38, 105, 90, 136, 26, 217, 53, 11, 99, 240, 14, 76, 58, 214, 251, 75, 37, 222, 128, 102, 241, 20, 75, 172, 37]], vec![vec![1, 2, 250, 36, 148, 124, 13, 32, 9, 129, 85, 138, 165, 21, 68, 104, 92, 88, 71, 64, 125, 1, 115, 208, 206, 243, 140, 204, 34, 119, 66, 57], vec![9, 47, 121, 233, 0, 72, 128, 199, 216, 84, 18, 239, 115, 123, 232, 126, 90, 233, 61, 117, 132, 46, 146, 105, 253, 154, 42, 186, 50, 170, 131, 204]]];
    let temp_c : &[Vec<u8>] = &[vec![2, 4, 48, 177, 29, 166, 90, 61, 73, 61, 205, 152, 124, 223, 171, 196, 21, 44, 174, 168, 94, 88, 70, 249, 179, 191, 226, 69, 235, 128, 129, 64], vec![41, 204, 22, 7, 21, 233, 206, 111, 228, 128, 180, 213, 61, 241, 32, 198, 96, 34, 54, 148, 86, 194, 144, 120, 22, 48, 98, 243, 180, 188, 224, 30]];
    let temp_pi = [[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 68, 125, 126, 18, 41, 19, 100, 219, 75, 197, 66, 17, 100, 136, 1, 41], [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 18, 196, 154, 210, 71, 210, 138, 50, 20, 126, 19, 97, 92, 108, 129, 249], [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 84, 180, 8, 39, 2, 187, 134, 214, 197, 231, 162, 96, 240, 66, 255, 170], [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 189, 73, 98, 22, 44, 240, 196, 239, 170, 12, 187, 223, 185, 46, 26, 228]] ;
    let proof: Proof<Bn254> = Proof {
        a: g1_from_bytes(temp_a).unwrap(),
        b: g2_from_bytes(&temp_b).unwrap(),
        c: g1_from_bytes(temp_c).unwrap(),
    };

    let pvk = prepared_verifying_key().unwrap();

    let public_inputs = temp_pi.iter().map(|x| {
        let temp = BigInt::from_bytes_be(Sign::Plus, x);
        <Bn254 as PairingEngine>::Fr::from_str(&temp.to_string()).unwrap()
    }).collect::<Vec<<Bn254 as PairingEngine>::Fr>>();

    let result = verify_proof(&pvk, &proof, &public_inputs);

    println!("Result: {:?}", result);

}

pub(crate) fn g1_from_bytes(elem: &[Vec<u8>]) -> Result<G1Affine, Error> {
    if elem.len() != 2 {
        return Err(anyhow!("Malformed G1 field element"));
    }
    let g1_affine: Vec<u8> = elem[0]
        .iter()
        .rev()
        .chain(elem[1].iter().rev())
        .cloned()
        .collect();

    Ok(get_g1_from_u8arr(elem))
    
    // G1Affine::deserialize_uncompressed(&*g1_affine).map_err(|err| anyhow!(err))
}

// Deserialize an element over the G2 group from bytes in big-endian format
pub(crate) fn g2_from_bytes(elem: &Vec<Vec<Vec<u8>>>) -> Result<G2Affine, Error> {
    if elem.len() != 2 || elem[0].len() != 2 || elem[1].len() != 2 {
        return Err(anyhow!("Malformed G2 field element"));
    }
    let g2_affine: Vec<u8> = elem[0][1]
        .iter()
        .rev()
        .chain(elem[0][0].iter().rev())
        .chain(elem[1][1].iter().rev())
        .chain(elem[1][0].iter().rev())
        .cloned()
        .collect();

    Ok(get_g2_from_u8arr(elem))
}

pub fn get_g1_from_u8arr(arr: &[Vec<u8>]) -> G1Affine {
    let temp_x = BigInt::from_bytes_be(Sign::Plus, &arr[0]);
    let temp_y = BigInt::from_bytes_be(Sign::Plus, &arr[1]);
    // Fr::from_str(&temp.to_string()).unwrap()
    let x = <G1Point as AffineCurve>::BaseField::from_str(&temp_x.to_string()).unwrap();
    let y = <G1Point as AffineCurve>::BaseField::from_str(&temp_y.to_string()).unwrap();

    G1Projective::new(x, y, <G1Projective as ProjectiveCurve>::BaseField::one()).into_affine()
}

pub fn get_g2_from_u8arr(arr: &Vec<Vec<Vec<u8>>>) -> G2Affine {
    let temp_x_1 = BigInt::from_bytes_be(Sign::Plus, &arr[0][0]);
    let temp_x_2 = BigInt::from_bytes_be(Sign::Plus, &arr[0][1]);

    let temp_y_1 = BigInt::from_bytes_be(Sign::Plus, &arr[1][0]);
    let temp_y_2 = BigInt::from_bytes_be(Sign::Plus, &arr[1][1]);


    let g2x1 = Fq::from_str(
        &temp_x_1.to_string(),
    )
    .unwrap();
    let g2x2 = Fq::from_str(
        &temp_x_2.to_string(),
    )
    .unwrap();
    let g2y1 = Fq::from_str(
        &temp_y_1.to_string(),
    )
            .unwrap();
    let g2y2 = Fq::from_str(
        &temp_y_2.to_string(),
    )
    .unwrap();

    G2Affine::new(Fq2::new(g2x1, g2x2), Fq2::new(g2y1, g2y2), true)
    // Fr::from_str(&temp.to_string()).unwrap()
    // let x_1 = <G2Point as AffineCurve>::BaseField::from_str(&temp_x_1.to_string()).unwrap();
    // let x_2 = <G2Point as AffineCurve>::BaseField:: (&temp_x_2.to_string()).unwrap();

    // let x_1 = <G2Point as AffineCurve>::BaseField::from_str (&temp_x_1.to_string()).unwrap();
    // let x_2 = <G2Point as AffineCurve>::BaseField::from_str(&temp_x_2.to_string()).unwrap();

    // let y_1 = <G2Point as AffineCurve>::BaseField::from_str(&temp_y_1.to_string()).unwrap();
    // let y_2 = <G2Point as AffineCurve>::BaseField::from_str(&temp_y_2.to_string()).unwrap();

    // G2Projective::new([x_1, x_2], [y_1, y_2], <G2Projective as ProjectiveCurve>::BaseField::one()).into_affine()
    
}

pub fn prepared_verifying_key() -> Result<PreparedVerifyingKey<Bn254>, Error> {
    let alpha_g1 = g1_from_bytes(&[from_u256(ALPHA_X)?, from_u256(ALPHA_Y)?])?;
    let beta_g2 = g2_from_bytes(&vec![
        vec![from_u256(BETA_X1)?, from_u256(BETA_X2)?],
        vec![from_u256(BETA_Y1)?, from_u256(BETA_Y2)?],
    ])?;
    let gamma_g2 = g2_from_bytes(&vec![
        vec![from_u256(GAMMA_X1)?, from_u256(GAMMA_X2)?],
        vec![from_u256(GAMMA_Y1)?, from_u256(GAMMA_Y2)?],
    ])?;
    let delta_g2 = g2_from_bytes(&vec![
        vec![from_u256(DELTA_X1)?, from_u256(DELTA_X2)?],
        vec![from_u256(DELTA_Y1)?, from_u256(DELTA_Y2)?],
    ])?;

    let ic0 = g1_from_bytes(&[from_u256(IC0_X)?, from_u256(IC0_Y)?])?;
    let ic1 = g1_from_bytes(&[from_u256(IC1_X)?, from_u256(IC1_Y)?])?;
    let ic2 = g1_from_bytes(&[from_u256(IC2_X)?, from_u256(IC2_Y)?])?;
    let ic3 = g1_from_bytes(&[from_u256(IC3_X)?, from_u256(IC3_Y)?])?;
    let ic4 = g1_from_bytes(&[from_u256(IC4_X)?, from_u256(IC4_Y)?])?;
    let gamma_abc_g1 = vec![ic0, ic1, ic2, ic3, ic4];

    let vk = VerifyingKey::<Bn254> {
        alpha_g1,
        beta_g2,
        gamma_g2,
        delta_g2,
        gamma_abc_g1,
    };

    Ok(prepare_verifying_key(&vk))
}


// Convert the U256 value to a byte array in big-endian format
pub fn from_u256(value: &str) -> Result<Vec<u8>, Error> {
    Ok(value.as_bytes().to_vec())
    // let value = if let Some(stripped) = value.strip_prefix("0x") {
    //     to_fixed_array(  ::decode(stripped).map_err(|_| anyhow!("conversion from u256 failed"))?)
    //         .to_vec()
    // } else {
    //     to_fixed_array(
    //         BigInt::from_str(value)
    //             .map_err(|_| anyhow!("conversion from u256 failed"))?
    //             .to_bytes_be()
    //             .1,
    //     )
    //     .to_vec()
    // };
    // Ok(value)
}


fn to_fixed_array(input: Vec<u8>) -> [u8; 32] {
    let mut fixed_array = [0u8; 32];
    let start = core::cmp::max(32, input.len()) - core::cmp::min(32, input.len());
    fixed_array[start..].copy_from_slice(&input[input.len().saturating_sub(32)..]);
    fixed_array
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_verify() {
        verify();
    }
}