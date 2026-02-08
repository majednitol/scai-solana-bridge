
use anchor_lang::prelude::*;
use anchor_spl::token::{self, MintTo, Token};
use crate::{state::*, errors::BridgeError, utils::crypto::{keccak_hash, recover_address}};

/// Event emitted when a mint is executed successfully
#[event]
pub struct MintExecuted {
    pub order_id: [u8; 32],
    pub amount: u64,
    pub recipient: [u8; 32],
    pub executed_at: i64,
}

#[derive(Accounts)]
#[instruction(bridge_msg: BridgeMessage)]
pub struct ExecuteMint<'info> {
    #[account(mut, seeds = [b"config"], bump = config.bump)]
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

    /// CHECK: SPL Token mint (PDA must be the mint authority)
    #[account(mut)]
    pub mint: AccountInfo<'info>,

    /// CHECK: Recipient's SPL token account, verified via CPI call to token program
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
    let executed = &mut ctx.accounts.executed;
    let validator_set = &ctx.accounts.validator_set;

    //  Replay & Expiry Checks
    let now = Clock::get()?.unix_timestamp;
    require!(now - bridge_msg.timestamp < 600, BridgeError::Expired);
    require!(!executed.executed, BridgeError::Replay);

    // Hash the bridge message
    let serialized_msg = bridge_msg.try_to_vec()?;
    let hash = keccak_hash(&serialized_msg);

    // Validator Signature Verification
    let mut valid_count: u8 = 0;
    let mut seen_validators: Vec<[u8; 20]> = vec![];

    for sig in signatures.iter() {
        let addr = recover_address(&hash, sig)?;
        if validator_set.validators.iter().any(|&v| v == addr) && !seen_validators.contains(&addr) {
            valid_count = valid_count.saturating_add(1);
            seen_validators.push(addr);
        }
    }

    // Ensure threshold is met
    require!(valid_count >= cfg.validator_threshold, BridgeError::ThresholdNotMet);

    //  Mark as executed to prevent replay
    executed.executed = true;

    //  Mint tokens via CPI
    let seeds = &[b"config".as_ref(), &[cfg.bump]];
    let signer = &[&seeds[..]];

    token::mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.mint.to_account_info(),
                to: ctx.accounts.recipient.to_account_info(),
                authority: cfg.to_account_info(),
            },
            signer,
        ),
        bridge_msg.amount,
    )?;

    //  Update total minted safely
    cfg.total_minted = cfg
        .total_minted
        .checked_add(bridge_msg.amount)
        .ok_or(BridgeError::Overflow)?;

    //  Emit event for off-chain relayers
    emit!(MintExecuted {
        order_id: bridge_msg.order_id,
        amount: bridge_msg.amount,
        recipient: bridge_msg.recipient,
        executed_at: now,
    });

    Ok(())
}
