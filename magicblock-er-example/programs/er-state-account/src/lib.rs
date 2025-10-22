#![allow(unexpected_cfgs)]
#![allow(deprecated)]

use anchor_lang::prelude::*;
use ephemeral_rollups_sdk::anchor::ephemeral;

mod instructions;
mod state;

use instructions::*;

declare_id!("55Uk5mWb5u86pZCyThbsAQ9w2fWvSEz2f2WshueQoB6t");

#[ephemeral]
#[program]
pub mod er_state_account {

    use super::*;

    pub fn initialize(ctx: Context<InitUser>) -> Result<()> {
        ctx.accounts.initialize(&ctx.bumps)?;
        Ok(())
    }

    pub fn update(ctx: Context<UpdateUser>, new_data: u64) -> Result<()> {
        ctx.accounts.update(new_data)?;
        Ok(())
    }

    pub fn update_with_vrf(ctx: Context<UpdateUserVrf>, seeds: u8) -> Result<()> {
        ctx.accounts.update_user_vrf(seeds)?;
        Ok(())
    }

    pub fn update_with_vrf_on_er(ctx: Context<UpdateUserVrf>, seeds: u8) -> Result<()> {
        ctx.accounts.update_user_vrf_on_er(seeds)?;
        Ok(())
    }

    pub fn callback_update_user_account(
        ctx: Context<CallbackUpdateUserAccount>,
        randomness: [u8; 32],
    ) -> Result<()> {
        ctx.accounts.callback_update_user_account(randomness)?;

        Ok(())
    }

    pub fn callback_update_user_account_on_er(
        ctx: Context<CallbackUpdateUserAccount>,
        randomness: [u8; 32],
    ) -> Result<()> {
        ctx.accounts
            .callback_update_user_account_on_er(randomness)?;

        Ok(())
    }

    pub fn update_commit(ctx: Context<UpdateCommit>, new_data: u64) -> Result<()> {
        ctx.accounts.update_commit(new_data)?;

        Ok(())
    }

    pub fn delegate(ctx: Context<Delegate>) -> Result<()> {
        ctx.accounts.delegate()?;

        Ok(())
    }

    pub fn undelegate(ctx: Context<Undelegate>) -> Result<()> {
        ctx.accounts.undelegate()?;

        Ok(())
    }

    pub fn close(ctx: Context<CloseUser>) -> Result<()> {
        ctx.accounts.close()?;

        Ok(())
    }
}
