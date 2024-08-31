use crate::types::{G1Point, G2Point, Proof, ProofWithPubSignal, VerificationKey};
use ark_bn254::{G2Affine, G2Projective, Fq2, Fq};
use ark_bn254::{g1, g1::Parameters, Bn254, FqParameters, Fr, FrParameters, G1Projective};
use ark_ec::short_weierstrass_jacobian::GroupAffine;
use ark_ec::*;
use ark_ff::{Field, Fp256, Fp256Parameters, One, PrimeField, UniformRand, Zero};
use ark_poly::univariate::DensePolynomial;
use ark_poly::{domain, Polynomial};
use std::str::FromStr;
use zksync_basic_types::{Address, H256, U128, U256};
use num_bigint::*;

#[cfg(any(feature = "native"))]
pub use zksync_types::commitment::serialize_commitments;
pub fn read_address(bytes: &[u8], start: usize) -> (Address, U256) {
    let mut address = [0u8; 20];
    address.copy_from_slice(&bytes[start..start + 20]);
    let offset = start + 20;
    (Address::from(address), offset.into())
}

pub fn read_uint256(bytes: &[u8], start: usize) -> (U256, U256) {
    let mut uint256_bytes = [0u8; 32];
    uint256_bytes.copy_from_slice(&bytes[start..start + 32]);

    let mut result = 0;
    for &byte in &uint256_bytes[16..] {
        // Take the last 16 bytes (128 bits)
        result = (result << 8) | (byte as u128);
    }

    let offset = start + 32;
    (U256::from(result), U256::from(offset))
}

pub fn read_bytes32(bytes: &[u8], start: usize) -> (H256, U256) {
    let mut result = [0u8; 32];
    result.copy_from_slice(&bytes[start..start + 32]);
    let offset = start + 32;
    (H256::from(result), offset.into())
}

pub fn get_mock_proof() -> Proof {
    // TODO: remove it just for testing purposes
    let proof_serialized: Vec<&str> = vec![
        "13419774624802246849707163164953093276102905224157218382320181624777730094068",
        "21702951633099711306717881145938427726799789327370707773122814631477010326205",
        "20186025367334498694255507504402678811940244188836904498062782206441848697250",
        "16039531690348160018090498256674443197528542990354829451107302259962928282850",
        "132707291782303765828572041778387086902962737637807421744660716727545804275",
        "110063284841104705126513555976886214816472330146558077207949117074241791349",
        "10003489969952560577405356760442895893875954195040619332956504442747180613154",
        "5431062914231260721023730790821643729646618161395572022717822152654921342187",
        "11489010572900159354684580506809109719134838013476998375592344229760640008599",
        "2638113933436337221703062945729350077401381962121795898130919953249927197444",
        "16998513425303411322236067579537841706334300256481002601087797911219860770809",
        "20417478078605457833907004561552685764864612968694445173946073746811586091306",
        "14782693845525078265438908440658243793041203991564373946541929764344862894591",
        "18751351158268880774449205205524859789341365522230672846245609740983975074112",
        "12611146933143204764044915444725197681187967780239120732939339646040787480430",
        "21306319803824267966205341592147903653225000405210120851482129437524110980396",
        "8811081182231387191999401003281389569226112418513108560122804586224615574537",
        "9274649608753394985258657215409094334037920658849368738034287135225354649885",
        "6028308143464493410384282363234825623137167894834407817708556402483084133943",
        "16230420712910140674643955884932561568524697496273495232382830984670392920171",
        "12150865231097461994057152020594610105574745202230538983376699350135698576955",
        "6928549005084698979552600871476048284905414340973400605654189108535781408023",
        "10435019220664372008499359080747895951310924684214688225732544859935610849671",
        "1424333514635203043939328263954837814879457310144145264750105124515452656983",
        "3450358486293537127324504305271392033185497751663728044346274819849145956032",
        "11530141611913171731248542631729239935228029508182290857439011572045480457823",
        "17288113056600248783152128925700821331404053057184461146099657608889976513337",
        "8058588498323128944327338705962525082707281716564140996306243190889186604119",
        "4222795565004432813942657071388729820012773684736853134408643208563488073501",
        "9499803568135428079090862242357576936025860340438801409326675059033540929524",
        "7566436829487309839509364709469849783026050058055377166289245318668570716108",
        "13124365195234288926862452165188944292836460111475691472855074569065236447852",
        "17841505314467970516583948425040858790005299584115409361139083818420057285736",
        "2512960923181905226378554824533297932688922479077898640832000621089797801111",
        "16086964716583776650663283846802441150977256989941654847527395622797547936132",
        "7213567273107098922381754787766220775323211779010967784908924097048259712955",
        "19172252481315166456087382663465592512634029215687953935504818522229046925392",
        "3357686465507005712931770449714694875940794508066076988079411044241924084472",
        "16325921429395035820426791792009454815050565359595004796415537872232916523949",
        "11458257946808337084263787849570177978180567534175083586860980183506047811175",
        "1352160172633061416286540469494796771972653521528356574548275629610779010675",
        "3748766158879395269948733317921102364237204740114227154935411576780102480940",
        "10975230023456464665540181117219046666224551826985633541271700416035282108578",
        "20809813771764421222673075724819887428395378515112529750700667784746361877281"
      ];
    let state_poly_0_x =
        <G1Point as AffineCurve>::BaseField::from_str(proof_serialized[0]).unwrap();
    let state_poly_0_y =
        <G1Point as AffineCurve>::BaseField::from_str(proof_serialized[1]).unwrap();
    let state_poly_0_affine = G1Projective::new(
        state_poly_0_x,
        state_poly_0_y,
        <G1Projective as ProjectiveCurve>::BaseField::one(),
    );

    let state_poly_1_x =
        <G1Point as AffineCurve>::BaseField::from_str(proof_serialized[2]).unwrap();
    let state_poly_1_y =
        <G1Point as AffineCurve>::BaseField::from_str(proof_serialized[3]).unwrap();
    let state_poly_1_affine = G1Projective::new(
        state_poly_1_x,
        state_poly_1_y,
        <G1Projective as ProjectiveCurve>::BaseField::one(),
    );

    let state_poly_2_x =
        <G1Point as AffineCurve>::BaseField::from_str(proof_serialized[4]).unwrap();
    let state_poly_2_y =
        <G1Point as AffineCurve>::BaseField::from_str(proof_serialized[5]).unwrap();
    let state_poly_2_affine = G1Projective::new(
        state_poly_2_x,
        state_poly_2_y,
        <G1Projective as ProjectiveCurve>::BaseField::one(),
    );

    let state_poly_3_x =
        <G1Point as AffineCurve>::BaseField::from_str(proof_serialized[6]).unwrap();
    let state_poly_3_y =
        <G1Point as AffineCurve>::BaseField::from_str(proof_serialized[7]).unwrap();
    let state_poly_3_affine = G1Projective::new(
        state_poly_3_x,
        state_poly_3_y,
        <G1Projective as ProjectiveCurve>::BaseField::one(),
    );

    let copy_permutation_grand_product_x =
        <G1Point as AffineCurve>::BaseField::from_str(proof_serialized[8]).unwrap();
    let copy_permutation_grand_product_y =
        <G1Point as AffineCurve>::BaseField::from_str(proof_serialized[9]).unwrap();
    let copy_permutation_grand_product_affine = G1Projective::new(
        copy_permutation_grand_product_x,
        copy_permutation_grand_product_y,
        <G1Projective as ProjectiveCurve>::BaseField::one(),
    );

    let lookup_s_poly_x =
        <G1Point as AffineCurve>::BaseField::from_str(proof_serialized[10]).unwrap();
    let lookup_s_poly_y =
        <G1Point as AffineCurve>::BaseField::from_str(proof_serialized[11]).unwrap();
    let lookup_s_poly_affine = G1Projective::new(
        lookup_s_poly_x,
        lookup_s_poly_y,
        <G1Projective as ProjectiveCurve>::BaseField::one(),
    );

    let lookup_grand_product_x =
        <G1Point as AffineCurve>::BaseField::from_str(proof_serialized[12]).unwrap();
    let lookup_grand_product_y =
        <G1Point as AffineCurve>::BaseField::from_str(proof_serialized[13]).unwrap();
    let lookup_grand_product_affine = G1Projective::new(
        lookup_grand_product_x,
        lookup_grand_product_y,
        <G1Projective as ProjectiveCurve>::BaseField::one(),
    );

    let quotient_poly_parts_0_x =
        <G1Point as AffineCurve>::BaseField::from_str(proof_serialized[14]).unwrap();
    let quotient_poly_parts_0_y =
        <G1Point as AffineCurve>::BaseField::from_str(proof_serialized[15]).unwrap();
    let quotient_poly_parts_0_affine = G1Projective::new(
        quotient_poly_parts_0_x,
        quotient_poly_parts_0_y,
        <G1Projective as ProjectiveCurve>::BaseField::one(),
    );

    let quotient_poly_parts_1_x =
        <G1Point as AffineCurve>::BaseField::from_str(proof_serialized[16]).unwrap();
    let quotient_poly_parts_1_y =
        <G1Point as AffineCurve>::BaseField::from_str(proof_serialized[17]).unwrap();
    let quotient_poly_parts_1_affine = G1Projective::new(
        quotient_poly_parts_1_x,
        quotient_poly_parts_1_y,
        <G1Projective as ProjectiveCurve>::BaseField::one(),
    );

    let quotient_poly_parts_2_x =
        <G1Point as AffineCurve>::BaseField::from_str(proof_serialized[18]).unwrap();
    let quotient_poly_parts_2_y =
        <G1Point as AffineCurve>::BaseField::from_str(proof_serialized[19]).unwrap();
    let quotient_poly_parts_2_affine = G1Projective::new(
        quotient_poly_parts_2_x,
        quotient_poly_parts_2_y,
        <G1Projective as ProjectiveCurve>::BaseField::one(),
    );

    let quotient_poly_parts_3_x =
        <G1Point as AffineCurve>::BaseField::from_str(proof_serialized[20]).unwrap();
    let quotient_poly_parts_3_y =
        <G1Point as AffineCurve>::BaseField::from_str(proof_serialized[21]).unwrap();
    let quotient_poly_parts_3_affine = G1Projective::new(
        quotient_poly_parts_3_x,
        quotient_poly_parts_3_y,
        <G1Projective as ProjectiveCurve>::BaseField::one(),
    );

    let state_poly_0_opening_at_z = Fr::from_str(proof_serialized[22]).unwrap();
    let state_poly_1_opening_at_z = Fr::from_str(proof_serialized[23]).unwrap();
    let state_poly_2_opening_at_z = Fr::from_str(proof_serialized[24]).unwrap();
    let state_poly_3_opening_at_z = Fr::from_str(proof_serialized[25]).unwrap();

    let state_poly_3_opening_at_z_omega = Fr::from_str(proof_serialized[26]).unwrap();
    let gate_selectors_0_opening_at_z = Fr::from_str(proof_serialized[27]).unwrap();

    let copy_permutation_polys_0_opening_at_z = Fr::from_str(proof_serialized[28]).unwrap();
    let copy_permutation_polys_1_opening_at_z = Fr::from_str(proof_serialized[29]).unwrap();
    let copy_permutation_polys_2_opening_at_z = Fr::from_str(proof_serialized[30]).unwrap();

    let copy_permutation_grand_product_opening_at_z_omega =
        Fr::from_str(proof_serialized[31]).unwrap();
    let lookup_s_poly_opening_at_z_omega = Fr::from_str(proof_serialized[32]).unwrap();
    let lookup_grand_product_opening_at_z_omega = Fr::from_str(proof_serialized[33]).unwrap();
    let lookup_t_poly_opening_at_z = Fr::from_str(proof_serialized[34]).unwrap();
    let lookup_t_poly_opening_at_z_omega = Fr::from_str(proof_serialized[35]).unwrap();
    let lookup_selector_poly_opening_at_z = Fr::from_str(proof_serialized[36]).unwrap();
    let lookup_table_type_poly_opening_at_z = Fr::from_str(proof_serialized[37]).unwrap();
    let quotient_poly_opening_at_z = Fr::from_str(proof_serialized[38]).unwrap();
    let linearisation_poly_opening_at_z = Fr::from_str(proof_serialized[39]).unwrap();

    let opening_proof_at_z_x =
        <G1Point as AffineCurve>::BaseField::from_str(proof_serialized[40]).unwrap();
    let opening_proof_at_z_y =
        <G1Point as AffineCurve>::BaseField::from_str(proof_serialized[41]).unwrap();
    let opening_proof_at_z_affine = G1Projective::new(
        opening_proof_at_z_x,
        opening_proof_at_z_y,
        <G1Projective as ProjectiveCurve>::BaseField::one(),
    );

    let opening_proof_at_z_omega_x =
        <G1Point as AffineCurve>::BaseField::from_str(proof_serialized[42]).unwrap();
    let opening_proof_at_z_omega_y =
        <G1Point as AffineCurve>::BaseField::from_str(proof_serialized[43]).unwrap();
    let opening_proof_at_z_omega_affine = G1Projective::new(
        opening_proof_at_z_omega_x,
        opening_proof_at_z_omega_y,
        <G1Projective as ProjectiveCurve>::BaseField::one(),
    );

    Proof {
        state_poly_0: state_poly_0_affine.into(),
        state_poly_1: state_poly_1_affine.into(),
        state_poly_2: state_poly_2_affine.into(),
        state_poly_3: state_poly_3_affine.into(),
        copy_permutation_grand_product: copy_permutation_grand_product_affine.into(),
        lookup_s_poly: lookup_s_poly_affine.into(),
        lookup_grand_product: lookup_grand_product_affine.into(),
        quotient_poly_parts_0: quotient_poly_parts_0_affine.into(),
        quotient_poly_parts_1: quotient_poly_parts_1_affine.into(),
        quotient_poly_parts_2: quotient_poly_parts_2_affine.into(),
        quotient_poly_parts_3: quotient_poly_parts_3_affine.into(),
        state_poly_0_opening_at_z,
        state_poly_1_opening_at_z,
        state_poly_2_opening_at_z,
        state_poly_3_opening_at_z,
        state_poly_3_opening_at_z_omega,
        gate_selectors_0_opening_at_z,
        copy_permutation_polys_0_opening_at_z,
        copy_permutation_polys_1_opening_at_z,
        copy_permutation_polys_2_opening_at_z,
        copy_permutation_grand_product_opening_at_z_omega,
        lookup_s_poly_opening_at_z_omega,
        lookup_grand_product_opening_at_z_omega,
        lookup_t_poly_opening_at_z,
        lookup_t_poly_opening_at_z_omega,
        lookup_selector_poly_opening_at_z,
        lookup_table_type_poly_opening_at_z,
        quotient_poly_opening_at_z,
        linearisation_poly_opening_at_z,
        opening_proof_at_z: opening_proof_at_z_affine.into(),
        opening_proof_at_z_omega: opening_proof_at_z_omega_affine.into(),
    }
}

pub fn get_pub_signal() -> Fp256<FrParameters> {
    Fr::from_str("14516932981781041565586298118536599721399535462624815668597272732223874827152")
        .unwrap()
}

pub fn get_verification_key() -> VerificationKey {
    VerificationKey {
        gate_setup: vec![
            G1Projective::new(
                <G1Point as AffineCurve>::BaseField::from_str(
                    &BigInt::parse_bytes(
                        "110deb1e0863737f9a3d7b4de641a03aa00a77bc9f1a05acc9d55b76ab9fdd4d"
                            .as_bytes(),
                        16,
                    )
                    .unwrap()
                    .to_string(),
                )
                .unwrap(),
                <G1Point as AffineCurve>::BaseField::from_str(
                    &BigInt::parse_bytes(
                        "2c9dc252441e9298b7f6df6335a252517b7bccb924adf537b87c5cd3383fd7a9"
                            .as_bytes(),
                        16,
                    )
                    .unwrap()
                    .to_string(),
                )
                .unwrap(),
                <G1Projective as ProjectiveCurve>::BaseField::one(),
            )
            .into_affine(),
            G1Projective::new(
                <G1Point as AffineCurve>::BaseField::from_str(
                    &BigInt::parse_bytes(
                        "04659caf7b05471ba5ba85b1ab62267aa6c456836e625f169f7119d55b9462d2"
                            .as_bytes(),
                        16,
                    )
                    .unwrap()
                    .to_string(),
                )
                .unwrap(),
                <G1Point as AffineCurve>::BaseField::from_str(
                    &BigInt::parse_bytes(
                        "0ea63403692148d2ad22189a1e5420076312f4d46e62036a043a6b0b84d5b410"
                            .as_bytes(),
                        16,
                    )
                    .unwrap()
                    .to_string(),
                )
                .unwrap(),
                <G1Projective as ProjectiveCurve>::BaseField::one(),
            )
            .into_affine(),
            G1Projective::new(
                <G1Point as AffineCurve>::BaseField::from_str(
                    &BigInt::parse_bytes(
                        "0e6696d09d65fce1e42805be03fca1f14aea247281f688981f925e77d4ce2291"
                            .as_bytes(),
                        16,
                    )
                    .unwrap()
                    .to_string(),
                )
                .unwrap(),
                <G1Point as AffineCurve>::BaseField::from_str(
                    &BigInt::parse_bytes(
                        "0228f6cf8fe20c1e07e5b78bf8c41d50e55975a126d22a198d1e56acd4bbb3dd"
                            .as_bytes(),
                        16,
                    )
                    .unwrap()
                    .to_string(),
                )
                .unwrap(),
                <G1Projective as ProjectiveCurve>::BaseField::one(),
            )
            .into_affine(),
            G1Projective::new(
                <G1Point as AffineCurve>::BaseField::from_str(
                    &BigInt::parse_bytes(
                        "14685dafe340b1dec5eafcd5e7faddaf24f3781ddc53309cc25d0b42c00541dd"
                            .as_bytes(),
                        16,
                    )
                    .unwrap()
                    .to_string(),
                )
                .unwrap(),
                <G1Point as AffineCurve>::BaseField::from_str(
                    &BigInt::parse_bytes(
                        "0e651cff9447cb360198899b80fa23e89ec13bc94ff161729aa841d2b55ea5be"
                            .as_bytes(),
                        16,
                    )
                    .unwrap()
                    .to_string(),
                )
                .unwrap(),
                <G1Projective as ProjectiveCurve>::BaseField::one(),
            )
            .into_affine(),
            G1Projective::new(
                <G1Point as AffineCurve>::BaseField::from_str(
                    &BigInt::parse_bytes(
                        "16e9ef76cb68f2750eb0ee72382dd9911a982308d0ab10ef94dada13c382ae73"
                            .as_bytes(),
                        16,
                    )
                    .unwrap()
                    .to_string(),
                )
                .unwrap(),
                <G1Point as AffineCurve>::BaseField::from_str(
                    &BigInt::parse_bytes(
                        "22e404bc91350f3bc7daad1d1025113742436983c85eac5ab7b42221a181b81e"
                            .as_bytes(),
                        16,
                    )
                    .unwrap()
                    .to_string(),
                )
                .unwrap(),
                <G1Projective as ProjectiveCurve>::BaseField::one(),
            )
            .into_affine(),
            G1Projective::new(
                <G1Point as AffineCurve>::BaseField::from_str(
                    &BigInt::parse_bytes(
                        "0d9b29613037a5025655c82b143d2b7449c98f3aea358307c8529249cc54f3b9"
                            .as_bytes(),
                        16,
                    )
                    .unwrap()
                    .to_string(),
                )
                .unwrap(),
                <G1Point as AffineCurve>::BaseField::from_str(
                    &BigInt::parse_bytes(
                        "15b3c4c946ad1babfc4c03ff7c2423fd354af3a9305c499b7fb3aaebe2fee746"
                            .as_bytes(),
                        16,
                    )
                    .unwrap()
                    .to_string(),
                )
                .unwrap(),
                <G1Projective as ProjectiveCurve>::BaseField::one(),
            )
            .into_affine(),
            G1Projective::new(
                <G1Point as AffineCurve>::BaseField::from_str(
                    &BigInt::parse_bytes(
                        "2a4cb6c495dbc7201142cc773da895ae2046e790073988fb850aca6aead27b8a"
                            .as_bytes(),
                        16,
                    )
                    .unwrap()
                    .to_string(),
                )
                .unwrap(),
                <G1Point as AffineCurve>::BaseField::from_str(
                    &BigInt::parse_bytes(
                        "28ef9200c3cb67da82030520d640292014f5f7c2e2909da608812e04671a3acf"
                            .as_bytes(),
                        16,
                    )
                    .unwrap()
                    .to_string(),
                )
                .unwrap(),
                <G1Projective as ProjectiveCurve>::BaseField::one(),
            )
            .into_affine(),
            G1Projective::new(
                <G1Point as AffineCurve>::BaseField::from_str(
                    &BigInt::parse_bytes(
                        "283344a1ab3e55ecfd904d0b8e9f4faea338df5a4ead2fa9a42f0e103da40abc"
                            .as_bytes(),
                        16,
                    )
                    .unwrap()
                    .to_string(),
                )
                .unwrap(),
                <G1Point as AffineCurve>::BaseField::from_str(
                    &BigInt::parse_bytes(
                        "223b37b83b9687512d322993edd70e508dd80adb10bcf7321a3cc8a44c269521"
                            .as_bytes(),
                        16,
                    )
                    .unwrap()
                    .to_string(),
                )
                .unwrap(),
                <G1Projective as ProjectiveCurve>::BaseField::one(),
            )
            .into_affine(),
        ],
        gate_selectors: vec![
            G1Projective::new(
                <G1Point as AffineCurve>::BaseField::from_str(
                    &BigInt::parse_bytes(
                        "1f67f0ba5f7e837bc680acb4e612ebd938ad35211aa6e05b96cad19e66b82d2d"
                            .as_bytes(),
                        16,
                    )
                    .unwrap()
                    .to_string(),
                )
                .unwrap(),
                <G1Point as AffineCurve>::BaseField::from_str(
                    &BigInt::parse_bytes(
                        "2820641a84d2e8298ac2ac42bd4b912c0c37f768ecc83d3a29e7c720763d15a1"
                            .as_bytes(),
                        16,
                    )
                    .unwrap()
                    .to_string(),
                )
                .unwrap(),
                <G1Projective as ProjectiveCurve>::BaseField::one(),
            )
            .into_affine(),
            G1Projective::new(
                <G1Point as AffineCurve>::BaseField::from_str(
                    &BigInt::parse_bytes(
                        "0353257957562270292a17860ca8e8827703f828f440ee004848b1e23fdf9de2"
                            .as_bytes(),
                        16,
                    )
                    .unwrap()
                    .to_string(),
                )
                .unwrap(),
                <G1Point as AffineCurve>::BaseField::from_str(
                    &BigInt::parse_bytes(
                        "305f4137fee253dff8b2bfe579038e8f25d5bd217865072af5d89fc8800ada24"
                            .as_bytes(),
                        16,
                    )
                    .unwrap()
                    .to_string(),
                )
                .unwrap(),
                <G1Projective as ProjectiveCurve>::BaseField::one(),
            )
            .into_affine(),
        ],
        permutation: vec![
            G1Projective::new(
                <G1Point as AffineCurve>::BaseField::from_str(
                    &BigInt::parse_bytes(
                        "13a600154b369ff3237706d00948e465ee1c32c7a6d3e18bccd9c4a15910f2e5"
                            .as_bytes(),
                        16,
                    )
                    .unwrap()
                    .to_string(),
                )
                .unwrap(),
                <G1Point as AffineCurve>::BaseField::from_str(
                    &BigInt::parse_bytes(
                        "138aa24fbf4cdddc75114811b3d59040394c218ecef3eb46ef9bd646f7e53776"
                            .as_bytes(),
                        16,
                    )
                    .unwrap()
                    .to_string(),
                )
                .unwrap(),
                <G1Projective as ProjectiveCurve>::BaseField::one(),
            )
            .into_affine(),
            G1Projective::new(
                <G1Point as AffineCurve>::BaseField::from_str(
                    &BigInt::parse_bytes(
                        "277fff1f80c409357e2d251d79f6e3fd2164b755ce69cfd72de5c690289df662"
                            .as_bytes(),
                        16,
                    )
                    .unwrap()
                    .to_string(),
                )
                .unwrap(),
                <G1Point as AffineCurve>::BaseField::from_str(
                    &BigInt::parse_bytes(
                        "25235588e28c70eea3e35531c80deac25cd9b53ea3f98993f120108bc7abf670"
                            .as_bytes(),
                        16,
                    )
                    .unwrap()
                    .to_string(),
                )
                .unwrap(),
                <G1Projective as ProjectiveCurve>::BaseField::one(),
            )
            .into_affine(),
            G1Projective::new(
                <G1Point as AffineCurve>::BaseField::from_str(
                    &BigInt::parse_bytes(
                        "0990e07a9b001048b947d0e5bd6157214c7359b771f01bf52bd771ba563a900e"
                            .as_bytes(),
                        16,
                    )
                    .unwrap()
                    .to_string(),
                )
                .unwrap(),
                <G1Point as AffineCurve>::BaseField::from_str(
                    &BigInt::parse_bytes(
                        "05e5fb090dd40914c8606d875e301167ae3047d684a02b44d9d36f1eaf43d0b4"
                            .as_bytes(),
                        16,
                    )
                    .unwrap()
                    .to_string(),
                )
                .unwrap(),
                <G1Projective as ProjectiveCurve>::BaseField::one(),
            )
            .into_affine(),
            G1Projective::new(
                <G1Point as AffineCurve>::BaseField::from_str(
                    &BigInt::parse_bytes(
                        "1d4656690b33299db5631401a282afab3e16c78ee2c9ad9efea628171dcbc6bc"
                            .as_bytes(),
                        16,
                    )
                    .unwrap()
                    .to_string(),
                )
                .unwrap(),
                <G1Point as AffineCurve>::BaseField::from_str(
                    &BigInt::parse_bytes(
                        "0ebda2ebe582f601f813ec1e3970d13ef1500c742a85cce9b7f190f333de03b0"
                            .as_bytes(),
                        16,
                    )
                    .unwrap()
                    .to_string(),
                )
                .unwrap(),
                <G1Projective as ProjectiveCurve>::BaseField::one(),
            )
            .into_affine(),
        ],
        lookup_table: vec![
            G1Projective::new(
                <G1Point as AffineCurve>::BaseField::from_str(
                    &BigInt::parse_bytes(
                        "2c513ed74d9d57a5ec901e074032741036353a2c4513422e96e7b53b302d765b"
                            .as_bytes(),
                        16,
                    )
                    .unwrap()
                    .to_string(),
                )
                .unwrap(),
                <G1Point as AffineCurve>::BaseField::from_str(
                    &BigInt::parse_bytes(
                        "04dd964427e430f16004076d708c0cb21e225056cc1d57418cfbd3d472981468"
                            .as_bytes(),
                        16,
                    )
                    .unwrap()
                    .to_string(),
                )
                .unwrap(),
                <G1Projective as ProjectiveCurve>::BaseField::one(),
            )
            .into_affine(),
            G1Projective::new(
                <G1Point as AffineCurve>::BaseField::from_str(
                    &BigInt::parse_bytes(
                        "1ea83e5e65c6f8068f4677e2911678cf329b28259642a32db1f14b8347828aac"
                            .as_bytes(),
                        16,
                    )
                    .unwrap()
                    .to_string(),
                )
                .unwrap(),
                <G1Point as AffineCurve>::BaseField::from_str(
                    &BigInt::parse_bytes(
                        "1d22bc884a2da4962a893ba8de13f57aaeb785ed52c5e686994839cab8f7475d"
                            .as_bytes(),
                        16,
                    )
                    .unwrap()
                    .to_string(),
                )
                .unwrap(),
                <G1Projective as ProjectiveCurve>::BaseField::one(),
            )
            .into_affine(),
            G1Projective::new(
                <G1Point as AffineCurve>::BaseField::from_str(
                    &BigInt::parse_bytes(
                        "0b2e7212d0d9cff26d0bdf3d79b2cac029a25dfeb1cafdf49e2349d7db348d89"
                            .as_bytes(),
                        16,
                    )
                    .unwrap()
                    .to_string(),
                )
                .unwrap(),
                <G1Point as AffineCurve>::BaseField::from_str(
                    &BigInt::parse_bytes(
                        "1301f9b252419ea240eb67fda720ca0b16d92364027285f95e9b1349490fa283"
                            .as_bytes(),
                        16,
                    )
                    .unwrap()
                    .to_string(),
                )
                .unwrap(),
                <G1Projective as ProjectiveCurve>::BaseField::one(),
            )
            .into_affine(),
            G1Projective::new(
                <G1Point as AffineCurve>::BaseField::from_str(
                    &BigInt::parse_bytes(
                        "02f7b99fdfa5b418548c2d777785820e02383cfc87e7085e280a375a358153bf"
                            .as_bytes(),
                        16,
                    )
                    .unwrap()
                    .to_string(),
                )
                .unwrap(),
                <G1Point as AffineCurve>::BaseField::from_str(
                    &BigInt::parse_bytes(
                        "09d004fe08dc4d19c382df36fad22ef676185663543703e6a4b40203e50fd8a6"
                            .as_bytes(),
                        16,
                    )
                    .unwrap()
                    .to_string(),
                )
                .unwrap(),
                <G1Projective as ProjectiveCurve>::BaseField::one(),
            )
            .into_affine(),
        ],

        lookup_selector: G1Projective::new(
            <G1Point as AffineCurve>::BaseField::from_str(
                &BigInt::parse_bytes(
                    "2f4d347c7fb61daaadfff881e24f4b5dcfdc0d70a95bcb148168b90ef93e0007".as_bytes(),
                    16,
                )
                .unwrap()
                .to_string(),
            )
            .unwrap(),
            <G1Point as AffineCurve>::BaseField::from_str(
                &BigInt::parse_bytes(
                    "2322632465ba8e28cd0a4befd813ea85a972f4f6fa8e8603cf5d062dbcb14065".as_bytes(),
                    16,
                )
                .unwrap()
                .to_string(),
            )
            .unwrap(),
            <G1Projective as ProjectiveCurve>::BaseField::one(),
        )
        .into_affine(),
        lookup_table_type: G1Projective::new(
            <G1Point as AffineCurve>::BaseField::from_str(
                &BigInt::parse_bytes(
                    "1e3c9fc98c118e4bc34f1f93d214a5d86898e980c40d8e2c180c6ada377a7467".as_bytes(),
                    16,
                )
                .unwrap()
                .to_string(),
            )
            .unwrap(),
            <G1Point as AffineCurve>::BaseField::from_str(
                &BigInt::parse_bytes(
                    "2260a13535c35a15c173f5e5797d4b675b55d164a9995bfb7624971324bd84a8".as_bytes(),
                    16,
                )
                .unwrap()
                .to_string(),
            )
            .unwrap(),
            <G1Projective as ProjectiveCurve>::BaseField::one(),
        )
        .into_affine(),
        recursive_flag: false,
    }
}

pub fn get_g2_elements() -> (G2Affine, G2Affine) {
    let g2_0_x1 = Fq::from_str("11559732032986387107991004021392285783925812861821192530917403151452391805634").unwrap();
    let g2_0_x2 = Fq::from_str("10857046999023057135944570762232829481370756359578518086990519993285655852781").unwrap();
    let g2_0_y1 = Fq::from_str("4082367875863433681332203403145435568316851327593401208105741076214120093531").unwrap();
    let g2_0_y2 = Fq::from_str("8495653923123431417604973247489272438418190587263600148770280649306958101930").unwrap();
    let g2_element_0 = G2Affine::new(Fq2::new(g2_0_x1, g2_0_x2), Fq2::new(g2_0_y1, g2_0_y2), true);

    let g2_1_x1 = Fq::from_str("17212635814319756364507010169094758005397460366678210664966334781961899574209").unwrap();
    let g2_1_x2 = Fq::from_str("496075682290949347282619629729389528669750910289829251317610107342504362928").unwrap();
    let g2_1_y1 = Fq::from_str("2255182984359105691812395885056400739448730162863181907784180250290003009508").unwrap();
    let g2_1_y2 = Fq::from_str("15828724851114720558251891430452666121603726704878231219287131634746610441813").unwrap();
    let g2_element_1 = G2Affine::new(Fq2::new(g2_1_x1, g2_1_x2), Fq2::new(g2_1_y1, g2_1_y2), true);

    (g2_element_0, g2_element_1)
}

pub fn get_public_inputs() -> Fp256<FrParameters> {
    let ttt = get_fr_mask().into_repr().0[0] & get_fr_mask().into_repr().0[1];
    let pi = Fr::from_str("1791774931943744072805199616601928379125307843078816245551496799836")
        .unwrap();
    let mut res = apply_fr_mask(padd_bytes32(get_u8arr_from_fr(pi)));
    get_fr_from_u8arr(res)
}

pub fn get_u8arr_from_fq(fq: Fp256<FqParameters>) -> Vec<u8> {
    let mut st = fq.to_string();
    let temp = &st[8..8 + 64];
    BigInt::parse_bytes(temp.as_bytes(), 16)
        .unwrap()
        .to_bytes_be()
        .1
}

pub fn get_u8arr_from_fr(fr: Fp256<FrParameters>) -> Vec<u8> {
    get_bigint_from_fr(fr).to_bytes_be().1
}

pub fn get_fr_from_u8arr(arr: Vec<u8>) -> Fp256<FrParameters> {
    let temp = BigInt::from_bytes_be(Sign::Plus, &arr);
    Fr::from_str(&temp.to_string()).unwrap()
}

pub fn get_bigint_from_fr(fr: Fp256<FrParameters>) -> BigInt {
    let mut st = fr.to_string();
    let temp = &st[8..8 + 64];
    BigInt::parse_bytes(temp.as_bytes(), 16).unwrap()
}

pub fn padd_bytes32(input: Vec<u8>) -> Vec<u8> {
    let mut result = input.clone();
    let mut padding = vec![0; 32 - input.len()];
    padding.append(&mut result);
    // result.append(&mut padding);
    padding
}

pub fn padd_bytes3(input: Vec<u8>) -> Vec<u8> {
    let mut result = input.clone();
    let mut padding = vec![0; 3 - input.len()];
    padding.append(&mut result);
    // result.append(&mut padding);
    padding
}

pub fn apply_fr_mask(out: Vec<u8>) -> Vec<u8> {
    const FR_MASK: [u8; 32] = [
        0x1f, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        0xff, 0xff,
    ];
    let mut res_fr = [0u8; 32];

    for i in 0..32 {
        res_fr[i] = out[i] & FR_MASK[i];
    }

    res_fr.to_vec()
}

pub fn get_fr_mask() -> Fp256<FrParameters> {
    Fr::from_str("14474011154664524427946373126085988481658748083205070504932198000989141204991")
        .unwrap()
}

pub fn get_domain_size() -> u64 {
    16777216
}

pub fn get_scalar_field() -> Fp256<FrParameters> {
    Fr::from_str("21888242871839275222246405745257275088548364400416034343698204186575808495617")
        .unwrap()
}

pub fn get_omega() -> Fp256<FrParameters> {
    Fr::from_str("11451405578697956743456240853980216273390554734748796433026540431386972584651")
        .unwrap()
}
