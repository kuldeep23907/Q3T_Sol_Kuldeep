// nft, nft ata
// nft metadata
// check if nft belong to nft
// stake it
// transfer nft
// freeze it

use crate::error::*;
use crate::state::{StakeAccount, StakeConfig, UserAccount};
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    metadata::{
        thaw_delegated_account, MasterEditionAccount, Metadata, MetadataAccount,
        ThawDelegatedAccount,
    },
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
      bump= staker_account.bump
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
      associated_token::authority = staker,
    ]]
    pub user_stake_token_ata: Account<'info, TokenAccount>,
    #[account[
      seeds=[b"metadata", metadata_program.key().as_ref(), stake_token.key().as_ref()],
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
            CustomError::Error1
        );

        self.staker_account.points +=
            ((time_elapsed / 86400) as u32) * self.config.reward_per_token;

        let skey = &self.staker.key();
        let sskey = &self.stake_token.key();

        // Construct the seed array for PDA
        let seeds: &[&[u8]; 4] = &[
            b"stake",
            skey.as_ref(),
            sskey.as_ref(),
            &[self.stake_account.bump],
        ];

        // `invoke_signed` expects a `&[&[&[u8]]]`
        // So just use `&[seeds[..]]` to match the type
        let s: &[&[&[u8]]] = &[&seeds[..]];

        let thaw_account = ThawDelegatedAccount {
            metadata: self.metadata.to_account_info(),
            delegate: self.stake_account.to_account_info(),
            token_account: self.user_stake_token_ata.to_account_info(),
            mint: self.stake_token.to_account_info(),
            edition: self.edition.to_account_info(),
            token_program: self.token_program.to_account_info(),
        };

        let thaw_ctx =
            CpiContext::new_with_signer(self.token_program.to_account_info(), thaw_account, s);

        thaw_delegated_account(thaw_ctx);
        Ok(())
    }
}
