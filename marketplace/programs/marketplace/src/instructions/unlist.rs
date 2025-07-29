use anchor_lang::prelude::*;
use anchor_lang::system_program::transfer;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{transfer_checked, Mint, Token, TokenAccount, Transfer, TransferChecked};

use crate::error::*;
use crate::state::{Listing, Marketplace};
#[derive(Accounts)]
#[instruction(name:String)]
pub struct Unlist<'info> {
    #[account[mut]]
    pub seller: Signer<'info>,
    #[account[
      seeds = [b"marketplace", marketplace.name.as_bytes()],
      bump = marketplace.bump
    ]]
    pub marketplace: Account<'info, Marketplace>,
    #[account[
      mint::token_program=token_program
    ]]
    pub seller_mint: Account<'info, Mint>,
    #[account[
      mut,
      associated_token::mint = seller_mint,
      associated_token::authority = seller
    ]]
    pub seller_ata: Account<'info, TokenAccount>,
    #[account[
      mut,
      close=seller,
      associated_token::mint=seller_mint,
      associated_token::authority=listing,
    ]]
    pub vault: Account<'info, TokenAccount>,
    #[account[
      seeds = [marketplace.key().as_ref(), seller_mint.key().as_ref()],
      bump=listing.bump
    ]]
    pub listing: Account<'info, Listing>,
    // metadata
    // master edition
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> Unlist<'info> {
    pub fn unlisting(&mut self) -> Result<()> {
        self.return_nft();
        Ok(())
    }

    pub fn return_nft(&mut self) -> Result<()> {
        let mkey = self.marketplace.key();
        let lkey = self.listing.key();

        let seeds: &[&[u8]] = &[mkey.as_ref(), lkey.as_ref(), &[self.listing.bump]];

        let s = [seeds];

        let cpi_ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            TransferChecked {
                from: self.vault.to_account_info(),
                to: self.seller_ata.to_account_info(),
                authority: self.listing.to_account_info(),
                mint: self.seller_mint.to_account_info(),
            },
            &s,
        );

        transfer_checked(cpi_ctx, 1, 0);

        Ok(())
    }
}
