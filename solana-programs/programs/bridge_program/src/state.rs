use anchor_lang::prelude::*;

/// Maximum validators supported
pub const MAX_VALIDATORS: usize = 10;

#[account]
pub struct BridgeConfig {
    pub admin: Pubkey,
    pub paused: bool,
    pub validator_threshold: u8,
    pub validator_count: u8,
    pub total_minted: u64,
    pub total_burned: u64,
    pub bump: u8,
}

#[account]
pub struct ValidatorSet {
    /// Fixed-size array of validator addresses (EVM 20-byte addresses)
    pub validators: [[u8; 20]; MAX_VALIDATORS],

    /// Number of active validators
    pub count: u8,

    /// Epoch/version of this validator set
    pub epoch: u64,
}

#[account]
pub struct ExecutedMessage {
    pub executed: bool,
}


/// Burn Order Account

/// Represents a pending burn operation to unlock SCAI on EVM
#[account]
pub struct BurnOrder {
    /// Amount to burn
    pub amount: u64,

    /// Recipient on EVM chain (20-byte address)
    pub evm_recipient: [u8; 20],

    /// Whether the burn has been executed and confirmed
    pub executed: bool,
}


/// Bridge Message Struct

/// Canonical cross-chain message format (EVM → Solana)
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct BridgeMessage {
    /// Chain ID of source chain (EVM L2)
    pub source_chain_id: u64,

    /// Chain ID of destination chain (Solana = 1)
    pub destination_chain_id: u64,

    /// Unique order ID for this message (32 bytes)
    pub order_id: [u8; 32],

    /// Amount of SCAI tokens to bridge
    pub amount: u64,

    /// Sender address on source chain (20 bytes)
    pub sender: [u8; 20],

    /// Recipient Solana pubkey (32 bytes)
    pub recipient: [u8; 32],

    /// Nonce for replay protection
    pub nonce: u64,

    /// Timestamp (unix epoch)
    pub timestamp: i64,
}
