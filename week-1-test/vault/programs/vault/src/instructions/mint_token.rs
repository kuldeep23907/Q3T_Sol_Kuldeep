use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_2022::{mint_to, MintTo},
    token_interface::{Mint, TokenAccount, TokenInterface},
};

use crate::error::VaultError;
use crate::state::Config;

#[derive(Accounts)]
pub struct MintToken<'info> {
    #[account[mut]]
    pub admin: Signer<'info>,
    /// CHECK: System account
    pub user: AccountInfo<'info>,
    #[account[
      mut,
      mint::token_program=token_program
    ]]
    pub mint: InterfaceAccount<'info, Mint>,
    #[account[
      init,
      payer=admin,
      associated_token::mint=mint,
      associated_token::token_program=token_program,
      associated_token::authority=user
    ]]
    pub user_ata: InterfaceAccount<'info, TokenAccount>,
    #[account[
      seeds = [b"config"],
      bump = config.bump
    ]]
    pub config: Account<'info, Config>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl MintToken<'_> {
    pub fn mint_token(&mut self) -> Result<()> {
        require!(
            self.admin.key() == self.config.admin,
            VaultError::Unauthorized
        );
        let ctx_accounts = MintTo {
            mint: self.mint.to_account_info(),
            to: self.user_ata.to_account_info(),
            authority: self.admin.to_account_info(),
        };

        let ctx = CpiContext::new(self.token_program.to_account_info(), ctx_accounts);

        mint_to(ctx, 1000_000_000)?;
        Ok(())
    }
}
