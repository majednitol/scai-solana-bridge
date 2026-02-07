use anchor_lang::prelude::*;
use crate::{state::*, errors::*, utils::crypto::*};

#[derive(Accounts)]
pub struct ConfirmUnlock<'info> {
    #[account(mut)]
    pub config: Account<'info, BridgeConfig>,

    #[account(mut)]
    pub burn_order: Account<'info, BurnOrder>,

    #[account(mut)]
    pub validator_set: Account<'info, ValidatorSet>,
}

pub fn handler(
    ctx: Context<ConfirmUnlock>,
    msg: BridgeMessage,
    signatures: Vec<[u8; 65]>
) -> Result<()> {
    require!(!ctx.accounts.burn_order.executed, BridgeError::Replay);

    let hash = anchor_lang::solana_program::keccak::hashv(&[
        &msg.try_to_vec()?
    ]).0;

    let mut valid = 0u8;

    for sig in signatures {
        let addr = recover_address(&hash, &sig)?;
        if ctx.accounts.validator_set.validators.contains(&addr) {
            valid += 1;
        }
    }

    require!(
        valid >= ctx.accounts.config.validator_threshold,
        BridgeError::ThresholdNotMet
    );

    ctx.accounts.burn_order.executed = true;
    Ok(())
}
