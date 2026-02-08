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







// use anchor_lang::prelude::*;
// use crate::state::*;
// use crate::errors::BridgeError;

// /// Event emitted when the bridge is initialized
// #[event]
// pub struct BridgeInitialized {
//     pub admin: Pubkey,
//     pub validator_threshold: u8,
//     pub validator_count: u8,
//     pub initialized_at: i64,
// }

// #[derive(AnchorSerialize, AnchorDeserialize)]
// pub struct InitializeArgs {
//     pub validators: Vec<[u8; 20]>, // EVM-style validator addresses
//     pub threshold: u8,             // Minimum signatures required
// }

// #[derive(Accounts)]
// pub struct Initialize<'info> {
//     /// Bridge configuration account (PDA)
//     #[account(
//         init,
//         payer = payer,
//         space = BridgeConfig::LEN,
//         seeds = [b"config"],
//         bump
//     )]
//     pub config: Account<'info, BridgeConfig>,

//     /// Validator set account (PDA)
//     #[account(
//         init,
//         payer = payer,
//         space = ValidatorSet::LEN,
//         seeds = [b"validators"],
//         bump
//     )]
//     pub validator_set: Account<'info, ValidatorSet>,

//     /// Payer for account creation
//     #[account(mut)]
//     pub payer: Signer<'info>,

//     pub system_program: Program<'info, System>,
// }

// pub fn initialize_handler(ctx: Context<Initialize>, args: InitializeArgs) -> Result<()> {
//     // 1️⃣ Validate inputs
//     require!(!args.validators.is_empty(), BridgeError::InvalidSignatures);
//     require!(
//         args.threshold > 0 && args.threshold <= args.validators.len() as u8,
//         BridgeError::ThresholdNotMet
//     );
//     require!(args.validators.len() <= MAX_VALIDATORS, BridgeError::Overflow);

//     // 2️⃣ Initialize bridge config
//     let cfg = &mut ctx.accounts.config;
//     cfg.admin = ctx.accounts.payer.key();
//     cfg.validator_threshold = args.threshold;
//     cfg.validator_count = args.validators.len() as u8;
//     cfg.paused = false;
//     cfg.total_minted = 0;
//     cfg.total_burned = 0;

//     // 3️⃣ Initialize validator set
//     let vs = &mut ctx.accounts.validator_set;
//     for i in 0..MAX_VALIDATORS {
//         vs.validators[i] = if i < args.validators.len() {
//             args.validators[i]
//         } else {
//             [0u8; 20] // zero-fill remaining slots
//         };
//     }
//     vs.count = args.validators.len() as u8;
//     vs.epoch = 1;

//     // 4️⃣ Emit event for relayers / monitoring
//     emit!(BridgeInitialized {
//         admin: cfg.admin,
//         validator_threshold: cfg.validator_threshold,
//         validator_count: cfg.validator_count,
//         initialized_at: Clock::get()?.unix_timestamp,
//     });

//     Ok(())
// }
