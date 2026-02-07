use anchor_lang::prelude::*;
use anchor_spl::token::{mint_to, MintTo, Token};
use solana_program::keccak;
use crate::{state::*, utils::crypto::*, errors::BridgeError};

/// Event emitted when a mint is executed
#[event]
pub struct MintExecuted {
    pub order_id: [u8; 32],
    pub amount: u64,
    pub recipient: [u8; 32],
    pub epoch: u64,
}

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
        space = 8 + ExecutedMessage::INIT_SPACE,
        seeds = [b"exec", bridge_msg.order_id.as_ref()],
        bump
    )]
    pub executed: Account<'info, ExecutedMessage>,

    /// CHECK: SPL Token mint account; verified via CPI
    #[account(mut)]
    pub mint: AccountInfo<'info>,

    /// CHECK: Recipient SPL token account; verified via CPI
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
    let cfg = &mut ctx.accounts.config;

    // Ensure bridge is active
    require!(!cfg.paused, BridgeError::Paused);

    // Prevent expired messages (10 minutes window)
    let now = Clock::get()?.unix_timestamp;
    require!(now - bridge_msg.timestamp < 600, BridgeError::Expired);

    // Prevent double execution (replay protection)
    require!(!ctx.accounts.executed.executed, BridgeError::Replay);

    // Compute Keccak256 hash of the bridge message
    let hash = keccak::hashv(&[&bridge_msg.try_to_vec()?]).to_bytes();

    // Validate signatures
    let mut valid_signatures: u8 = 0;
    for sig in signatures.iter() {
        let addr = recover_address(&hash, sig)?;
        if ctx.accounts.validator_set.validators.contains(&addr) {
            valid_signatures = valid_signatures.saturating_add(1);
        }
    }

    require!(
        valid_signatures >= cfg.validator_threshold,
        BridgeError::ThresholdNotMet
    );

    // Mark message as executed to prevent replay
    ctx.accounts.executed.executed = true;

    // Mint tokens via CPI using PDA as authority
    let seeds: &[&[u8]] = &[b"config", &[cfg.bump]];
    let signer = &[&seeds[..]];

    mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.mint.clone(),
                to: ctx.accounts.recipient.clone(),
                authority: cfg.to_account_info(),
            },
            signer,
        ),
        bridge_msg.amount,
    )?;

    // Update total minted safely
    cfg.total_minted = cfg
        .total_minted
        .checked_add(bridge_msg.amount)
        .ok_or(BridgeError::Overflow)?;

    // Emit event for off-chain listeners
    emit!(MintExecuted {
        order_id: bridge_msg.order_id,
        amount: bridge_msg.amount,
        recipient: bridge_msg.recipient,
        epoch: ctx.accounts.validator_set.epoch,
    });

    Ok(())
}
