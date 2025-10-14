#![allow(deprecated)] // for no warnings
#[allow(unexpected_cfgs)]
pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;
mod tests;
use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("2ekwA9pchasuAwe2tX5J1NUGkXAMkgzZsENwwv9pYEcM");

#[program]
pub mod vault {

    use super::*;

    pub fn init_vault(ctx: Context<InitVault>) -> Result<()> {
        ctx.accounts.init_vault(&ctx.bumps)
    }

    pub fn mint_token(ctx: Context<MintToken>) -> Result<()> {
        ctx.accounts.mint_token()
    }

    pub fn deposit(ctx: Context<Trade>, amount: u64) -> Result<()> {
        ctx.accounts.deposit(amount)
    }

    pub fn withdraw(ctx: Context<Trade>, amount: u64) -> Result<()> {
        ctx.accounts.withdraw(amount)
    }
}
