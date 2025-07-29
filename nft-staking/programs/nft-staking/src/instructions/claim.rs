use crate::state::{StakeAccount, StakeConfig, UserAccount};
use anchor_lang::{prelude::*, solana_program::sysvar::rewards};
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{
        spl_token::instruction::transfer_checked, transfer, Mint, Token, TokenAccount,
        TransferChecked,
    },
};

#[derive(Accounts)]
pub struct Claim<'info> {
    #[account[mut]]
    pub staker: Signer<'info>,
    #[account[
      seeds=[b"user", staker.key().as_ref()],
      bump= user_account.bump
    ]]
    pub staker_account: Account<'info, UserAccount>,
    #[account[
      seeds=[b"config"],
      bump = config.bump
    ]]
    pub config: Account<'info, StakeConfig>,
    #[account[
      mint::token_program = token_program
    ]]
    pub stake_token: InterfaceAccount<'info, Mint>,
    #[account[
      seeds=[b"stake", staker.key().as_ref(), stake_token.key().as_ref() ],
      bump = stake_account.bump
    ]]
    pub stake_account: Account<'info, StakeAccount>,
    #[account[
      seeds = [b"reward", config.key().as_ref()],
      bump= config.rewards_bump,
      mint::authority = config
    ]]
    pub reward_mint: Account<'info, Mint>,
    #[account[
      init,
      payer=staker,
      associated_token::mint=reward_mint,
      associated_token::authority=staker,
      associated_token::token_program=token_program
    ]]
    pub user_reward_mint_ata: InterfaceAccount<'info, TokenAccount>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl<'info> Claim<'info> {
    pub fn claim(&mut self) -> Result<()> {
        let clock = Clock::get()?; // Pull the clock sysvar
        let current_time = clock.unix_timestamp; // i64 in seconds

        let staked_for = current_time - self.stake_account.staked_at;

        let rewards = staked_for as u64 * self.config.reward_per_token as u64;

        Ok(())
    }
}

// pub fn handler(ctx: Context<Config>) -> Result<()> {
//     // msg!("Greetings from: {{:?}}", ctx.program_id);
//     Ok(())
// }
