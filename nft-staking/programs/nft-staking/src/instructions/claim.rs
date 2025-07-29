use std::thread::sleep;

use crate::state::{StakeAccount, StakeConfig, UserAccount};
use anchor_lang::{prelude::*, solana_program::sysvar::rewards};
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{
        mint_to, spl_token::instruction::transfer_checked, transfer, Mint, MintTo, Token,
        TokenAccount,
    },
};

#[derive(Accounts)]
pub struct Claim<'info> {
    #[account[mut]]
    pub staker: Signer<'info>,
    #[account[
      seeds=[b"user", staker.key().as_ref()],
      bump= staker_account.bump
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
    pub stake_token: Account<'info, Mint>,
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
    pub user_reward_mint_ata: Account<'info, TokenAccount>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl<'info> Claim<'info> {
    pub fn claim(&mut self) -> Result<()> {
        // mint rewards to user as per user points
        let mint_rewards = MintTo {
            mint: self.reward_mint.to_account_info(),
            to: self.staker.to_account_info(),
            authority: self.config.to_account_info(),
        };

        // unfreeze
        let signer_seeds: &[&[u8]] = &[
            b"config",           // Static byte seed
            &[self.config.bump], // `u8` bump converted to a slice
        ];

        mint_to(
            CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                mint_rewards,
                &[signer_seeds],
            ),
            self.staker_account.points as u64,
        )?;

        self.staker_account.points = 0;
        // update points
        Ok(())
    }
}
