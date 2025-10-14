#![allow(unexpected_cfgs)]
#![allow(deprecated)]

use anchor_lang::prelude::*;

pub mod error;
mod instructions;
mod state;
mod tests;

use error::EscrowError;
use instructions::*;

declare_id!("FircrADQ2wgGuvpm8qneNCfKM7o5zoHTWnDQxngpTQ3J");

#[program]
pub mod anchor_escrow {
    use super::*;

    pub fn make(ctx: Context<Make>, seed: u64, deposit: u64, receive: u64) -> Result<()> {
        ctx.accounts.init_escrow(seed, receive, &ctx.bumps)?;
        ctx.accounts.deposit(deposit)
    }

    pub fn refund(ctx: Context<Refund>) -> Result<()> {
        ctx.accounts.refund_and_close_vault()
    }

    pub fn take(ctx: Context<Take>) -> Result<()> {
        ctx.accounts.deposit()?;
        ctx.accounts.withdraw_and_close_vault()
    }

    pub fn make_with_interval(
        ctx: Context<MakeInterval>,
        seed: u64,
        deposit: u64,
        interval: u64,
        receive: u64,
    ) -> Result<()> {
        ctx.accounts
            .init_escrow_with_interval(seed, receive, interval, &ctx.bumps)?;
        ctx.accounts.deposit(deposit)
    }

    pub fn refund_with_interval(ctx: Context<RefundInterval>) -> Result<()> {
        require!(
            ctx.accounts.escrow.interval <= (Clock::get()?.unix_timestamp as u64),
            EscrowError::NeedToWait
        );
        ctx.accounts.refund_and_close_vault()
    }

    pub fn take_with_interval(ctx: Context<TakeInterval>) -> Result<()> {
        require!(
            ctx.accounts.escrow.interval <= (Clock::get()?.unix_timestamp as u64),
            EscrowError::NeedToWait
        );
        ctx.accounts.deposit()?;
        ctx.accounts.withdraw_and_close_vault()
    }
}
