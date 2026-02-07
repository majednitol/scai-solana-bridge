use anchor_lang::prelude::*;
use anchor_spl::token::{burn, Burn, Token};
use crate::state::*;
use crate::errors::BridgeError;

/// Event emitted when a burn is initiated
#[event]
pub struct BurnInitiated {
    pub order_id: [u8; 32],
    pub amount: u64,
    pub evm_recipient: [u8; 20],
}

#[derive(Accounts)]
pub struct InitiateBurn<'info> {
    #[account(mut)]
    pub config: Account<'info, BridgeConfig>,

    /// CHECK: User's SPL token account; authority verified via CPI call to token program
    #[account(mut)]
    pub user_token: AccountInfo<'info>,

    /// CHECK: SPL Token mint account; verified via CPI call
    #[account(mut)]
    pub mint: AccountInfo<'info>,

    #[account(
        init,
        payer = user,
        space = 8 + BurnOrder::INIT_SPACE,
        seeds = [b"burn", user.key().as_ref(), &[Clock::get()?.unix_timestamp as u8]],
        bump
    )]
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
    let cfg = &mut ctx.accounts.config;
    let order = &mut ctx.accounts.burn_order;

    // Ensure amount is non-zero
    require!(amount > 0, BridgeError::InvalidSignatures);

    // Burn tokens via CPI
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

    // Initialize burn order
    order.amount = amount;
    order.evm_recipient = evm_recipient;
    order.executed = false;

    // Update total burned safely
    cfg.total_burned = cfg
        .total_burned
        .checked_add(amount)
        .ok_or(BridgeError::Overflow)?;

    // Emit event for off-chain relayers
    emit!(BurnInitiated {
        order_id: ctx.accounts.burn_order.key().to_bytes(),
        amount,
        evm_recipient,
    });

    Ok(())
}
