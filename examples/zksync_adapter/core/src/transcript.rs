
use ark_bn254::{FrParameters, Fr};
use ark_ff::Fp256;
use num_bigint::*;

use std::str::FromStr;
use tiny_keccak::{Hasher, Keccak};
use std::ops::{Add, Mul, Neg, Sub};

use crate::utils::{padd_bytes32, padd_bytes3};
pub struct Transcript {
    state_0: [u8; 32], // bytes32 in Solidity is equivalent to an array of 32 bytes in Rust
    state_1: [u8; 32], // Similarly, bytes32 translates to [u8; 32] in Rust
    challenge_counter: u32, // uint32 in Solidity is equivalent to u32 in Rust
    // TODO: make below values are constants
    FR_MASK: Fp256<FrParameters>,
    DST_0: u32,
    DST_1: u32,
    DST_CHALLENGE: u32,
}

impl Transcript {
    pub fn new_transcript() -> Self {
        Transcript {
            state_0: [0; 32],     // Initializes state_0 with 32 bytes of zeros
            state_1: [0; 32],     // Initializes state_1 with 32 bytes of zeros
            challenge_counter: 0, // Initializes challenge_counter to 0
            FR_MASK: Fr::from_str(
                "14474011154664524427946373126085988481658748083205070504932198000989141204991",
            )
            .unwrap(),
            DST_0: 0,
            DST_1: 1,
            DST_CHALLENGE: 2,
        }
    }

    pub fn update_transcript(&mut self, value: &[u8]) {
        // Assuming TRANSCRIPT_BEGIN_SLOT is an initial part of the transcript
        // and it's somehow represented or stored. For this example, let's just use
        // a vector to simulate the whole transcript for simplicity.
        let mut transcript = Keccak::v256();

        // Simulate DST_0 and DST_1 as part of the transcript. In a real scenario,
        // these would be properly defined and included as per your protocol's design.
        let dst_0: u8 = 0;
        let dst_1: u8 = 1;

        // Update the transcript with DST_0 and the value, then hash it for new_state_0
        let val_beg = padd_bytes3(0u8.to_be_bytes().to_vec());
        let val_dst = (dst_0.to_be_bytes().to_vec());
        let val_s0 = padd_bytes32(self.state_0.to_vec());
        let val_s1 = padd_bytes32(self.state_1.to_vec());
        let val_chall = padd_bytes32(value.to_vec());

        let mut concatenated = Vec::new();
        concatenated.extend_from_slice(&val_beg);
        concatenated.extend_from_slice(&val_dst);
        concatenated.extend_from_slice(&val_s0);
        concatenated.extend_from_slice(&val_s1);
        concatenated.extend_from_slice(&val_chall);
        transcript.update(&concatenated);
        let mut out = [0u8; 32];
        transcript.finalize(&mut out);
        let new_state_0 = out;

        let new_state_0_val = BigInt::from_bytes_be(Sign::Plus, &new_state_0);

        transcript = Keccak::v256();

        let val_beg = padd_bytes3(0u8.to_be_bytes().to_vec());
        let val_dst1 = (dst_1.to_be_bytes().to_vec());
        let val_s0 = padd_bytes32(self.state_0.to_vec());
        let val_s1 = padd_bytes32(self.state_1.to_vec());
        let val_chall = padd_bytes32(value.to_vec());

        let mut concatenated = Vec::new();
        concatenated.extend_from_slice(&val_beg);
        concatenated.extend_from_slice(&val_dst1);
        concatenated.extend_from_slice(&val_s0);
        concatenated.extend_from_slice(&val_s1);
        concatenated.extend_from_slice(&val_chall);
        transcript.update(&concatenated);

        let mut out = [0u8; 32];
        transcript.finalize(&mut out);
        let new_state_1 = out;
        let new_state_1_val = BigInt::from_bytes_be(Sign::Plus, &out);

        self.state_0.copy_from_slice(&new_state_0);
        self.state_1.copy_from_slice(&new_state_1);
    }

    pub fn get_transcript_challenge(&mut self, number_of_challenge: u32) -> [u8; 32] {
        let mut transcript = Keccak::v256();

        let val_beg = padd_bytes3(0u8.to_be_bytes().to_vec());
        let val_dst2 = 2u8.to_be_bytes().to_vec();
        let val_s0 = self.state_0.to_vec();
        let val_s1 = self.state_1.to_vec();
        // chall
        let temp_chall = BigInt::from(number_of_challenge).mul(BigInt::from(2).pow(224));
        let mut val_chall = padd_bytes32(temp_chall.to_bytes_be().1);
        let final_val_chall = &val_chall[0..4];

        // println!("final_val_chall: {:?}", final_val_chall);

        let mut concatenated = Vec::new();
        concatenated.extend_from_slice(&val_beg);
        concatenated.extend_from_slice(&val_dst2);
        concatenated.extend_from_slice(&val_s0);
        concatenated.extend_from_slice(&val_s1);
        concatenated.extend_from_slice(&final_val_chall);
        transcript.update(&concatenated);

        let mut out = [0u8; 32];
        transcript.finalize(&mut out);
        let res = BigInt::from_bytes_be(Sign::Plus, &out);

        const FR_MASK: [u8; 32] = [
            0x1f, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff,
        ];
        let mut res_fr = [0u8; 32];

        for i in 0..32 {
            res_fr[i] = out[i] & FR_MASK[i];
        }
        res_fr
    }
}