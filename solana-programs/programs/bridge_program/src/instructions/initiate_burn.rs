use anchor_lang::prelude::*;
use anchor_spl::token::{burn, Burn, Token};
use crate::state::*;

#[derive(Accounts)]
pub struct InitiateBurn<'info> {
    #[account(mut)]
    pub config: Account<'info, BridgeConfig>,

    #[account(mut)]
    pub user_token: AccountInfo<'info>,
    #[account(mut)]
    pub mint: AccountInfo<'info>,

    #[account(init, payer = user, space = 8 + 8 + 20 + 1)]
    pub burn_order: Account<'info, BurnOrder>,

    pub token_program: Program<'info, Token>,

    #[account(mut)]
    pub user: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn initiate_burn_handler(
    ctx: Context<InitiateBurn>,
    amount: u64,
    evm_recipient: [u8; 20],
) -> Result<()> {
    burn(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Burn {
                mint: ctx.accounts.mint.clone(),
                from: ctx.accounts.user_token.clone(),
                authority: ctx.accounts.user.to_account_info(),
            },
        ),
        amount,
    )?;

    let order = &mut ctx.accounts.burn_order;
    order.amount = amount;
    order.evm_recipient = evm_recipient;
    order.executed = false;

    ctx.accounts.config.total_burned = ctx.accounts
        .config
        .total_burned
        .checked_add(amount)
        .ok_or(ProgramError::Custom(0))?;

    Ok(())
}
