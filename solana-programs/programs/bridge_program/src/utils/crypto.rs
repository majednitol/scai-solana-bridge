use anchor_lang::prelude::*;
use solana_program::keccak::{hashv, Hash};
use k256::ecdsa::{Signature, VerifyingKey};
use crate::errors::*;

pub fn recover_address(_hash: &[u8; 32], sig: &[u8; 65]) -> Result<[u8; 20]> {
    let _recovery_id = sig[64];
    let sig_bytes: [u8; 64] = sig[0..64]
        .try_into()
        .map_err(|_| error!(BridgeError::InvalidSignatures))?;

    let signature = Signature::from_bytes(&sig_bytes.into())
        .map_err(|_| error!(BridgeError::InvalidSignatures))?;

    let pubkey = VerifyingKey::from_sec1_bytes(&signature.to_bytes())
        .map_err(|_| error!(BridgeError::InvalidSignatures))?;
    let encoded = pubkey.to_encoded_point(false);
    let pubkey_bytes = encoded.as_bytes();

    let addr_hash = keccak_hash(&pubkey_bytes[1..]);
    Ok(addr_hash[12..32].try_into().unwrap())
}
pub fn keccak_hash(data: &[u8]) -> [u8; 32] {
    let h: Hash = hashv(&[data]);
    h.to_bytes()
}
