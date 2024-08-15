use crate::utils::hasher::Sha256;
use ethabi::{decode, encode, ParamType, Token};
use serde::{Deserialize, Serialize};

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

impl AccountState {
    pub fn zero() -> Self {
        Self {
            state_root: [0; 32],
            statement: StatementDigest::zero(),
            start_nexus_hash: [0; 32],
            last_proof_height: 0,
            height: 0,
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        let tokens = vec![
            self.statement.encode(),
            Token::FixedBytes(self.state_root.to_vec()),
            Token::FixedBytes(self.start_nexus_hash.to_vec()),
            Token::Uint(self.last_proof_height.into()),
            Token::Uint(self.height.into()),
        ];
        encode(&tokens)
    }

    pub fn decode(encoded: &[u8]) -> Result<Self, ethabi::Error> {
        let tokens = decode(
            &[
                ParamType::FixedBytes(32),
                ParamType::FixedBytes(32),
                ParamType::FixedBytes(32),
                ParamType::Uint(32),
                ParamType::Uint(32),
            ],
            encoded,
        )?;

        if tokens.len() != 5 {
            return Err(ethabi::Error::InvalidData);
        }

        let statement = StatementDigest::decode(&tokens[0])?;
        let state_root: [u8; 32] = tokens[1]
            .clone()
            .into_fixed_bytes()
            .ok_or(ethabi::Error::InvalidData)?
            .try_into()
            .map_err(|_| ethabi::Error::InvalidData)?;
        let start_nexus_hash: [u8; 32] = tokens[2]
            .clone()
            .into_fixed_bytes()
            .ok_or(ethabi::Error::InvalidData)?
            .try_into()
            .map_err(|_| ethabi::Error::InvalidData)?;
        let last_proof_height = tokens[3]
            .clone()
            .into_uint()
            .ok_or(ethabi::Error::InvalidData)?
            .as_u32();
        let height = tokens[4]
            .clone()
            .into_uint()
            .ok_or(ethabi::Error::InvalidData)?
            .as_u32();

        Ok(AccountState {
            statement,
            state_root,
            start_nexus_hash,
            last_proof_height,
            height,
        })
    }
}

impl StatementDigest {
    pub fn encode(&self) -> Token {
        let mut bytes = vec![];
        for &num in &self.0 {
            bytes.extend(&num.to_be_bytes());
        }
        Token::FixedBytes(bytes)
    }

    pub fn decode(token: &Token) -> Result<Self, ethabi::Error> {
        if let Token::FixedBytes(bytes) = token {
            if bytes.len() != 32 {
                return Err(ethabi::Error::InvalidData);
            }

            let mut u32_array = [0u32; 8];
            for (i, chunk) in bytes.chunks(4).enumerate() {
                u32_array[i] =
                    u32::from_be_bytes(chunk.try_into().map_err(|_| ethabi::Error::InvalidData)?);
            }

            Ok(StatementDigest(u32_array))
        } else {
            Err(ethabi::Error::InvalidData)
        }
    }
}

impl StatementDigest {
    fn zero() -> Self {
        Self([0u32; 8])
    }
}

// impl From<RiscZeroDigest> for StatementDigest {
//     fn from(item: RiscZeroDigest) -> Self {
//         let words = item.as_words();
//         let mut new_digest = [0u32; 8];

//         for (i, &element) in words.iter().take(8).enumerate() {
//             new_digest[i] = element;
//         }

//         Self(new_digest)
//     }
// }
