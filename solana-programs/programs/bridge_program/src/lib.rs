use anchor_lang::prelude::*;

pub mod instructions;
pub mod state;
pub mod errors;
pub mod utils;

use instructions::*;
use crate::state::BridgeMessage;

declare_id!("4n1ZgVeynaczVMmdSW5d4s5NNRBbF4rojRwNCq4bbyWJ");

#[program]
pub mod bridge_program {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, args: InitializeArgs) -> Result<()> {
        instructions::initialize::initialize_handler(ctx, args)
    }

    pub fn execute_mint(
        ctx: Context<ExecuteMint>,
        msg: BridgeMessage,
        signatures: Vec<[u8; 65]>
    ) -> Result<()> {
        instructions::execute_mint::execute_mint_handler(ctx, msg, signatures)
    }

    pub fn initiate_burn(
        ctx: Context<InitiateBurn>,
        amount: u64,
        evm_recipient: [u8; 20]
    ) -> Result<()> {
        instructions::initiate_burn::initiate_burn_handler(ctx, amount, evm_recipient)
    }

    pub fn confirm_unlock(
        ctx: Context<ConfirmUnlock>,
        msg: BridgeMessage,
        signatures: Vec<[u8; 65]>
    ) -> Result<()> {
        instructions::confirm_unlock::confirm_unlock_handler(ctx, msg, signatures)
    }

    pub fn update_validators(
        ctx: Context<UpdateValidators>,
        new_validators: Vec<[u8; 20]>,
        new_threshold: u8
    ) -> Result<()> {
        instructions::update_validators::update_validators_handler(ctx, new_validators, new_threshold)
    }
}
