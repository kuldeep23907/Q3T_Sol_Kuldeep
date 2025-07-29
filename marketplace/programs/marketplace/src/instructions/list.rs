use anchor_lang::prelude::*;
use anchor_lang::system_program::transfer;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{transfer_checked, Mint, Token, TokenAccount, Transfer, TransferChecked};

use crate::state::{Listing, Marketplace};
use crate::Error::*;
#[derive(Accounts)]
#[instruction(name:String)]
pub struct List<'info> {
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
      init,
      payer=seller,
      associated_token::mint=seller_mint,
      associated_token::authority=listing,
    ]]
    pub vault: Account<'info, TokenAccount>,
    #[account[
      init,
      space = 8+Listing::INIT_SPACE,
      payer=seller,
      seeds = [marketplace.key().as_ref(), seller_mint.key().as_ref()],
      bump
    ]]
    pub listing: Account<'info, Listing>,
    // metadata
    // master edition
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> List<'info> {
    pub fn listing(&mut self, price: u64, bumps: &ListBumps) -> Result<()> {
        self.listing.set_inner(Listing {
            seller: self.seller.key(),
            mint: self.seller_mint.key(),
            price,
            bump: bumps.listing,
        });

        Ok(())
    }

    pub fn deposit_nft(&mut self) -> Result<()> {
        let cpi_ctx = CpiContext::new(
            self.token_program.to_account_info(),
            TransferChecked {
                from: self.seller_ata.to_account_info(),
                to: self.vault.to_account_info(),
                authority: self.seller.to_account_info(),
                mint: self.seller_mint.to_account_info(),
            },
        );

        transfer_checked(cpi_ctx, 1, 0);

        Ok(())
    }
}
