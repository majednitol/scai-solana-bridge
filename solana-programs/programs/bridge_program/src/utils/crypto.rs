use anchor_lang::prelude::*;
use solana_program::{
    keccak::hash,
    secp256k1_recover::secp256k1_recover,
};
use crate::errors::BridgeError;

pub fn recover_address(hash_bytes: &[u8; 32], sig: &[u8; 65]) -> Result<[u8; 20]> {
    require!(sig.len() == 65, BridgeError::InvalidSignatures);

    // Split signature: 64 bytes (r,s) + 1 byte recovery ID (v)
    let (signature, recovery_id_slice) = sig.split_at(64);
    let mut recovery_id = recovery_id_slice[0];

    // Normalize recovery ID
    if recovery_id >= 27 {
        recovery_id -= 27;
    }

    // Ensure recovery_id is 0 or 1 (Ethereum-style)
    require!(recovery_id <= 1, BridgeError::InvalidSignatures);

    // Recover uncompressed public key using Solana native syscall
    let pubkey = secp256k1_recover(hash_bytes, recovery_id, signature)
        .map_err(|_| error!(BridgeError::InvalidSignatures))?;

    // Take Keccak256 hash of the recovered public key
    let pubkey_hash = hash(&pubkey.to_bytes());

    // Last 20 bytes of Keccak hash is the Ethereum-style address
    let mut address = [0u8; 20];
    address.copy_from_slice(&pubkey_hash.to_bytes()[12..32]);

    Ok(address)
}

pub fn keccak_hash(data: &[u8]) -> [u8; 32] {
    hash(data).to_bytes()
}
