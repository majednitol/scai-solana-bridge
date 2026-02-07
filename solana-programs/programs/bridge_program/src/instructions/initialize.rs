use anchor_lang::prelude::*;
use crate::state::*;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct InitializeArgs {
    pub validators: Vec<[u8; 20]>,
    pub threshold: u8,
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = payer, space = 8 + 256)]
    pub config: Account<'info, BridgeConfig>,

    #[account(init, payer = payer, space = 8 + 512)]
    pub validator_set: Account<'info, ValidatorSet>,

    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

pub fn initialize_handler(ctx: Context<Initialize>, args: InitializeArgs) -> Result<()> {
    let cfg = &mut ctx.accounts.config;
    cfg.admin = ctx.accounts.payer.key();
    cfg.validator_threshold = args.threshold;
    cfg.validator_count = args.validators.len() as u8;
    cfg.paused = false;
    cfg.total_minted = 0;
    cfg.total_burned = 0;
    cfg.bump = 0;


    let vs = &mut ctx.accounts.validator_set;
    for i in 0..MAX_VALIDATORS {
        vs.validators[i] = if i < args.validators.len() {
            args.validators[i]
        } else {
            [0u8; 20]
        };
    }

    vs.count = args.validators.len() as u8;
    vs.epoch = 1;

    Ok(())
}
