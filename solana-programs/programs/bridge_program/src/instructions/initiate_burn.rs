use anchor_lang::prelude::*;
use anchor_spl::token::{burn, Token, TokenAccount};
use crate::state::*;

#[derive(Accounts)]
pub struct InitiateBurn<'info> {
    #[account(mut)]
    pub config: Account<'info, BridgeConfig>,

    #[account(mut)]
    pub user_token: Account<'info, TokenAccount>,

    #[account(init, payer = user, space = 8 + 64)]
    pub burn_order: Account<'info, BurnOrder>,

    pub token_program: Program<'info, Token>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<InitiateBurn>,
    amount: u64,
    evm_recipient: [u8; 20],
) -> Result<()> {
    burn(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            anchor_spl::token::Burn {
                mint: ctx.accounts.user_token.mint.to_account_info(),
                from: ctx.accounts.user_token.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            },
        ),
        amount,
    )?;

    let order = &mut ctx.accounts.burn_order;
    order.amount = amount;
    order.evm_recipient = evm_recipient;
    order.executed = false;

    ctx.accounts.config.total_burned += amount;

    Ok(())
}
