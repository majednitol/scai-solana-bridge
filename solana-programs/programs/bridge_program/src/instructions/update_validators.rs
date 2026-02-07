use anchor_lang::prelude::*;
use crate::state::*;

/// Accounts required to update validators
#[derive(Accounts)]
pub struct UpdateValidators<'info> {
    #[account(mut, has_one = admin)]
    pub config: Account<'info, BridgeConfig>,
    
    #[account(mut)]
    pub validator_set: Account<'info, ValidatorSet>,

    pub admin: Signer<'info>,
}

pub fn update_validators_handler(
    ctx: Context<UpdateValidators>,
    new_validators: Vec<[u8; 20]>,
    new_threshold: u8,
) -> Result<()> {
    let vs = &mut ctx.accounts.validator_set;
    for i in 0..vs.validators.len() {
        if i < new_validators.len() {
            vs.validators[i] = new_validators[i];
        } else {
            vs.validators[i] = [0u8; 20]; 
        }
    }

    // Increment epoch
    vs.epoch = vs.epoch.saturating_add(1);

    // Update config
    let cfg = &mut ctx.accounts.config;
    cfg.validator_threshold = new_threshold;
    cfg.validator_count = new_validators.len() as u8;

    Ok(())
}
