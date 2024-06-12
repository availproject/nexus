use solabi::{
  encode::{Encode as SolabiEncode, Encoder, Size},
  decode::{Decode as SolabiDecode, DecodeError, Decoder},
};
use risc0_zkvm::sha::rust_crypto::{Digest as RiscZeroDigestTrait, Sha256};
use risc0_zkvm::sha::Digest as RiscZeroDigest;
use sparse_merkle_tree::traits::{Hasher, Value};
use serde::{Deserialize, Serialize};
use sparse_merkle_tree::H256;

#[derive(Default)]
pub struct ShaHasher(pub Sha256);

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct StatementDigest(pub [u32; 8]);

//TODO: Need to check PartialEq to Eq difference, to ensure there is not security vulnerability.
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct AccountState {
    pub statement: StatementDigest,
    pub state_root: [u8; 32],
    pub start_nexus_hash: [u8; 32],
    pub last_proof_height: u32,
    pub height: u32,
}

impl SolabiEncode for AccountState {
  fn size(&self) -> Size {
      (&self.statement.0, &self.state_root, &self.start_nexus_hash, self.last_proof_height, self.height).size()
  }

  fn encode(&self, encoder: &mut Encoder) {
      (&self.statement.0, &self.state_root, &self.start_nexus_hash, self.last_proof_height, self.height).encode(encoder);
  }
}

impl SolabiDecode for AccountState {
  fn is_dynamic() -> bool {
      true // AccountState contains dynamic fields
  }

  fn decode(decoder: &mut Decoder) -> Result<Self, DecodeError> {
      let (statement, state_root, start_nexus_hash, last_proof_height, height) = SolabiDecode::decode(decoder)?;
      
      Ok(AccountState {
          statement: StatementDigest(statement),
          state_root,
          start_nexus_hash,
          last_proof_height,
          height,
      })
  }
}

impl Value for AccountState {
  fn to_h256(&self) -> H256 {
      if self.statement == StatementDigest::zero() {
          return H256::zero();
      }

      let mut hasher = ShaHasher::new();
      
      let serialized = solabi::encode(self);
      hasher.0.update(&serialized);

      hasher.finish()
  }

  fn zero() -> Self {
      Self {
          state_root: [0; 32],
          statement: StatementDigest::zero(),
          start_nexus_hash: [0; 32],
          last_proof_height: 0,
          height: 0,
      }
  }
}

impl StatementDigest {
  fn zero() -> Self {
      Self([0u32; 8])
  }
}

impl From<RiscZeroDigest> for StatementDigest {
  fn from(item: RiscZeroDigest) -> Self {
      let words = item.as_words();
      let mut new_digest = [0u32; 8];

      for (i, &element) in words.iter().take(8).enumerate() {
          new_digest[i] = element;
      }

      Self(new_digest)
  }
}
