use anchor_lang::prelude::*;
use solana_program::secp256k1_recover::secp256k1_recover;

pub fn recover_address(hash: &[u8; 32], sig: &[u8; 65]) -> Result<[u8; 20]> {
    let recovery_id = sig[64];
    let signature = &sig[0..64];

    let pubkey = secp256k1_recover(hash, recovery_id, signature)
        .map_err(|_| error!(crate::errors::BridgeError::InvalidSignatures))?;

    Ok(keccak_hash(&pubkey)[12..32].try_into().unwrap())
}

fn keccak_hash(data: &[u8]) -> [u8; 32] {
    use solana_program::keccak::hash;
    hash(data).0
}
