// nft, nft ata
// nft metadata
// check if nft belong to nft
// stake it
// freeze it

use crate::state::{StakeAccount, StakeConfig, UserAccount};
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    metadata::{
        freeze_delegated_account, FreezeDelegatedAccount, MasterEditionAccount, Metadata,
        MetadataAccount,
    },
    token::{
        spl_token::instruction::transfer_checked, transfer, Mint, Token, TokenAccount,
        TransferChecked,
    },
};

use crate::error::*;
#[derive(Accounts)]
pub struct Stake<'info> {
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
      init,
      payer=staker,
      seeds=[b"stake", staker.key().as_ref(), stake_token.key().as_ref() ],
      bump,
      space = 8 + StakeAccount::INIT_SPACE
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
      init,
      payer=staker,
      associated_token::mint = stake_token,
      associated_token::authority = config,
      associated_token::token_program = token_program
    ]]
    pub config_stake_token_ata: Account<'info, TokenAccount>,
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

impl<'info> Stake<'info> {
    pub fn stake(&mut self, bumps: &StakeBumps) -> Result<()> {
        require!(
            self.staker_account.amount_staked <= self.config.max_stake,
            CustomError::Error1
        );
        let clock = Clock::get()?; // Pull the clock sysvar
        let current_time = clock.unix_timestamp; // i64 in seconds

        self.stake_account.set_inner(StakeAccount {
            stake_token: self.stake_token.key(),
            owner: self.staker.key(),
            staked_at: current_time,
            bump: bumps.stake_account,
        });

        self.staker_account.amount_staked += 1;
        // free nft authority till stake is over

        let freeze_accounts = FreezeDelegatedAccount {
            metadata: self.metadata.to_account_info(),
            delegate: self.stake_account.to_account_info(),
            token_account: self.user_stake_token_ata.to_account_info(),
            mint: self.stake_token.to_account_info(),
            edition: self.edition.to_account_info(),
            token_program: self.token_program.to_account_info(),
        };

        let ctx = CpiContext::new(self.token_program.to_account_info(), freeze_accounts);
        freeze_delegated_account(ctx);

        Ok(())
    }
}
