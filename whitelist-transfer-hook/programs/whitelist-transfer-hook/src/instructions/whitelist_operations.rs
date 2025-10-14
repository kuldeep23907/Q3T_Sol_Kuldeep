use anchor_lang::{prelude::*, system_program};

use crate::state::whitelist::Whitelist;

#[derive(Accounts)]
pub struct WhitelistOperations<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    /// CHECK: Just a user
    pub user: AccountInfo<'info>,
    #[account(
        init_if_needed,
        space = 8+1+1,
        payer=admin,
        seeds = [b"whitelist", user.key().as_ref()],
        bump,
    )]
    pub whitelist: Account<'info, Whitelist>,
    pub system_program: Program<'info, System>,
}

impl<'info> WhitelistOperations<'info> {
    pub fn add_to_whitelist(&mut self) -> Result<()> {
        self.whitelist.allowed = true;
        Ok(())
    }

    pub fn remove_from_whitelist(&mut self) -> Result<()> {
        self.whitelist.allowed = false;
        Ok(())
    }
}
