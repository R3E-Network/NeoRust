use neo_crypto::hash::HashableForVec;
use primitive_types::H256;

pub fn test_hash256(message: &[u8]) -> H256 {
    let hash = message.hash256();
    H256::from_slice(&hash)
} 