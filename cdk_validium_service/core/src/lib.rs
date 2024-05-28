use adapter_sdk::traits::Proof;
use adapter_sdk::types::RollupPublicInputs;
use nexus_core::types::H256;
use serde::{Deserialize, Serialize, Deserializer};
use base64::{decode, encode};

fn decode_base64<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    decode(&s).map_err(serde::de::Error::custom)
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CdkProof {
    last_verified_batch: u64,
	new_verified_batch: u64,
    #[serde(deserialize_with = "decode_base64")]
	new_state_root: Vec<u8>,
    #[serde(deserialize_with = "decode_base64")]
	new_local_exit_root: Vec<u8>,
    #[serde(deserialize_with = "decode_base64")]
	proof: Vec<u8>
}

impl Proof for CdkProof {
    fn verify(
        &self,
        vk: &[u8; 32],
        public_inputs: &RollupPublicInputs,
    ) -> Result<(), anyhow::Error> {
        Ok(())
    }
}
