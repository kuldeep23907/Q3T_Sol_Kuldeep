use crate::state::StakeConfig;
use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{Mint, Token};

#[derive(Accounts)]
pub struct Config<'info> {
    #[account[mut]]
    pub admin: Signer<'info>,
    #[account[
      init,
      payer=admin,
      seeds=[b"config"],
      bump,
      space = 8 + StakeConfig::INIT_SPACE
    ]]
    pub config: Account<'info, StakeConfig>,
    #[account[
      init_if_needed,
      payer=admin,
      seeds = [b"reward", config.key().as_ref()],
      bump,
      mint::decimals = 6,
      mint::authority = config
    ]]
    pub reward_mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> Config<'info> {
    pub fn init_config(
        &mut self,
        bumps: &ConfigBumps,
        reward_per_token: u32,
        max_stake: u32,
        freeze_period: u32,
    ) -> Result<()> {
        self.config.set_inner(StakeConfig {
            reward_per_token,
            max_stake,
            freeze_period,
            rewards_bump: bumps.reward_mint,
            bump: bumps.config,
        });
        Ok(())
    }
}
