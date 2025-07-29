// nft, nft ata
// nft metadata
// check if nft belong to nft
// stake it
// transfer nft
// freeze it

use crate::state::{StakeAccount, StakeConfig, UserAccount};
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    metadata::{Metadata, MetadataAccount, ThawDelegatedAccountCpi, ThawDelegatedAccount},
    token::{
        spl_token::instruction::transfer_checked, transfer, Mint, Token, TokenAccount,
        TransferChecked,
    },
};

#[derive(Accounts)]
pub struct Unstake<'info> {
    #[account[mut]]
    pub staker: Signer<'info>,
    #[account[
      mut,
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
      seeds=[b"stake", staker.key().as_ref(), stake_token.key().as_ref() ],
      bump = stake_account.bump
    ]]
    pub stake_account: Account<'info, StakeAccount>,
    #[account[
      mint::token_program = token_program
    ]]
    pub stake_token: Account<'info, Mint>,
    pub collection_mint: Account<'info, Mint>,
    #[account[
      mut,
      associated_token::mint = stake_token,
      associated_token::authority = user,
    ]]
    pub user_stake_token_ata: InterfaceAccount<'info, TokenAccount>,
    #[account[
      seeds=[b"metadata", metadata_program.key().as_ref(), mint.key().as_ref()],
      seeds::program = metadata_program.key(),
      bump,
      constraint = metadata.collection.as_ref().unwrap().key.as_ref() == collection_mint.key().as_ref(),
      constraint = metadata.collection.as_ref().unwrap().verified == true
    ]]
    pub metadata: Account<'info, MetadataAccount>,
    #[account(
        seeds = [
            b"metadata",
            metadata_program.key().as_ref(),
            stake_token.key().as_ref(),
            b"edition"
        ],
        seeds::program = metadata_program.key(),
        bump,
    )]
    pub edition: Account<'info, MasterEditionAccount>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub metadata_program: Program<'info, Metadata>,

    pub system_program: Program<'info, System>,
}

impl<'info> Unstake<'info> {
    pub fn unstake(&mut self) -> Result<()> {
        let clock = Clock::get()?; // Pull the clock sysvar
        let current_time = clock.unix_timestamp; // i64 in seconds

        let time_elapsed = current_time - self.stake_account.staked_at;
        require!(
            ((time_elapsed / 86400) as u32) < self.config.freeze_period,
            StakeError::FreePeriodNotOver
        );

        self.staker_account.points +=
            ((time_elapsed / 86400) as u32) * self.config.reward_per_token;

        // unfreeze
        let seeds = &[
            b"stake",
            self.staker.key().as_ref(),
            self.stake_token.key().as_ref(),
            &[self.stake_account.bump],
        ];

        let delegate = self.stake_account.to_account_info();
        let token_account = self.stake_token.to_account_info();
        let edition = self.edition.to_account_info();
        let

        // update amount staked
        // close the stake account

        Ok(())

        // require!(
        //     self.staker_account.amount_staked <= self.config.max_stake,
        //     CustomError::StakeLimitReached
        // );
        // let clock = Clock::get()?; // Pull the clock sysvar
        // let current_time = clock.unix_timestamp; // i64 in seconds

        // self.stake_account.set_inner(StakeAccount {
        //     stake_token: self.stake_token.key(),
        //     owner: self.staker.key(),
        //     staked_at: current_time,
        //     bump: bumps.stake_account,
        // });

        // self.staker_account.amount_staked += 1;

        // let cpi_program = self.token_program.to_account_info();
        // let cpi_accounts = Transfer {
        //     from: self.user_stake_token_ata.to_account_info(),
        //     to: self.config_stake_token_ata.to_account_info(),
        //     authority: self.staker.to_account_info(),
        // };
        // let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        // transfer(cpi_ctx, 1);
        // Ok(())
    }
}

// pub fn handler(ctx: Context<Config>) -> Result<()> {
//     // msg!("Greetings from: {{:?}}", ctx.program_id);
//     Ok(())
// }
