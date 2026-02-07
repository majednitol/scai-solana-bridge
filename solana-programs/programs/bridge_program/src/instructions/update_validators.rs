use anchor_lang::prelude::*;
use crate::state::*;

#[derive(Accounts)]
pub struct UpdateValidators<'info> {
    #[account(mut, has_one = admin)]
    pub config: Account<'info, BridgeConfig>,
    #[account(mut)]
    pub validator_set: Account<'info, ValidatorSet>,
    pub admin: Signer<'info>,
}

pub fn handler(
    ctx: Context<UpdateValidators>,
    new_validators: Vec<[u8; 20]>,
    new_threshold: u8,
) -> Result<()> {
    let vs = &mut ctx.accounts.validator_set;
    vs.validators = new_validators;
    vs.epoch += 1;

    let cfg = &mut ctx.accounts.config;
    cfg.validator_threshold = new_threshold;
    cfg.validator_count = vs.validators.len() as u8;

    Ok(())
}
