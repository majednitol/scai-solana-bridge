use anchor_lang::prelude::*;
use crate::{state::*, errors::*, utils::crypto::{keccak_hash, recover_address}};

#[derive(Accounts)]
pub struct ConfirmUnlock<'info> {
    #[account(mut)]
    pub config: Account<'info, BridgeConfig>,

    #[account(mut)]
    pub burn_order: Account<'info, BurnOrder>,

    #[account(mut)]
    pub validator_set: Account<'info, ValidatorSet>,
}

pub fn confirm_unlock_handler(
    ctx: Context<ConfirmUnlock>,
    msg: BridgeMessage,
    signatures: Vec<[u8; 65]>
) -> Result<()> {
    // Prevent replay attacks
    require!(!ctx.accounts.burn_order.executed, BridgeError::Replay);

    // Compute hash of the bridge message
    let hash = keccak_hash(&msg.try_to_vec()?);

    let mut valid = 0u8;

    // Verify validator signatures
    for sig in signatures {
        let addr = recover_address(&hash, &sig)?;
        if ctx.accounts.validator_set.validators.contains(&addr) {
            valid = valid.saturating_add(1);
        }
    }

    require!(
        valid >= ctx.accounts.config.validator_threshold,
        BridgeError::ThresholdNotMet
    );

    // Mark burn order as executed
    ctx.accounts.burn_order.executed = true;

    Ok(())
}
