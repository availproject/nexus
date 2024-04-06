use adapter_sdk::{
    adapter_zkvm::verify_proof,
    state::AdapterState,
    types::{AdapterConfig, AdapterPrivateInputs, AdapterPublicInputs, RollupProof},
};
// use demo_rollup_core::{DemoProof, DemoRollupPublicInputs};
use zk_evm_rollup_core::{ZkEvmProof, ZkEvmRollupPublicInputs, ZkEvmVerificationKey};
use zk_evm_methods::{ADAPTER_ELF, ADAPTER_ID};
use nexus_core::types::{
    AppAccountId, AppId, AvailHeader, Header, StatementDigest, SubmitProof, TransactionV2,
    TxParamsV2, TxSignature, H256,
};
use risc0_zkvm::{default_prover, ExecutorEnv, InnerReceipt};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;
use std::time::Instant;

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

use std::fmt::{format, Debug, DebugMap, Display};
use std::hash::Hash;
use std::marker::PhantomData;
use std::ops::{Add, Mul, Neg, Sub};
use std::str::FromStr;
use std::vec;
use num_bigint::*;
use num_bigint::BigUint;


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

fn main() {
    let mut adapter: AdapterState<ZkEvmRollupPublicInputs, ZkEvmProof> = AdapterState::new(
        String::from("adapter_store"),
        AdapterConfig {
            app_id: AppId(100),
            elf: ADAPTER_ELF.to_vec(),
            adapter_elf_id: StatementDigest(ADAPTER_ID),
            vk: [0u8; 32],
            rollup_start_height: 606460,
        },
    );
    let rt = tokio::runtime::Runtime::new().unwrap();

    let proof_: ZkEvmProof = ZkEvmProof{
        c1_x: get_u8_arr_from_str("12195165594784431822497303968938621279445690754376121387655513728730220550454"),
        c1_y: get_u8_arr_from_str("19482351300768228183728567743975524187837254971200066453308487514712354412818"),
        c2_x: get_u8_arr_from_str("270049702185508019342640204324826241417613526941291105097079886683911146886"),
        c2_y: get_u8_arr_from_str("8044577183782099118358991257374623532841698893838076750142877485824795072127"),
        w1_x: get_u8_arr_from_str("18899554350581376849619715242908819289791150067233598694602356239698407061017"),
        w1_y: get_u8_arr_from_str("868483199604273061042760252576862685842931472081080113229115026384087738503"),
        w2_x: get_u8_arr_from_str("15400234196629481957150851143665757067987965100904384175896686561307554593394"),
        w2_y: get_u8_arr_from_str("1972554287366869807517068788787992038621302618305780153544292964897315682091"),
        eval_ql: get_u8_arr_from_str("13012702442141574024514112866712813523553321876510290446303561347565844930654"),
        eval_qr: get_u8_arr_from_str("6363613431504422665441435540021253583148414748729550612486380209002057984394"),
        eval_qm: get_u8_arr_from_str("16057866832337652851142304414708366836077577338023656646690877057031251541947"),
        eval_qo: get_u8_arr_from_str("12177497208173170035464583425607209406245985123797536695060336171641250404407"),
        eval_qc: get_u8_arr_from_str("1606928575748882874942488864331180511279674792603033713048693169239812670017"),
        eval_s1: get_u8_arr_from_str("12502690277925689095499239281542937835831064619179570213662273016815222024218"),
        eval_s2: get_u8_arr_from_str("21714950310348017755786780913378098925832975432250486683702036755613488957178"),
        eval_s3: get_u8_arr_from_str("7373645520955771058170141217317033724805640797155623483741097103589211150628"),
        eval_a: get_u8_arr_from_str("10624974841759884514517518996672059640247361745924203600968035963539096078745"),
        eval_b: get_u8_arr_from_str("12590031312322329503809710776715067780944838760473156014126576247831324341903"),
        eval_c: get_u8_arr_from_str("17676078410435205056317710999346173532618821076911845052950090109177062725036"),
        eval_z: get_u8_arr_from_str("13810130824095164415807955516712763121131180676617650812233616232528698737619"),
        eval_zw: get_u8_arr_from_str("9567903658565551430748252507556148460902008866092926659415720362326593620836"),
        eval_t1w: get_u8_arr_from_str("17398514793767712415669438995039049448391479578008786242788501594157890722459"),
        eval_t2w: get_u8_arr_from_str("11804645688707233673914574834599506530652461017683048951953032091830492459803"),
        eval_inv: get_u8_arr_from_str("6378827379501409574366452872421073840754012879130221505294134572417254316105"),
    };

    let proof = Some(RollupProof {
        proof: proof_,
        public_inputs: ZkEvmRollupPublicInputs {
            prev_state_root: [0u8; 32].into(),
            post_state_root: [0u8; 32].into(),
            blob_hash: [0u8; 32].into(),
            pub_signal: get_u8_arr_from_fr(Fr::from_str("14516932981781041565586298118536599721399535462624815668597272732223874827152").unwrap()).into()
        },
    });


// proof: Option<RollupProof<ZkEvmRollupPublicInputs, ZkEvmProof>>

    // let rollup_pi: DemoRollupPublicInputs = DemoRollupPublicInputs {
    //     prev_state_root: H256::zero(),
    //     post_state_root: H256::from([1u8; 32]),
    //     blob_hash: H256::zero(),
    // };
    let prev_adapter_public_inputs: Option<AdapterPublicInputs> = None;

    // //1. Inclusion proof to the blob root of that block. (Which is maintained by adapter when it finds blob in an Avail Block.)
    // //So this does not have completeness check.
    println!("Reading header.json");
    // // Open the JSON file
    let mut file = File::open("header.json").expect("Unable to open file");

    // // Read the contents of the file into a string
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Unable to read file");

    // // Deserialize the JSON into a Header object
    let current_header: AvailHeader = AvailHeader::from(
        &serde_json::from_str::<Header>(&contents).expect("Unable to parse JSON"),
    );

    let private_inputs: AdapterPrivateInputs = AdapterPrivateInputs {
        header: current_header.clone(),
        app_id: AppId(0),
    };
    println!("priv innputs prepared");

    let vk: [[u8; 32]; 6] = [
        get_u8_arr_from_str("7005013949998269612234996630658580519456097203281734268590713858661772481668"),
        get_u8_arr_from_str("869093939501355406318588453775243436758538662501260653214950591532352435323"),
        get_u8_arr_from_str("21831381940315734285607113342023901060522397560371972897001948545212302161822"),
        get_u8_arr_from_str("17231025384763736816414546592865244497437017442647097510447326538965263639101"),
        get_u8_arr_from_str("2388026358213174446665280700919698872609886601280537296205114254867301080648"),
        get_u8_arr_from_str("11507326595632554467052522095592665270651932854513688777769618397986436103170"),
    ];
    println!("vk prepared");

    let env = ExecutorEnv::builder()
        .write(&prev_adapter_public_inputs)
        .unwrap()
        .write(&proof)
        .unwrap()
        .write(&private_inputs)
        .unwrap()
        .write(&ADAPTER_ID)
        .unwrap()
        .write(&vk)
        .unwrap()
        .build()
        .unwrap();

    println!("sending proof to zkvm");

    // Measure time taken for the first proof
    let start_time_first_proof = Instant::now();
    let receipt = default_prover().prove(env, ADAPTER_ELF).unwrap();
    let end_time_first_proof = Instant::now();
    let time_taken_first_proof = end_time_first_proof.duration_since(start_time_first_proof);
    eprintln!("Time taken for first proof: {:?}", time_taken_first_proof);

    // let new_adapter_pi: AdapterPublicInputs = receipt.journal.decode().unwrap();

    // println!("First proof {:?}", new_adapter_pi);

    // let proof = match receipt.inner {
    //     InnerReceipt::Composite(i) => i,
    //     _ => panic!("Should have received a composite proof"),
    // };

    // let submit_tx: TransactionV2 = TransactionV2 {
    //     params: TxParamsV2::SubmitProof(SubmitProof {
    //         public_inputs: new_adapter_pi,
    //     }),
    //     signature: TxSignature([0u8; 64]),
    //     proof: Some(proof),
    // };

    // let json_data = serde_json::to_string_pretty(&submit_tx).expect("Serialization failed");

    // // Write to file
    // std::fs::write("submit_tx.json", json_data).expect("Failed to write to file");

    // // let new_rollup_pi: DemoRollupPublicInputs = DemoRollupPublicInputs {
    // //     prev_state_root: H256::from([1u8; 32]),
    // //     post_state_root: H256::from([2u8; 32]),
    // //     blob_hash: H256::zero(),
    // // };
    // // let new_header = AvailHeader {
    // //     parent_hash: current_header.hash(),
    // //     number: current_header.number + 1,
    // //     state_root: current_header.state_root,
    // //     extrinsics_root: current_header.extrinsics_root,
    // //     digest: current_header.digest,
    // //     extension: current_header.extension,
    // // };

    // // let new_private_inputs = AdapterPrivateInputs {
    // //     header: new_header.clone(),
    // //     avail_start_hash: private_inputs.avail_start_hash.clone(),
    // //     app_id: AppAccountId::from(AppId(0)),
    // // };

    // // let env = ExecutorEnv::builder()
    // //     .add_assumption(receipt)
    // //     .write(&proof)
    // //     .unwrap()
    // //     .write(&new_rollup_pi)
    // //     .unwrap()
    // //     .write(&Some(new_adapter_pi))
    // //     .unwrap()
    // //     .write(&ADAPTER_ID)
    // //     .unwrap()
    // //     .write(&new_private_inputs)
    // //     .unwrap()
    // //     .write(&[0u8; 32])
    // //     .unwrap()
    // //     .build()
    // //     .unwrap();

    // // // Measure time taken for the second proof
    // // let start_time_second_proof = Instant::now();
    // // let receipt_2 = default_prover().prove(env, ADAPTER_ELF).unwrap();
    // // let end_time_second_proof = Instant::now();
    // // let time_taken_second_proof = end_time_second_proof.duration_since(start_time_second_proof);
    // // println!("Time taken for second proof: {:?}", time_taken_second_proof);

    // // let latest_adapter_pi: AdapterPublicInputs = receipt_2.journal.decode().unwrap();

    // // println!("Second proof {:?}", latest_adapter_pi);
}
