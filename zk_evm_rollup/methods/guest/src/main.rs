#![no_main]
// If you want to try std support, also update the guest Cargo.toml file
// std support is experimental

use adapter_sdk::adapter_zkvm::verify_proof;
use adapter_sdk::types::AdapterPrivateInputs;
use adapter_sdk::types::AdapterPublicInputs;
use adapter_sdk::types::RollupProof;
// use demo_rollup_core::DemoProof;
// use demo_rollup_core::DemoRollupPublicInputs;
use zk_evm_rollup_core::ZkEvmProof;
use zk_evm_rollup_core::ZkEvmRollupPublicInputs;
use nexus_core::types::StatementDigest;
use risc0_zkvm::guest::env;
use risc0_zkvm::sha::Digest;

use ark_bn254::{g1, g1::Parameters, Bn254, FqParameters, Fr, FrParameters, G1Projective};
use ark_ec::short_weierstrass_jacobian::GroupAffine;
use ark_ec::*;
use ark_ff::{Field, Fp256, Fp256Parameters, One, PrimeField, UniformRand, Zero};

pub type G1Point = <Bn254 as PairingEngine>::G1Affine;
pub type G2Point = <Bn254 as PairingEngine>::G2Affine;

risc0_zkvm::guest::entry!(main);

pub fn get_dummy_proof() -> Option<RollupProof<ZkEvmRollupPublicInputs, ZkEvmProof>> {
    // let pr = vec![
    //     "12195165594784431822497303968938621279445690754376121387655513728730220550454",
    //     "19482351300768228183728567743975524187837254971200066453308487514712354412818",
    //     "270049702185508019342640204324826241417613526941291105097079886683911146886",
    //     "8044577183782099118358991257374623532841698893838076750142877485824795072127",
    //     "18899554350581376849619715242908819289791150067233598694602356239698407061017",
    //     "868483199604273061042760252576862685842931472081080113229115026384087738503",
    //     "15400234196629481957150851143665757067987965100904384175896686561307554593394",
    //     "1972554287366869807517068788787992038621302618305780153544292964897315682091",
    //     "13012702442141574024514112866712813523553321876510290446303561347565844930654",
    //     "6363613431504422665441435540021253583148414748729550612486380209002057984394",
    //     "16057866832337652851142304414708366836077577338023656646690877057031251541947",
    //     "12177497208173170035464583425607209406245985123797536695060336171641250404407",
    //     "1606928575748882874942488864331180511279674792603033713048693169239812670017",
    //     "12502690277925689095499239281542937835831064619179570213662273016815222024218",
    //     "21714950310348017755786780913378098925832975432250486683702036755613488957178",
    //     "7373645520955771058170141217317033724805640797155623483741097103589211150628",
    //     "10624974841759884514517518996672059640247361745924203600968035963539096078745",
    //     "12590031312322329503809710776715067780944838760473156014126576247831324341903",
    //     "17676078410435205056317710999346173532618821076911845052950090109177062725036",
    //     "13810130824095164415807955516712763121131180676617650812233616232528698737619",
    //     "9567903658565551430748252507556148460902008866092926659415720362326593620836",
    //     "17398514793767712415669438995039049448391479578008786242788501594157890722459",
    //     "11804645688707233673914574834599506530652461017683048951953032091830492459803",
    //     "6378827379501409574366452872421073840754012879130221505294134572417254316105",
    // ];

    // // for i in 0..pr.len() {
    // //     // println!("{}: {}", i, pr[i]);
    // //   let val = &U256::from_str(pr[i]).unwrap().to_string();
    // //   println!(" \"{},\" ", val);
    // // }

    // let c1_x = <G1Point as AffineCurve>::BaseField::from_str(pr[0]).unwrap();
    // let c1_y = <G1Point as AffineCurve>::BaseField::from_str(pr[1]).unwrap();
    // let c1_affine = G1Projective::new(
    //     c1_x,
    //     c1_y,
    //     <G1Projective as ProjectiveCurve>::BaseField::one(),
    // )
    // .into_affine();

    // let c2_x = <G1Point as AffineCurve>::BaseField::from_str(pr[2]).unwrap();
    // let c2_y = <G1Point as AffineCurve>::BaseField::from_str(pr[3]).unwrap();
    // let c2_affine = G1Projective::new(
    //     c2_x,
    //     c2_y,
    //     <G1Projective as ProjectiveCurve>::BaseField::one(),
    // )
    // .into_affine();

    // let w1_x = <G1Point as AffineCurve>::BaseField::from_str(pr[4]).unwrap();
    // let w1_y = <G1Point as AffineCurve>::BaseField::from_str(pr[5]).unwrap();
    // let w1_affine = G1Projective::new(
    //     w1_x,
    //     w1_y,
    //     <G1Projective as ProjectiveCurve>::BaseField::one(),
    // )
    // .into_affine();

    // let w2_x = <G1Point as AffineCurve>::BaseField::from_str(pr[6]).unwrap();
    // let w2_y = <G1Point as AffineCurve>::BaseField::from_str(pr[7]).unwrap();
    // let w2_affine = G1Projective::new(
    //     w2_x,
    //     w2_y,
    //     <G1Projective as ProjectiveCurve>::BaseField::one(),
    // )
    // .into_affine();

    // Proof {
    //     c1: c1_affine,
    //     c2: c2_affine,
    //     w1: w1_affine,
    //     w2: w2_affine,

    //     eval_ql: Fr::from_str(pr[8]).unwrap(),
    //     eval_qr: Fr::from_str(pr[9]).unwrap(),
    //     eval_qm: Fr::from_str(pr[10]).unwrap(),
    //     eval_qo: Fr::from_str(pr[11]).unwrap(),
    //     eval_qc: Fr::from_str(pr[12]).unwrap(),
    //     eval_s1: Fr::from_str(pr[13]).unwrap(),
    //     eval_s2: Fr::from_str(pr[14]).unwrap(),
    //     eval_s3: Fr::from_str(pr[15]).unwrap(),
    //     eval_a: Fr::from_str(pr[16]).unwrap(),
    //     eval_b: Fr::from_str(pr[17]).unwrap(),
    //     eval_c: Fr::from_str(pr[18]).unwrap(),
    //     eval_z: Fr::from_str(pr[19]).unwrap(),
    //     eval_zw: Fr::from_str(pr[20]).unwrap(),
    //     eval_t1w: Fr::from_str(pr[21]).unwrap(),
    //     eval_t2w: Fr::from_str(pr[22]).unwrap(),
    //     eval_inv: Fr::from_str(pr[23]).unwrap(),
    // }

    let proof = ZkEvmProof {
        c1: [0u64; 32],
        c2: [0u64; 32],
        w1: [0u64; 32],
        w2: [0u64; 32],

        // eval_ql: Fr::zero(),
        // eval_qr: Fr::zero(),
        // eval_qm: Fr::zero(),
        // eval_qo: Fr::zero(),
        // eval_qc: Fr::zero(),
        // eval_s1: Fr::zero(),
        // eval_s2: Fr::zero(),
        // eval_s3: Fr::zero(),
        // eval_a: Fr::zero(),
        // eval_b: Fr::zero(),
        // eval_c: Fr::zero(),
        // eval_z: Fr::zero(),
        // eval_zw: Fr::zero(),
        // eval_t1w: Fr::zero(),
        // eval_t2w: Fr::zero(),
        // eval_inv: Fr::zero(),
    };

    let public_inputs = ZkEvmRollupPublicInputs {
        prev_state_root: [0u8; 32].into(),
        post_state_root: [0u8; 32].into(),
        blob_hash: [0u8; 32].into(),
    };

    Some(RollupProof {
        proof,
        public_inputs,
    })
    
    

}

fn main() {
    let prev_adapter_public_inputs: Option<AdapterPublicInputs> = env::read();
    let proof: Option<RollupProof<ZkEvmRollupPublicInputs, ZkEvmProof>> = env::read();
    let private_inputs: AdapterPrivateInputs = env::read();
    let img_id: StatementDigest = env::read();
    let vk: [u8; 32] = env::read();

    let result = verify_proof(
        proof,
        prev_adapter_public_inputs,
        private_inputs,
        img_id,
        vk,
    )
    .unwrap();

    env::commit(&result);
}
