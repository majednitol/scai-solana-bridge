use anchor_lang::prelude::*;

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
    pub validators: [[u8; 20]; MAX_VALIDATORS],
    pub count: u8,
    pub epoch: u64,
}

#[account]
pub struct ExecutedMessage {
    pub executed: bool,
}

impl ExecutedMessage {
    pub const INIT_SPACE: usize = 1;
}

#[account]
pub struct BurnOrder {
    pub amount: u64,
    pub evm_recipient: [u8; 20],
    pub executed: bool,
}

impl BurnOrder {
    pub const INIT_SPACE: usize = 8 + 20 + 1;
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct BridgeMessage {
    pub source_chain_id: u64,
    pub destination_chain_id: u64,
    pub order_id: [u8; 32],
    pub amount: u64,
    pub sender: [u8; 20],
    pub recipient: [u8; 32],
    pub nonce: u64,
    pub timestamp: i64,
}
