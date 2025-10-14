#![allow(unexpected_cfgs)]
#![allow(deprecated)]

use anchor_lang::prelude::*;

mod instructions;
mod state;

use instructions::*;

use spl_discriminator::SplDiscriminate;
use spl_tlv_account_resolution::state::ExtraAccountMetaList;
use spl_transfer_hook_interface::instruction::{
    ExecuteInstruction, InitializeExtraAccountMetaListInstruction,
};

declare_id!("DhzyDgCmmQzVC4vEcj2zRGUyN8Mt5JynfdGLKkBcRGaX");

#[program]
pub mod whitelist_transfer_hook {
    use super::*;

    pub fn create_token(ctx: Context<TokenFactory>) -> Result<()> {
        ctx.accounts.init_mint()
    }

    pub fn transfer_token(ctx: Context<TransferToken>, amount: u64) -> Result<()> {
        ctx.accounts.transfer_token(amount)
    }

    pub fn add_to_whitelist(ctx: Context<WhitelistOperations>) -> Result<()> {
        ctx.accounts.add_to_whitelist()
    }

    pub fn remove_from_whitelist(ctx: Context<WhitelistOperations>) -> Result<()> {
        ctx.accounts.remove_from_whitelist()
    }

    #[instruction(discriminator = ExecuteInstruction::SPL_DISCRIMINATOR_SLICE)]
    pub fn transfer_hook(ctx: Context<TransferHook>, amount: u64) -> Result<()> {
        // Call the transfer hook logic
        ctx.accounts.transfer_hook(amount)
    }
}
