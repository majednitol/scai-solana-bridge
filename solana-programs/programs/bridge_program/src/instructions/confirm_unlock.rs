use anchor_lang::prelude::*;
use crate::{state::*, errors::BridgeError, utils::crypto::{keccak_hash, recover_address}};

/// Event emitted when a burn order is successfully confirmed/unlocked
#[event]
pub struct BurnConfirmed {
    pub order_id: [u8; 32],
    pub amount: u64,
    pub evm_recipient: [u8; 20],
    pub confirmed_at: i64,
}

#[derive(Accounts)]
pub struct ConfirmUnlock<'info> {
    /// Bridge configuration account
    #[account(mut)]
    pub config: Account<'info, BridgeConfig>,

    /// Burn order account to unlock
    #[account(mut)]
    pub burn_order: Account<'info, BurnOrder>,

    /// Validator set account
    #[account(mut)]
    pub validator_set: Account<'info, ValidatorSet>,
}

pub fn confirm_unlock_handler(
    ctx: Context<ConfirmUnlock>,
    bridge_msg: BridgeMessage,
    signatures: Vec<[u8; 65]>,
) -> Result<()> {
    let cfg = &ctx.accounts.config;
    let burn_order = &mut ctx.accounts.burn_order;
    let validator_set = &ctx.accounts.validator_set;

    // 1️⃣ Prevent replay attacks
    require!(!burn_order.executed, BridgeError::Replay);

    // 2️⃣ Ensure the bridge message is recent (prevent expired messages)
    let now = Clock::get()?.unix_timestamp;
    require!(
        now - bridge_msg.timestamp < 600, // 10 minutes expiration
        BridgeError::Expired
    );

    // 3️⃣ Hash the bridge message
    let hash = keccak_hash(&bridge_msg.try_to_vec()?);

    // 4️⃣ Verify signatures from validators
    let mut valid_count: u8 = 0;
    let mut seen_validators: Vec<[u8; 20]> = vec![];

    for sig in signatures.iter() {
        let addr = recover_address(&hash, sig)?;
        if validator_set.validators.iter().any(|&v| v == addr) && !seen_validators.contains(&addr) {
            valid_count = valid_count.saturating_add(1);
            seen_validators.push(addr);
        }
    }

    // 5️⃣ Ensure threshold is met
    require!(
        valid_count >= cfg.validator_threshold,
        BridgeError::ThresholdNotMet
    );

    // 6️⃣ Mark burn order as executed
    burn_order.executed = true;

    // 7️⃣ Emit an event for off-chain relayers
    emit!(BurnConfirmed {
        order_id: burn_order.key().to_bytes(),
        amount: burn_order.amount,
        evm_recipient: burn_order.evm_recipient,
        confirmed_at: now,
    });

    Ok(())
}
