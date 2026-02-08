use anchor_lang::prelude::*;
use crate::state::*;
use crate::errors::BridgeError;

/// Event emitted when validators are updated
#[event]
pub struct ValidatorsUpdated {
    pub epoch: u64,
    pub new_threshold: u8,
    pub validator_count: u8,
    pub updated_at: i64, // timestamp for relayer tracking
}

/// Accounts required to update validators
#[derive(Accounts)]
pub struct UpdateValidators<'info> {
    /// Bridge configuration account
    #[account(mut, has_one = admin)]
    pub config: Account<'info, BridgeConfig>,

    /// Validator set account
    #[account(mut)]
    pub validator_set: Account<'info, ValidatorSet>,

    /// Admin signer
    pub admin: Signer<'info>,
}

pub fn update_validators_handler(
    ctx: Context<UpdateValidators>,
    new_validators: Vec<[u8; 20]>,
    new_threshold: u8,
) -> Result<()> {
    let cfg = &mut ctx.accounts.config;
    let vs = &mut ctx.accounts.validator_set;

    // 1️⃣ Pause check
    require!(!cfg.paused, BridgeError::Paused);

    // 2️⃣ Validate new threshold
    require!(
        new_threshold > 0 && new_threshold <= new_validators.len() as u8,
        BridgeError::ThresholdNotMet
    );

    // 3️⃣ Update validator set
    for i in 0..vs.validators.len() {
        vs.validators[i] = if i < new_validators.len() {
            new_validators[i]
        } else {
            [0u8; 20] // zero-fill remaining slots
        };
    }

    // 4️⃣ Update metadata
    vs.count = new_validators.len() as u8;
    cfg.validator_count = vs.count;
    cfg.validator_threshold = new_threshold;

    // 5️⃣ Increment epoch
    vs.epoch = vs.epoch.saturating_add(1);

    // 6️⃣ Emit event for off-chain relayers
    emit!(ValidatorsUpdated {
        epoch: vs.epoch,
        new_threshold,
        validator_count: vs.count,
        updated_at: Clock::get()?.unix_timestamp,
    });

    Ok(())
}
