use anchor_lang::prelude::*;
use crate::{state::*, errors::BridgeError};

/// Event emitted when validators are updated
#[event]
pub struct ValidatorsUpdated {
    pub epoch: u64,
    pub new_threshold: u8,
    pub validator_count: u8,
}

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
    let cfg = &mut ctx.accounts.config;
    let vs = &mut ctx.accounts.validator_set;

    // Pause check: optional but recommended
    require!(!cfg.paused, BridgeError::Paused);

    // Ensure threshold is valid
    require!(
        new_threshold > 0 && new_threshold <= new_validators.len() as u8,
        BridgeError::ThresholdNotMet
    );

    // Update validators, zero-fill remaining slots
    for i in 0..vs.validators.len() {
        if i < new_validators.len() {
            vs.validators[i] = new_validators[i];
        } else {
            vs.validators[i] = [0u8; 20]; 
        }
    }

    // Update validator count and threshold
    vs.count = new_validators.len() as u8;
    cfg.validator_count = vs.count;
    cfg.validator_threshold = new_threshold;

    // Increment epoch for tracking changes
    vs.epoch = vs.epoch.saturating_add(1);

    // Emit event for off-chain relayers / monitoring
    emit!(ValidatorsUpdated {
        epoch: vs.epoch,
        new_threshold,
        validator_count: vs.count,
    });

    Ok(())
}
