use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount, mint_to};
use crate::{state::*, errors::*, utils::crypto::*};

#[derive(Accounts)]
pub struct ExecuteMint<'info> {
    #[account(mut)]
    pub config: Account<'info, BridgeConfig>,

    #[account(mut)]
    pub validator_set: Account<'info, ValidatorSet>,

    #[account(init, payer = payer, space = 8 + 1, seeds = [b"exec", msg.order_id.as_ref()], bump)]
    pub executed: Account<'info, ExecutedMessage>,

    #[account(mut)]
    pub mint: Account<'info, Mint>,

    #[account(mut)]
    pub recipient: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<ExecuteMint>,
    msg: BridgeMessage,
    signatures: Vec<[u8; 65]>
) -> Result<()> {
    require!(!ctx.accounts.config.paused, BridgeError::Paused);

    let now = Clock::get()?.unix_timestamp;
    require!(now - msg.timestamp < 600, BridgeError::Expired);

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

    mint_to(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            anchor_spl::token::MintTo {
                mint: ctx.accounts.mint.to_account_info(),
                to: ctx.accounts.recipient.to_account_info(),
                authority: ctx.accounts.config.to_account_info(),
            },
        ),
        msg.amount,
    )?;

    ctx.accounts.config.total_minted += msg.amount;

    Ok(())
}
