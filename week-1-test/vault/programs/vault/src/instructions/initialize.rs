use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenAccount, TokenInterface},
};

use crate::state::Config;

#[derive(Accounts)]
pub struct InitVault<'info> {
    #[account[mut]]
    pub admin: Signer<'info>,
    #[account[
      init,
      payer=admin,
      mint::decimals = 6,
      mint::authority = admin,
      mint::token_program=token_program,
      extensions::transfer_hook::authority = admin,
      extensions::transfer_hook::program_id = hook_program.key(),
    ]]
    pub mint: InterfaceAccount<'info, Mint>,
    #[account[
      init,
      payer=admin,
      space= 8 + Config::INIT_SPACE,
      seeds = [b"config"],
      bump
    ]]
    pub config: Account<'info, Config>,
    #[account[
      init,
      payer=admin,
      associated_token::mint=mint,
      associated_token::token_program=token_program,
      associated_token::authority=config
    ]]
    pub vault: InterfaceAccount<'info, TokenAccount>,
    /// CHECK: Transfer hook program
    pub hook_program: UncheckedAccount<'info>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl InitVault<'_> {
    pub fn init_vault(&mut self, bumps: &InitVaultBumps) -> Result<()> {
        self.config.set_inner(Config {
            admin: self.admin.key(),
            mint: self.mint.key(),
            vault: self.vault.key(),
            bump: bumps.config,
        });
        Ok(())
    }
}
