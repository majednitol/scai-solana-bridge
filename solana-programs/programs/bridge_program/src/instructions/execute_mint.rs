use anchor_lang::prelude::*;
use anchor_spl::token::{mint_to, Token, MintTo};
use solana_program::keccak; 
use crate::{state::*, utils::crypto::*, errors::BridgeError};

#[derive(Accounts)]
#[instruction(bridge_msg: BridgeMessage)]
pub struct ExecuteMint<'info> {
    #[account(mut)]
    pub config: Account<'info, BridgeConfig>,

    #[account(mut)]
    pub validator_set: Account<'info, ValidatorSet>,

    #[account(
        init,
        payer = payer,
        space = 8 + ExecutedMessage::INIT_SPACE, // discriminator + bool
        seeds = [b"exec", bridge_msg.order_id.as_ref()],
        bump
    )]
    pub executed: Account<'info, ExecutedMessage>,
    #[account(mut)]
    pub mint: AccountInfo<'info>,
    #[account(mut)]
    pub recipient: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn execute_mint_handler(
    ctx: Context<ExecuteMint>,
    bridge_msg: BridgeMessage,
    signatures: Vec<[u8; 65]>,
) -> Result<()> {
    // Check paused
    require!(!ctx.accounts.config.paused, BridgeError::Paused);

    let now = Clock::get()?.unix_timestamp;
    require!(now - bridge_msg.timestamp < 600, BridgeError::Expired);

    let hash = keccak::hashv(&[&bridge_msg.try_to_vec()?]).to_bytes();

    let mut valid: u8 = 0;
    for sig in signatures.iter() {
        let addr = recover_address(&hash, sig)?;
        if ctx.accounts.validator_set.validators.contains(&addr) {
            valid = valid.saturating_add(1);
        }
    }
    require!(
        valid >= ctx.accounts.config.validator_threshold,
        BridgeError::ThresholdNotMet
    );
    let seeds: &[&[u8]] = &[b"config", &[ctx.accounts.config.bump]];
    let signer = &[&seeds[..]];

    mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.mint.clone(),
                to: ctx.accounts.recipient.clone(),
                authority: ctx.accounts.config.to_account_info(),
            },
            signer,
        ),
        bridge_msg.amount,
    )?;

    ctx.accounts.config.total_minted = ctx
        .accounts
        .config
        .total_minted
        .checked_add(bridge_msg.amount)
        .ok_or(BridgeError::Overflow)?;

    Ok(())
}
