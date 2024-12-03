use ark_bn254::{
    g1, g1::Parameters, Bn254, Fq, FqParameters, Fr, FrParameters, G1Projective, G2Projective,
};
use ark_bn254::{g2, Fq2, Fq2Parameters, G2Affine};
use ark_ec::group::Group;
use ark_ec::short_weierstrass_jacobian::GroupAffine;
use ark_ec::AffineCurve;
use ark_ec::PairingEngine;
use ark_ec::ProjectiveCurve;
use ark_ff::{
    field_new, Field, Fp256, Fp256Parameters, Fp2ParamsWrapper, One, PrimeField, QuadExtField,
    UniformRand, Zero,
};
use ark_poly::{domain, Polynomial};
use ethers_core::k256::U256;
use num_bigint::*;

use std::fmt::{format, Debug, DebugMap, Display};
use std::hash::Hash;
use std::marker::PhantomData;
use std::ops::{Add, Mul, Neg, Sub};
use std::str::FromStr;
use std::vec;
use tiny_keccak::{Hasher, Keccak};

use crate::transcript::Transcript;
use crate::types::{G1Point, PartialVerifierState, Proof};
use crate::utils::{
    apply_fr_mask, get_bigint_from_fr, get_domain_size, get_fr_from_u8arr, get_fr_mask,
    get_g2_elements, get_omega, get_pub_signal, get_public_inputs, get_scalar_field,
    get_u8arr_from_fq, get_u8arr_from_fr, get_verification_key, padd_bytes3, padd_bytes32,
    parse_proof,
};

pub struct ZksyncVerifier;

impl ZksyncVerifier {
    pub fn new() -> Self {
        Self
    }

    fn compute_challenges(pvs: &mut PartialVerifierState, public_input: Fr, proof: Proof) {
        let mut transcript = Transcript::new_transcript();

        transcript.update_transcript(&get_u8arr_from_fr(public_input));
        transcript.update_transcript(&get_u8arr_from_fq(proof.state_poly_0.x));
        transcript.update_transcript(&get_u8arr_from_fq(proof.state_poly_0.y));
        transcript.update_transcript(&get_u8arr_from_fq(proof.state_poly_1.x));
        transcript.update_transcript(&get_u8arr_from_fq(proof.state_poly_1.y));
        transcript.update_transcript(&get_u8arr_from_fq(proof.state_poly_2.x));
        transcript.update_transcript(&get_u8arr_from_fq(proof.state_poly_2.y));
        transcript.update_transcript(&get_u8arr_from_fq(proof.state_poly_3.x));
        transcript.update_transcript(&get_u8arr_from_fq(proof.state_poly_3.y));

        let etaaa = transcript.get_transcript_challenge(0);

        //round 1.5
        transcript.update_transcript(&get_u8arr_from_fq(proof.lookup_s_poly.x));
        transcript.update_transcript(&get_u8arr_from_fq(proof.lookup_s_poly.y));

        let betaa = transcript.get_transcript_challenge(1);
        let gammma = transcript.get_transcript_challenge(2);

        //round 2

        transcript.update_transcript(&get_u8arr_from_fq(proof.copy_permutation_grand_product.x));
        transcript.update_transcript(&get_u8arr_from_fq(proof.copy_permutation_grand_product.y));

        let beta_lookuppp = transcript.get_transcript_challenge(3);
        let gamma_lookuppp = transcript.get_transcript_challenge(4);

        //round 2.5

        transcript.update_transcript(&get_u8arr_from_fq(proof.lookup_grand_product.x));
        transcript.update_transcript(&get_u8arr_from_fq(proof.lookup_grand_product.y));

        let alphaaa = transcript.get_transcript_challenge(5);

        //round 3

        transcript.update_transcript(&get_u8arr_from_fq(proof.quotient_poly_parts_0.x));
        transcript.update_transcript(&get_u8arr_from_fq(proof.quotient_poly_parts_0.y));
        transcript.update_transcript(&get_u8arr_from_fq(proof.quotient_poly_parts_1.x));
        transcript.update_transcript(&get_u8arr_from_fq(proof.quotient_poly_parts_1.y));
        transcript.update_transcript(&get_u8arr_from_fq(proof.quotient_poly_parts_2.x));
        transcript.update_transcript(&get_u8arr_from_fq(proof.quotient_poly_parts_2.y));
        transcript.update_transcript(&get_u8arr_from_fq(proof.quotient_poly_parts_3.x));
        transcript.update_transcript(&get_u8arr_from_fq(proof.quotient_poly_parts_3.y));

        let zz = transcript.get_transcript_challenge(6);
        let zz_in_domain_size = get_fr_from_u8arr(zz.to_vec()).pow([get_domain_size()]);

        //round 4

        transcript.update_transcript(&get_u8arr_from_fr(proof.quotient_poly_opening_at_z));

        transcript.update_transcript(&get_u8arr_from_fr(proof.state_poly_0_opening_at_z));
        transcript.update_transcript(&get_u8arr_from_fr(proof.state_poly_1_opening_at_z));
        transcript.update_transcript(&get_u8arr_from_fr(proof.state_poly_2_opening_at_z));
        transcript.update_transcript(&get_u8arr_from_fr(proof.state_poly_3_opening_at_z));

        transcript.update_transcript(&get_u8arr_from_fr(proof.state_poly_3_opening_at_z_omega));
        transcript.update_transcript(&get_u8arr_from_fr(proof.gate_selectors_0_opening_at_z));

        transcript.update_transcript(&get_u8arr_from_fr(
            proof.copy_permutation_polys_0_opening_at_z,
        ));
        transcript.update_transcript(&get_u8arr_from_fr(
            proof.copy_permutation_polys_1_opening_at_z,
        ));
        transcript.update_transcript(&get_u8arr_from_fr(
            proof.copy_permutation_polys_2_opening_at_z,
        ));

        transcript.update_transcript(&get_u8arr_from_fr(
            proof.copy_permutation_grand_product_opening_at_z_omega,
        ));
        transcript.update_transcript(&get_u8arr_from_fr(proof.lookup_t_poly_opening_at_z));
        transcript.update_transcript(&get_u8arr_from_fr(proof.lookup_selector_poly_opening_at_z));
        transcript.update_transcript(&get_u8arr_from_fr(
            proof.lookup_table_type_poly_opening_at_z,
        ));
        transcript.update_transcript(&get_u8arr_from_fr(proof.lookup_s_poly_opening_at_z_omega));
        transcript.update_transcript(&get_u8arr_from_fr(
            proof.lookup_grand_product_opening_at_z_omega,
        ));
        transcript.update_transcript(&get_u8arr_from_fr(proof.lookup_t_poly_opening_at_z_omega));
        transcript.update_transcript(&get_u8arr_from_fr(proof.linearisation_poly_opening_at_z));

        let vv = transcript.get_transcript_challenge(7);

        // round 5

        transcript.update_transcript(&get_u8arr_from_fq(proof.opening_proof_at_z.x));
        transcript.update_transcript(&get_u8arr_from_fq(proof.opening_proof_at_z.y));
        transcript.update_transcript(&get_u8arr_from_fq(proof.opening_proof_at_z_omega.x));
        transcript.update_transcript(&get_u8arr_from_fq(proof.opening_proof_at_z_omega.y));

        let uu = transcript.get_transcript_challenge(8);

        let power_of_alpha_1 = get_fr_from_u8arr(alphaaa.to_vec());
        let power_of_alpha_2 = power_of_alpha_1.mul(power_of_alpha_1);
        let power_of_alpha_3 = power_of_alpha_2.mul(power_of_alpha_1);
        let power_of_alpha_4 = power_of_alpha_3.mul(power_of_alpha_1);
        let power_of_alpha_5 = power_of_alpha_4.mul(power_of_alpha_1);
        let power_of_alpha_6 = power_of_alpha_5.mul(power_of_alpha_1);
        let power_of_alpha_7 = power_of_alpha_6.mul(power_of_alpha_1);
        let power_of_alpha_8 = power_of_alpha_7.mul(power_of_alpha_1);

        pvs.alpha = power_of_alpha_1;
        pvs.beta = get_fr_from_u8arr(betaa.to_vec());
        pvs.gamma = get_fr_from_u8arr(gammma.to_vec());
        pvs.power_of_alpha_2 = power_of_alpha_2;
        pvs.power_of_alpha_3 = power_of_alpha_3;
        pvs.power_of_alpha_4 = power_of_alpha_4;
        pvs.power_of_alpha_5 = power_of_alpha_5;
        pvs.power_of_alpha_6 = power_of_alpha_6;
        pvs.power_of_alpha_7 = power_of_alpha_7;
        pvs.power_of_alpha_8 = power_of_alpha_8;
        pvs.eta = get_fr_from_u8arr(etaaa.to_vec());
        pvs.beta_lookup = get_fr_from_u8arr(beta_lookuppp.to_vec());
        pvs.gamma_lookup = get_fr_from_u8arr(gamma_lookuppp.to_vec());
        pvs.beta_plus_one = pvs.beta_lookup.add(Fr::from_str("1").unwrap());
        pvs.beta_gamma_plus_gamma = pvs
            .beta_lookup
            .add(Fr::from_str("1").unwrap())
            .mul(pvs.gamma_lookup);
        pvs.v = get_fr_from_u8arr(vv.to_vec());
        pvs.u = get_fr_from_u8arr(uu.to_vec());
        pvs.z = get_fr_from_u8arr(zz.to_vec());
        pvs.z_minus_last_omega = pvs.z.add(get_omega().pow([get_domain_size() - 1]).neg());
        pvs.z_in_domain_size = zz_in_domain_size;
    }

    fn permutation_quotient_contribution(
        pvs: &mut PartialVerifierState,
        l0_at_z: Fp256<FrParameters>,
        proof: Proof,
    ) -> Fp256<FrParameters> {
        let mut res = pvs
            .power_of_alpha_4
            .mul(proof.copy_permutation_grand_product_opening_at_z_omega);
        let mut factor_multiplier;

        factor_multiplier = proof.copy_permutation_polys_0_opening_at_z.mul(pvs.beta);

        factor_multiplier = factor_multiplier.add(pvs.gamma);
        factor_multiplier = factor_multiplier.add(proof.state_poly_0_opening_at_z);

        res = res.mul(factor_multiplier);

        factor_multiplier = proof.copy_permutation_polys_1_opening_at_z.mul(pvs.beta);
        factor_multiplier = factor_multiplier.add(pvs.gamma);
        factor_multiplier = factor_multiplier.add(proof.state_poly_1_opening_at_z);
        res = res.mul(factor_multiplier);

        factor_multiplier = proof.copy_permutation_polys_2_opening_at_z.mul(pvs.beta);
        factor_multiplier = factor_multiplier.add(pvs.gamma);
        factor_multiplier = factor_multiplier.add(proof.state_poly_2_opening_at_z);
        res = res.mul(factor_multiplier);

        res = res.mul(proof.state_poly_3_opening_at_z.add(pvs.gamma));
        res = get_scalar_field().sub(res);
        let mut temp_l0atz = l0_at_z.clone();
        temp_l0atz = temp_l0atz.mul(pvs.power_of_alpha_5);
        res = res.add(temp_l0atz.neg());
        res
    }

    fn lookup_quotient_contribution(
        pvs: &mut PartialVerifierState,
        proof: Proof,
    ) -> Fp256<FrParameters> {
        let betaplusone = pvs.beta_lookup.add(Fr::from_str("1").unwrap());
        let beta_gamma = betaplusone.mul(pvs.gamma_lookup);

        pvs.beta_gamma_plus_gamma = beta_gamma;

        let mut res = proof.lookup_s_poly_opening_at_z_omega.mul(pvs.beta_lookup);
        res = res.add(beta_gamma);
        res = res.mul(proof.lookup_grand_product_opening_at_z_omega);
        res = res.mul(pvs.power_of_alpha_6);

        let mut last_omega = get_omega().pow([get_domain_size() - 1]);
        let z_minus_last_omega = pvs.z.add(last_omega.neg());
        res = res.mul(z_minus_last_omega);

        let intermediate_val = pvs.l_0_at_z.mul(pvs.power_of_alpha_7);
        res = res.add(intermediate_val.neg());

        let beta_gamma_power = beta_gamma.pow([get_domain_size() - 1]);
        let subtrahend = pvs
            .power_of_alpha_8
            .mul(pvs.l_n_minus_one_at_z.mul(beta_gamma_power));
        res = res.add(subtrahend.neg());
        res
    }

    fn verify_quotient_evaluation(pvs: &mut PartialVerifierState, public_input: Fr, proof: Proof) {
        let alpha_2 = pvs.alpha.mul(pvs.alpha);
        let alpha_3 = pvs.power_of_alpha_3;
        let alpha_4 = pvs.power_of_alpha_4;
        let alpha_5 = pvs.power_of_alpha_5;
        let alpha_6 = pvs.power_of_alpha_6;
        let alpha_7 = pvs.power_of_alpha_7;
        let alpha_8 = pvs.power_of_alpha_8;

        let l0atz = Self::evaluate_lagrange_poly_out_of_domain(0, pvs.z);

        let lnminus_1_at_z =
            Self::evaluate_lagrange_poly_out_of_domain(get_domain_size() - 1, pvs.z);

        pvs.l_0_at_z = l0atz;
        pvs.l_n_minus_one_at_z = lnminus_1_at_z;

        let state_t = l0atz.mul(public_input);

        let mut result = state_t.mul(proof.gate_selectors_0_opening_at_z);

        result = result.add(Self::permutation_quotient_contribution(
            pvs,
            l0atz,
            proof.clone(),
        ));

        result = result.add(Self::lookup_quotient_contribution(pvs, proof.clone()));

        result = result.add(proof.linearisation_poly_opening_at_z);

        let vanishing = pvs.z_in_domain_size.add(Fr::from_str("1").unwrap().neg());

        let lhs = proof.quotient_poly_opening_at_z.mul(vanishing);

        assert_eq!(lhs, result);  
    }

    fn evaluate_lagrange_poly_out_of_domain(
        poly_num: u64,
        at: Fp256<FrParameters>,
    ) -> Fp256<FrParameters> {
        let mut omega_power = Fr::from_str("1").unwrap();
        if poly_num > 0 {
            omega_power = get_omega().pow(&[poly_num as u64]);
        }
        let mut res = at
            .pow(&[get_domain_size()])
            .add(get_scalar_field().sub(Fr::from_str("1").unwrap()));
        assert_ne!(res, Fp256::zero());

        res = res.mul(omega_power);

        let mut denominator = at.add(get_scalar_field().sub((Fr::from(omega_power))));
        denominator = denominator.mul(Fr::from(get_domain_size()));

        denominator = denominator.inverse().unwrap();
        res = res.mul(denominator);
        res
    }

    fn prepare_queries(
        vk_gate_setup_0_affine: GroupAffine<Parameters>,
        vk_gate_setup_1_affine: GroupAffine<Parameters>,
        vk_gate_setup_2_affine: GroupAffine<Parameters>,
        vk_gate_setup_3_affine: GroupAffine<Parameters>,
        vk_gate_setup_4_affine: GroupAffine<Parameters>,
        vk_gate_setup_5_affine: GroupAffine<Parameters>,
        vk_gate_setup_6_affine: GroupAffine<Parameters>,
        vk_gate_setup_7_affine: GroupAffine<Parameters>,
        vk_gate_selectors_1_affine: GroupAffine<Parameters>,
        vk_permutation_3_affine: GroupAffine<Parameters>,
        vk_lookp_table_0_affine: GroupAffine<Parameters>,
        vk_lookp_table_1_affine: GroupAffine<Parameters>,
        vk_lookp_table_2_affine: GroupAffine<Parameters>,
        vk_lookp_table_3_affine: GroupAffine<Parameters>,
        pvs: PartialVerifierState,
        proof: Proof,
    ) -> (
        GroupAffine<Parameters>,
        GroupAffine<Parameters>,
        Fr,
        Fr,
        GroupAffine<Parameters>,
        Fr,
    ) {
        let z_domain_size = pvs.z_in_domain_size;

        let mut current_z = z_domain_size;
        let proof_quotient_poly_parts_0_affine = proof.quotient_poly_parts_0;

        let proof_quotient_poly_parts_1_affine = proof.quotient_poly_parts_1;

        let proof_quotient_poly_parts_2_affine = proof.quotient_poly_parts_2;

        let proof_quotient_poly_parts_3_affine = proof.quotient_poly_parts_3;

        let mut queries_at_z_0 = proof_quotient_poly_parts_1_affine
            .mul(z_domain_size)
            .into_affine()
            .add(proof_quotient_poly_parts_0_affine);

        current_z = current_z.mul(z_domain_size);

        queries_at_z_0 = proof_quotient_poly_parts_2_affine
            .mul(current_z)
            .into_affine()
            .add(queries_at_z_0);

        current_z = current_z.mul(z_domain_size);

        queries_at_z_0 = proof_quotient_poly_parts_3_affine
            .mul(current_z)
            .into_affine()
            .add(queries_at_z_0);

        let state_opening_0_z = proof.state_poly_0_opening_at_z;

        let state_opening_1_z = proof.state_poly_1_opening_at_z;

        let state_opening_2_z = proof.state_poly_2_opening_at_z;

        let state_opening_3_z = proof.state_poly_3_opening_at_z;

        let mut queries_at_z_1 = Self::main_gate_linearisation_contribution_with_v(
            vk_gate_setup_0_affine,
            vk_gate_setup_1_affine,
            vk_gate_setup_2_affine,
            vk_gate_setup_3_affine,
            vk_gate_setup_4_affine,
            vk_gate_setup_5_affine,
            vk_gate_setup_6_affine,
            vk_gate_setup_7_affine,
            state_opening_0_z,
            state_opening_1_z,
            state_opening_2_z,
            state_opening_3_z,
            proof.clone(),
            pvs.clone(),
        );

        queries_at_z_1 = Self::add_assign_rescue_customgate_linearisation_contribution_with_v(
            queries_at_z_1,
            state_opening_0_z,
            state_opening_1_z,
            state_opening_2_z,
            state_opening_3_z,
            vk_gate_selectors_1_affine,
            proof.clone(),
            pvs.clone(),
        );

        // PROOF_QUOTIENT_POLY_PARTS_1_X_SLOT currentz QUERIES_AT_Z_0_X_SLOT
        // queries_at_z_1

        let resp = Self::add_assign_permutation_linearisation_contribution_with_v(
            queries_at_z_1,
            state_opening_0_z,
            state_opening_1_z,
            state_opening_2_z,
            state_opening_3_z,
            vk_permutation_3_affine,
            proof.clone(),
            pvs.clone(),
        );

        queries_at_z_1 = resp.0;
        let copy_permutation_first_aggregated_commitment_coeff = resp.1;

        // we are assigning few things here internally which would be required later on
        let (
            lookup_s_first_aggregated_commitment_coeff,
            lookup_grand_product_first_aggregated_commitment_coeff,
        ) = Self::add_assign_lookup_linearisation_contribution_with_v(
            queries_at_z_1,
            state_opening_0_z,
            state_opening_1_z,
            state_opening_2_z,
            proof.clone(),
            pvs.clone(),
        );

        let state_eta = pvs.eta;

        let eta = state_eta;
        let mut currenteta = eta;

        let mut queries_t_poly_aggregated = vk_lookp_table_0_affine;
        queries_t_poly_aggregated = vk_lookp_table_1_affine
            .mul(currenteta)
            .into_affine()
            .add(queries_t_poly_aggregated);

        currenteta = currenteta.mul(eta);
        queries_t_poly_aggregated = vk_lookp_table_2_affine
            .mul(currenteta)
            .into_affine()
            .add(queries_t_poly_aggregated);
        currenteta = currenteta.mul(eta);

        queries_t_poly_aggregated = vk_lookp_table_3_affine
            .mul(currenteta)
            .into_affine()
            .add(queries_t_poly_aggregated);

        (
            queries_at_z_0,
            queries_at_z_1,
            copy_permutation_first_aggregated_commitment_coeff,
            lookup_s_first_aggregated_commitment_coeff,
            queries_t_poly_aggregated,
            lookup_grand_product_first_aggregated_commitment_coeff,
        )
    }

    fn prepare_aggregated_commitment(
        queries: (
            GroupAffine<Parameters>,
            GroupAffine<Parameters>,
            Fr,
            Fr,
            GroupAffine<Parameters>,
            Fr,
        ),
        vk_gate_selectors_0_affine: GroupAffine<Parameters>,
        vk_gate_selectors_1_affine: GroupAffine<Parameters>,
        vk_permutation_0_affine: GroupAffine<Parameters>,
        vk_permutation_1_affine: GroupAffine<Parameters>,
        vk_permutation_2_affine: GroupAffine<Parameters>,
        vk_lookup_selector_affine: GroupAffine<Parameters>,
        vk_lookup_table_type_affine: GroupAffine<Parameters>,
        copy_permutation_first_aggregated_commitment_coeff: Fr,
        lookup_s_first_aggregated_commitment_coeff: Fr,
        queries_t_poly_aggregated: GroupAffine<Parameters>,
        lookup_grand_product_first_aggregated_commitment_coeff: Fr,
        pvs: PartialVerifierState,
        proof: Proof,
    ) -> (GroupAffine<Parameters>, GroupAffine<Parameters>) {
        let queries_z_0 = queries.0;
        let queries_z_1 = queries.1;

        let mut aggregation_challenge = Fr::from_str("1").unwrap();

        let first_d_coeff: Fr;
        let first_t_coeff: Fr;

        let mut aggregated_at_z = queries_z_0;
        let proof_quotient_poly_opening_at_z_slot = proof.quotient_poly_opening_at_z;

        let state_v_slot = pvs.v;

        let proof_linearisation_poly_opening_at_z_slot = proof.linearisation_poly_opening_at_z;

        let proof_state_polys_0 = proof.state_poly_0;

        let proof_state_polys_1 = proof.state_poly_1;

        let proof_state_polys_2 = proof.state_poly_2;

        let state_opening_0_z = proof.state_poly_0_opening_at_z;

        let state_opening_1_z = proof.state_poly_1_opening_at_z;

        let state_opening_2_z = proof.state_poly_2_opening_at_z;

        let state_opening_3_z = proof.state_poly_3_opening_at_z;

        let proof_gate_selectors_0_opening_at_z = proof.gate_selectors_0_opening_at_z;

        let proof_copy_permutation_polys_0_opening_at_z =
            proof.copy_permutation_polys_0_opening_at_z;

        let proof_copy_permutation_polys_1_opening_at_z =
            proof.copy_permutation_polys_1_opening_at_z;

        let proof_copy_permutation_polys_2_opening_at_z =
            proof.copy_permutation_polys_2_opening_at_z;

        let proof_lookup_t_poly_opening_at_z = proof.lookup_t_poly_opening_at_z;

        let proof_lookup_selector_poly_opening_at_z = proof.lookup_selector_poly_opening_at_z;

        let proof_lookup_table_type_poly_opening_at_z = proof.lookup_table_type_poly_opening_at_z;

        let mut aggregated_opening_at_z = proof_quotient_poly_opening_at_z_slot;

        aggregated_at_z = aggregated_at_z.add(queries_z_1);
        aggregation_challenge = aggregation_challenge.mul(state_v_slot);

        aggregated_opening_at_z = aggregated_opening_at_z
            .add(aggregation_challenge.mul(proof_linearisation_poly_opening_at_z_slot));

        fn update_aggregation_challenge(
            queries_commitment_pt: GroupAffine<Parameters>,
            value_at_z: Fr,
            curr_aggregation_challenge: Fr,
            current_agg_opening_at_z: Fr,
            state_v_slot: Fr,
            aggregated_at_z: GroupAffine<Parameters>,
        ) -> (Fr, GroupAffine<Parameters>, Fr) {
            let mut new_agg_challenege = curr_aggregation_challenge.mul(state_v_slot);
            let new_aggregated_at_z = queries_commitment_pt
                .mul(new_agg_challenege)
                .into_affine()
                .add(aggregated_at_z);
            let new_agg_opening_at_z = new_agg_challenege
                .mul(value_at_z)
                .add(current_agg_opening_at_z);
            (
                new_agg_challenege,
                new_aggregated_at_z,
                new_agg_opening_at_z,
            )
        }

        let mut update_agg_challenge = update_aggregation_challenge(
            proof_state_polys_0,
            state_opening_0_z,
            aggregation_challenge,
            aggregated_opening_at_z,
            state_v_slot,
            aggregated_at_z,
        );

        aggregated_at_z = update_agg_challenge.1;

        aggregation_challenge = update_agg_challenge.0;
        aggregated_opening_at_z = update_agg_challenge.2;

        update_agg_challenge = update_aggregation_challenge(
            proof_state_polys_1,
            state_opening_1_z,
            aggregation_challenge,
            aggregated_opening_at_z,
            state_v_slot,
            aggregated_at_z,
        );

        aggregated_at_z = update_agg_challenge.1;
        aggregation_challenge = update_agg_challenge.0;
        aggregated_opening_at_z = update_agg_challenge.2;

        update_agg_challenge = update_aggregation_challenge(
            proof_state_polys_2,
            state_opening_2_z,
            aggregation_challenge,
            aggregated_opening_at_z,
            state_v_slot,
            aggregated_at_z,
        );

        aggregated_at_z = update_agg_challenge.1;
        aggregation_challenge = update_agg_challenge.0;
        aggregated_opening_at_z = update_agg_challenge.2;

        aggregation_challenge = aggregation_challenge.mul(state_v_slot);
        first_d_coeff = aggregation_challenge;

        aggregated_opening_at_z = aggregation_challenge
            .mul(state_opening_3_z)
            .add(aggregated_opening_at_z);

        update_agg_challenge = update_aggregation_challenge(
            vk_gate_selectors_0_affine,
            proof_gate_selectors_0_opening_at_z,
            aggregation_challenge,
            aggregated_opening_at_z,
            state_v_slot,
            aggregated_at_z,
        );

        aggregated_at_z = update_agg_challenge.1;
        aggregation_challenge = update_agg_challenge.0;
        aggregated_opening_at_z = update_agg_challenge.2;

        update_agg_challenge = update_aggregation_challenge(
            vk_permutation_0_affine,
            proof_copy_permutation_polys_0_opening_at_z,
            aggregation_challenge,
            aggregated_opening_at_z,
            state_v_slot,
            aggregated_at_z,
        );

        aggregated_at_z = update_agg_challenge.1;
        aggregation_challenge = update_agg_challenge.0;
        aggregated_opening_at_z = update_agg_challenge.2;

        update_agg_challenge = update_aggregation_challenge(
            vk_permutation_1_affine,
            proof_copy_permutation_polys_1_opening_at_z,
            aggregation_challenge,
            aggregated_opening_at_z,
            state_v_slot,
            aggregated_at_z,
        );

        aggregated_at_z = update_agg_challenge.1;
        aggregation_challenge = update_agg_challenge.0;
        aggregated_opening_at_z = update_agg_challenge.2;

        update_agg_challenge = update_aggregation_challenge(
            vk_permutation_2_affine,
            proof_copy_permutation_polys_2_opening_at_z,
            aggregation_challenge,
            aggregated_opening_at_z,
            state_v_slot,
            aggregated_at_z,
        );

        aggregated_at_z = update_agg_challenge.1;
        aggregation_challenge = update_agg_challenge.0;
        aggregated_opening_at_z = update_agg_challenge.2;

        aggregation_challenge = aggregation_challenge.mul(state_v_slot);
        first_t_coeff = aggregation_challenge;

        aggregated_opening_at_z = aggregation_challenge
            .mul(proof_lookup_t_poly_opening_at_z)
            .add(aggregated_opening_at_z);

        update_agg_challenge = update_aggregation_challenge(
            vk_lookup_selector_affine,
            proof_lookup_selector_poly_opening_at_z,
            aggregation_challenge,
            aggregated_opening_at_z,
            state_v_slot,
            aggregated_at_z,
        );

        aggregated_at_z = update_agg_challenge.1;
        aggregation_challenge = update_agg_challenge.0;
        aggregated_opening_at_z = update_agg_challenge.2;

        update_agg_challenge = update_aggregation_challenge(
            vk_lookup_table_type_affine,
            proof_lookup_table_type_poly_opening_at_z,
            aggregation_challenge,
            aggregated_opening_at_z,
            state_v_slot,
            aggregated_at_z,
        );

        aggregated_at_z = update_agg_challenge.1;
        aggregation_challenge = update_agg_challenge.0;
        aggregated_opening_at_z = update_agg_challenge.2;

        aggregation_challenge = aggregation_challenge.mul(state_v_slot);

        let state_u = pvs.u;

        let copy_permutation_coeff = aggregation_challenge
            .mul(state_u)
            .add(copy_permutation_first_aggregated_commitment_coeff);

        let proof_copy_permutation_grand_product_affine = proof.copy_permutation_grand_product;

        let proof_copy_permutation_grand_product_opening_at_z_omega =
            proof.copy_permutation_grand_product_opening_at_z_omega;

        let mut aggregated_z_omega = proof_copy_permutation_grand_product_affine
            .mul(copy_permutation_coeff)
            .into_affine();

        let mut aggregated_opening_z_omega =
            proof_copy_permutation_grand_product_opening_at_z_omega.mul(aggregation_challenge);

        let proof_state_polys_3 = proof.state_poly_3;

        let proof_state_polys_3_opening_at_z_omega_slot = proof.state_poly_3_opening_at_z_omega;

        fn update_aggregation_challenge_second(
            queries_commitment_pt: GroupAffine<Parameters>,
            value_at_zomega: Fr,
            prev_coeff: Fr,
            curr_aggregation_challenge: Fr,
            current_aggregated_opening_z_omega: Fr,
            state_v_slot: Fr,
            state_u_slot: Fr,
            aggregated_at_z_omega: GroupAffine<Parameters>,
        ) -> (Fr, GroupAffine<Parameters>, Fr) {
            let new_aggregation_challenge = curr_aggregation_challenge.mul(state_v_slot);
            let final_coeff = new_aggregation_challenge.mul(state_u_slot).add(prev_coeff);
            let new_aggregated_at_z_omega = queries_commitment_pt
                .mul(final_coeff)
                .into_affine()
                .add(aggregated_at_z_omega);
            let new_aggregated_opening_at_z_omega = new_aggregation_challenge
                .mul(value_at_zomega)
                .add(current_aggregated_opening_z_omega);
            (
                new_aggregation_challenge,
                new_aggregated_at_z_omega,
                new_aggregated_opening_at_z_omega,
            )
        }

        update_agg_challenge = update_aggregation_challenge_second(
            proof_state_polys_3,
            proof_state_polys_3_opening_at_z_omega_slot,
            first_d_coeff,
            aggregation_challenge,
            aggregated_opening_z_omega,
            state_v_slot,
            state_u,
            aggregated_z_omega,
        );

        aggregated_z_omega = update_agg_challenge.1;
        aggregation_challenge = update_agg_challenge.0;
        aggregated_opening_z_omega = update_agg_challenge.2;

        let proof_lookup_s_poly = proof.lookup_s_poly;

        let proof_lookup_s_poly_opening_at_z_omega = proof.lookup_s_poly_opening_at_z_omega;

        let proof_lookup_grand_product_affine = proof.lookup_grand_product;

        update_agg_challenge = update_aggregation_challenge_second(
            proof_lookup_s_poly,
            proof_lookup_s_poly_opening_at_z_omega,
            lookup_s_first_aggregated_commitment_coeff,
            aggregation_challenge,
            aggregated_opening_z_omega,
            state_v_slot,
            state_u,
            aggregated_z_omega,
        );

        aggregated_z_omega = update_agg_challenge.1;
        aggregation_challenge = update_agg_challenge.0;
        aggregated_opening_z_omega = update_agg_challenge.2;

        let proof_lookup_grand_product_opening_at_z_omega =
            proof.lookup_grand_product_opening_at_z_omega;

        let proof_lookup_t_poly_opening_at_z_omega = proof.lookup_t_poly_opening_at_z_omega;

        update_agg_challenge = update_aggregation_challenge_second(
            proof_lookup_grand_product_affine,
            proof_lookup_grand_product_opening_at_z_omega,
            lookup_grand_product_first_aggregated_commitment_coeff,
            aggregation_challenge,
            aggregated_opening_z_omega,
            state_v_slot,
            state_u,
            aggregated_z_omega,
        );

        aggregated_z_omega = update_agg_challenge.1;
        aggregation_challenge = update_agg_challenge.0;
        aggregated_opening_z_omega = update_agg_challenge.2;

        update_agg_challenge = update_aggregation_challenge_second(
            queries_t_poly_aggregated,
            proof_lookup_t_poly_opening_at_z_omega,
            first_t_coeff,
            aggregation_challenge,
            aggregated_opening_z_omega,
            state_v_slot,
            state_u,
            aggregated_z_omega,
        );

        aggregated_z_omega = update_agg_challenge.1;
        aggregation_challenge = update_agg_challenge.0;
        aggregated_opening_z_omega = update_agg_challenge.2;

        let pairing_pair_with_generator = aggregated_at_z.add(aggregated_z_omega);

        let aggregated_value = aggregated_opening_z_omega
            .mul(state_u)
            .add(aggregated_opening_at_z);

        let mut pairing_buffer_point = G1Projective::new(
            <G1Point as AffineCurve>::BaseField::from_str(
                &BigInt::parse_bytes("1".as_bytes(), 16).unwrap().to_string(),
            )
            .unwrap(),
            <G1Point as AffineCurve>::BaseField::from_str(
                &BigInt::parse_bytes("2".as_bytes(), 16).unwrap().to_string(),
            )
            .unwrap(),
            <G1Projective as ProjectiveCurve>::BaseField::one(),
        )
        .into_affine();
        pairing_buffer_point = pairing_buffer_point.mul(aggregated_value).into_affine();

        // pointMulIntoDest(PAIRING_BUFFER_POINT_X_SLOT, aggregatedValue, PAIRING_BUFFER_POINT_X_SLOT)

        (pairing_pair_with_generator, pairing_buffer_point)
    }

    fn add_assign_lookup_linearisation_contribution_with_v(
        queries_at_z_1: GroupAffine<Parameters>,
        state_opening_0_z: Fr,
        state_opening_1_z: Fr,
        state_opening_2_z: Fr,
        proof: Proof,
        pvs: PartialVerifierState,
    ) -> (Fr, Fr) {
        let state_power_of_alpha_6 = pvs.power_of_alpha_6;
        let state_power_of_alpha_7 = pvs.power_of_alpha_7;
        let state_power_of_alpha_8 = pvs.power_of_alpha_8;
        let state_l_n_minus_1_at_z = pvs.l_n_minus_one_at_z;
        let state_z_minus_last_omega = pvs.z_minus_last_omega;
        let state_v_slot = pvs.v;
        let proof_lookup_t_poly_opening_at_z_omega = proof.lookup_t_poly_opening_at_z_omega;
        let proof_lookup_t_poly_opening_at_z = proof.lookup_t_poly_opening_at_z;
        let state_beta_lookup = pvs.beta_lookup;
        let state_beta_gamma_plus_gamma = pvs.beta_gamma_plus_gamma;
        let state_eta = pvs.eta;
        let proof_looup_table_type_poly_opening_at_z = proof.lookup_table_type_poly_opening_at_z;
        let proof_lookup_selector_poly_opening_at_z = proof.lookup_selector_poly_opening_at_z;
        let state_gamma_lookup = pvs.gamma_lookup;
        let state_beta_plus_one = pvs.beta_plus_one;
        let proof_lookup_grand_product_opening_at_z_omega =
            proof.lookup_grand_product_opening_at_z_omega;
        let state_l_0_at_z = pvs.l_0_at_z;
        // check is this assignment even correct ??
        let mut factor = proof_lookup_grand_product_opening_at_z_omega;
        factor = factor.mul(state_power_of_alpha_6);
        factor = factor.mul(state_z_minus_last_omega);
        factor = factor.mul(state_v_slot);

        // saving factor into
        let lookup_s_first_aggregated_commitment_coeff = factor;

        factor = proof_lookup_t_poly_opening_at_z_omega;
        factor = factor.mul(state_beta_lookup);
        factor = factor.add(proof_lookup_t_poly_opening_at_z);
        factor = factor.add(state_beta_gamma_plus_gamma);

        let mut freconstructed = state_opening_0_z;
        let eta = state_eta;
        let mut currenteta = eta;

        freconstructed = currenteta.mul(state_opening_1_z).add(freconstructed);
        currenteta = currenteta.mul(eta);
        freconstructed = currenteta.mul(state_opening_2_z).add(freconstructed);
        currenteta = currenteta.mul(eta);

        freconstructed =
            freconstructed.add(proof_looup_table_type_poly_opening_at_z.mul(currenteta));
        freconstructed = freconstructed.mul(proof_lookup_selector_poly_opening_at_z);
        freconstructed = freconstructed.add(state_gamma_lookup);

        factor = factor.mul(freconstructed);
        factor = factor.mul(state_beta_plus_one);
        factor = -factor;
        factor = factor.mul(state_power_of_alpha_6);
        factor = factor.mul(state_z_minus_last_omega);

        factor = factor.add(state_l_0_at_z.mul(state_power_of_alpha_7));
        factor = factor.add(state_l_n_minus_1_at_z.mul(state_power_of_alpha_8));

        factor = factor.mul(state_v_slot);

        (lookup_s_first_aggregated_commitment_coeff, factor)
        // LOOKUP_GRAND_PRODUCT_FIRST_AGGREGATED_COMMITMENT_COEFF

        // factor // need to store it in somewhere
    }

    fn add_assign_permutation_linearisation_contribution_with_v(
        queries_at_z_1: GroupAffine<Parameters>,
        state_opening_0_z: Fr,
        state_opening_1_z: Fr,
        state_opening_2_z: Fr,
        state_opening_3_z: Fr,
        vk_permutation_3_affine: GroupAffine<Parameters>,
        proof: Proof,
        pvs: PartialVerifierState,
    ) -> (GroupAffine<Parameters>, Fr) {
        let state_power_of_alpha_4 = pvs.power_of_alpha_4;
        let state_power_of_alpha_5 = pvs.power_of_alpha_5;
        // z and beta are challeneges
        let state_z_slot = pvs.z;
        let state_beta = pvs.beta;
        let state_gamma = pvs.gamma;
        let state_v_slot = pvs.v;

        // this is part of proof
        let proof_copy_permutation_grand_product_opening_at_z_omega =
            proof.copy_permutation_grand_product_opening_at_z_omega;

        let proof_copy_permutation_polys_0_opening_at_z =
            proof.copy_permutation_polys_0_opening_at_z;

        let proof_copy_permutation_polys_1_opening_at_z =
            proof.copy_permutation_polys_1_opening_at_z;

        let proof_copy_permutation_polys_2_opening_at_z =
            proof.copy_permutation_polys_2_opening_at_z;

        let non_residue_0 = Fr::from_str("5").unwrap();
        let non_residue_1 = Fr::from_str("7").unwrap();
        let non_residue_2 = Fr::from_str("10").unwrap();

        let mut factor = state_power_of_alpha_4;

        let zmulbeta = state_z_slot.mul(state_beta);

        let mut intermediate_value = state_opening_0_z.add(zmulbeta.add(state_gamma));
        factor = factor.mul(intermediate_value);

        intermediate_value = (zmulbeta.mul(non_residue_0))
            .add(state_gamma)
            .add(state_opening_1_z);
        factor = factor.mul(intermediate_value);

        intermediate_value = (zmulbeta.mul(non_residue_1))
            .add(state_gamma)
            .add(state_opening_2_z);
        factor = factor.mul(intermediate_value);

        intermediate_value = (zmulbeta.mul(non_residue_2))
            .add(state_gamma)
            .add(state_opening_3_z);
        factor = factor.mul(intermediate_value);

        // calcualated somewhere in the middle
        let state_l_0_at_z = pvs.l_0_at_z;

        factor = factor.add(state_l_0_at_z.mul(state_power_of_alpha_5));
        factor = factor.mul(state_v_slot);
        // skipping storing factor for now or else we need to store it into this
        let copy_permutation_first_aggregated_commitment_coeff = factor;

        factor = state_power_of_alpha_4.mul(state_beta);

        factor = factor.mul(proof_copy_permutation_grand_product_opening_at_z_omega);

        intermediate_value = state_opening_0_z
            .add(state_gamma.add(proof_copy_permutation_polys_0_opening_at_z.mul(state_beta)));
        factor = factor.mul(intermediate_value);

        intermediate_value = state_opening_1_z
            .add(state_gamma.add(proof_copy_permutation_polys_1_opening_at_z.mul(state_beta)));
        factor = factor.mul(intermediate_value);

        intermediate_value = state_opening_2_z
            .add(state_gamma.add(proof_copy_permutation_polys_2_opening_at_z.mul(state_beta)));
        factor = factor.mul(intermediate_value);

        factor = factor.mul(state_v_slot);

        let temp_query_val = vk_permutation_3_affine.mul(factor).into_affine();
        (
            queries_at_z_1.add(-temp_query_val),
            copy_permutation_first_aggregated_commitment_coeff,
        )
    }

    fn add_assign_rescue_customgate_linearisation_contribution_with_v(
        queries_at_z_1: GroupAffine<Parameters>,
        state_opening_0_z: Fr,
        state_opening_1_z: Fr,
        state_opening_2_z: Fr,
        state_opening_3_z: Fr,
        vk_gate_selectors_1_affine: GroupAffine<Parameters>,
        proof: Proof,
        pvs: PartialVerifierState,
    ) -> GroupAffine<Parameters> {
        // challenges wire later
        let state_alpha_slot = pvs.alpha;
        let state_power_of_alpha_2 = pvs.power_of_alpha_2;
        let state_power_of_alpha_3 = pvs.power_of_alpha_3;
        let state_v_slot = pvs.v;

        let mut accumulator: Fr;
        let mut intermediate_value: Fr;

        accumulator = state_opening_0_z.square();
        accumulator = accumulator.sub(state_opening_1_z);
        accumulator = accumulator.mul(state_alpha_slot);

        intermediate_value = state_opening_1_z.square();
        intermediate_value = intermediate_value.sub(state_opening_2_z);
        intermediate_value = intermediate_value.mul(state_power_of_alpha_2);
        accumulator = accumulator.add(intermediate_value);

        intermediate_value = state_opening_2_z.mul(state_opening_0_z);
        intermediate_value = intermediate_value.sub(state_opening_3_z);
        intermediate_value = intermediate_value.mul(state_power_of_alpha_3);
        accumulator = accumulator.add(intermediate_value);

        accumulator = accumulator.mul(state_v_slot);

        vk_gate_selectors_1_affine
            .mul(accumulator)
            .into_affine()
            .add(queries_at_z_1)
    }

    fn main_gate_linearisation_contribution_with_v(
        vk_gate_setup_0_affine: GroupAffine<Parameters>,
        vk_gate_setup_1_affine: GroupAffine<Parameters>,
        vk_gate_setup_2_affine: GroupAffine<Parameters>,
        vk_gate_setup_3_affine: GroupAffine<Parameters>,
        vk_gate_setup_4_affine: GroupAffine<Parameters>,
        vk_gate_setup_5_affine: GroupAffine<Parameters>,
        vk_gate_setup_6_affine: GroupAffine<Parameters>,
        vk_gate_setup_7_affine: GroupAffine<Parameters>,
        state_opening_0_z: Fr,
        state_opening_1_z: Fr,
        state_opening_2_z: Fr,
        state_opening_3_z: Fr,
        proof: Proof,
        pvs: PartialVerifierState,
    ) -> GroupAffine<Parameters> {
        let mut queries_at_z_1 = vk_gate_setup_0_affine.mul(state_opening_0_z).into_affine();
        queries_at_z_1 =
            queries_at_z_1.add(vk_gate_setup_1_affine.mul(state_opening_1_z).into_affine());
        queries_at_z_1 =
            queries_at_z_1.add(vk_gate_setup_2_affine.mul(state_opening_2_z).into_affine());
        queries_at_z_1 =
            queries_at_z_1.add(vk_gate_setup_3_affine.mul(state_opening_3_z).into_affine());
        queries_at_z_1 = queries_at_z_1.add(
            vk_gate_setup_4_affine
                .mul(state_opening_0_z.mul(state_opening_1_z))
                .into_affine(),
        );
        queries_at_z_1 = queries_at_z_1.add(
            vk_gate_setup_5_affine
                .mul(state_opening_0_z.mul(state_opening_2_z))
                .into_affine(),
        );
        queries_at_z_1 = queries_at_z_1.add(vk_gate_setup_6_affine);

        // proof value
        let proof_state_polys_3_opening_at_z_omega_slot = proof.state_poly_3_opening_at_z_omega;

        // proof value
        let proof_gate_selectors_0_opening_at_z = proof.gate_selectors_0_opening_at_z;

        // challenge
        let state_v_slot = pvs.v;

        queries_at_z_1 = queries_at_z_1.add(
            vk_gate_setup_7_affine
                .mul(proof_state_polys_3_opening_at_z_omega_slot)
                .into_affine(),
        );

        let coeff = proof_gate_selectors_0_opening_at_z.mul(state_v_slot);
        queries_at_z_1 = queries_at_z_1.mul(coeff).into_affine();

        queries_at_z_1
    }

    // TODO: need to verify this method
    fn final_pairing(
        state_u_slot: Fr,
        state_z_slot: Fr,
        mut pairing_pair_generator: GroupAffine<Parameters>,
        mut pairing_buffer_point: GroupAffine<Parameters>,
        proof: Proof,
    ) -> bool {
        pairing_pair_generator = pairing_pair_generator.add(-pairing_buffer_point);

        let z_omega = state_z_slot.mul(get_omega());

        let proof_opening_proof_at_z = proof.opening_proof_at_z;
        let proof_opening_proof_at_z_omega = proof.opening_proof_at_z_omega;
        pairing_pair_generator = (proof_opening_proof_at_z.mul(state_z_slot))
            .into_affine()
            .add(pairing_pair_generator);
        pairing_pair_generator = (proof_opening_proof_at_z_omega.mul(z_omega.mul(state_u_slot)))
            .into_affine()
            .add(pairing_pair_generator);

        let mut pairing_pair_with_x = proof.opening_proof_at_z;

        pairing_pair_with_x = proof_opening_proof_at_z_omega
            .mul(state_u_slot)
            .into_affine()
            .add(pairing_pair_with_x);
        pairing_pair_with_x = -pairing_pair_with_x;

        let (g2_0_element, g2_1_element) = get_g2_elements();

        let pairing1 = Bn254::pairing(pairing_pair_generator, g2_0_element);
        let pairing2 = Bn254::pairing(pairing_pair_with_x, g2_1_element);

        if pairing1 == pairing2 {
            return true;
        }

        return false;
    }

    // TODO: remove the hardcoded proof
    pub fn verify(&self, public_input: String, proof_strings: Vec<String>) -> bool {
        let proof = parse_proof(proof_strings);
        let verification_key = get_verification_key();
        let public_inputs = get_public_inputs(public_input);

        // start verification
        let mut pvs = PartialVerifierState::new();
        Self::compute_challenges(&mut pvs, public_inputs, proof.clone());
        Self::verify_quotient_evaluation(&mut pvs, public_inputs, proof.clone());

        // prepare vk
        let vk_gate_setup_0_affine = verification_key.gate_setup[0];
        let vk_gate_setup_1_affine = verification_key.gate_setup[1];
        let vk_gate_setup_2_affine = verification_key.gate_setup[2];
        let vk_gate_setup_3_affine = verification_key.gate_setup[3];
        let vk_gate_setup_4_affine = verification_key.gate_setup[4];
        let vk_gate_setup_5_affine = verification_key.gate_setup[5];
        let vk_gate_setup_6_affine = verification_key.gate_setup[6];
        let vk_gate_setup_7_affine = verification_key.gate_setup[7];
        let vk_gate_selectors_0_affine = verification_key.gate_selectors[0];
        let vk_gate_selectors_1_affine = verification_key.gate_selectors[1];
        let vk_permutation_0_affine = verification_key.permutation[0];
        let vk_permutation_1_affine = verification_key.permutation[1];
        let vk_permutation_2_affine = verification_key.permutation[2];
        let vk_permutation_3_affine = verification_key.permutation[3];
        let vk_lookp_table_0_affine = verification_key.lookup_table[0];
        let vk_lookp_table_1_affine = verification_key.lookup_table[1];
        let vk_lookp_table_2_affine = verification_key.lookup_table[2];
        let vk_lookp_table_3_affine = verification_key.lookup_table[3];
        let vk_lookup_selector_affine = verification_key.lookup_selector;
        let vk_lookup_table_type_affine = verification_key.lookup_table_type;

        let queries = Self::prepare_queries(
            vk_gate_setup_0_affine,
            vk_gate_setup_1_affine,
            vk_gate_setup_2_affine,
            vk_gate_setup_3_affine,
            vk_gate_setup_4_affine,
            vk_gate_setup_5_affine,
            vk_gate_setup_6_affine,
            vk_gate_setup_7_affine,
            vk_gate_selectors_1_affine,
            vk_permutation_3_affine,
            vk_lookp_table_0_affine,
            vk_lookp_table_1_affine,
            vk_lookp_table_2_affine,
            vk_lookp_table_3_affine,
            pvs.clone(),
            proof.clone(),
        );

        let lookup_s_first_aggregated_commitment_coeff = queries.3;

        let (pairing_pair_with_generator, pairing_pair_buffer_point) =
            Self::prepare_aggregated_commitment(
                queries,
                vk_gate_selectors_0_affine,
                vk_gate_selectors_1_affine,
                vk_permutation_0_affine,
                vk_permutation_1_affine,
                vk_permutation_2_affine,
                vk_lookup_selector_affine,
                vk_lookup_table_type_affine,
                queries.2,
                lookup_s_first_aggregated_commitment_coeff,
                queries.4,
                queries.5,
                pvs.clone(),
                proof.clone(),
            );

        let is_proof_verified = Self::final_pairing(
            pvs.u,
            pvs.z,
            pairing_pair_with_generator,
            pairing_pair_buffer_point,
            proof,
        );

        println!("Is Proof Verified: {:?}", is_proof_verified);

        is_proof_verified
    }
}
