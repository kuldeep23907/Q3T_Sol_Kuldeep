use anchor_lang::prelude::*;
use anchor_lang::solana_program::system_instruction::transfer;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{transfer_checked, Mint, Token, TokenAccount, Transfer, TransferChecked};

use crate::error::*;
use crate::state::{Listing, Marketplace};
#[derive(Accounts)]
#[instruction(name:String)]
pub struct Purchase<'info> {
    #[account[mut]]
    pub buyer: Signer<'info>,
    #[account[
      mint::token_program=token_program,
    ]]
    pub mint: Account<'info, Mint>,
    #[account[
      seeds = [b"marketplace", marketplace.name.as_bytes()],
      bump = marketplace.bump
    ]]
    pub marketplace: Account<'info, Marketplace>,
    #[account[
      mut,
      associated_token::mint = listing.mint,
      associated_token::authority = buyer
    ]]
    pub buyer_ata: Account<'info, TokenAccount>,
    #[account[
      mut,
      associated_token::mint=listing.mint,
      associated_token::authority=listing,
      close=buyer
    ]]
    pub vault_ata: Account<'info, TokenAccount>,
    #[account[
      mut,
      seeds = [marketplace.key().as_ref(), listing.mint.key().as_ref()],
      bump,
      close=buyer
    ]]
    pub listing: Account<'info, Listing>,
    #[account[
       seeds=[b"treasury", marketplace.key().as_ref()],
      bump=marketplace.treasury_bump
    ]]
    pub treasury: SystemAccount<'info>,
    // metadata
    // master edition
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> Purchase<'info> {
    pub fn purchase(&mut self) -> Result<()> {
        require!(
            self.mint.key() == self.listing.mint.key(),
            MarketplaceError::CustomError1
        );
        // transfer of SOL
        let price = self.listing.price;
        // let fees = price * (self.marketplace.fees as u64 / 10000);
        let fees = price
            .checked_mul((self.marketplace.fees as u64))
            .unwrap()
            .checked_div(10000_u64)
            .unwrap();

        let price_to_be_sent = price.checked_sub(fees).unwrap();

        transfer(
            &self.buyer.key(),
            &self.listing.seller.key(),
            price_to_be_sent,
        );

        transfer(&self.buyer.key(), &self.treasury.key(), fees);

        Ok(())
    }

    pub fn purchasing_nft(&mut self) -> Result<()> {
        let mkey = self.marketplace.key();
        let lkey = self.listing.key();

        let seeds: &[&[u8]] = &[mkey.as_ref(), lkey.as_ref(), &[self.listing.bump]];

        let s = [seeds];

        let cpi_ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            TransferChecked {
                from: self.vault_ata.to_account_info(),
                to: self.buyer_ata.to_account_info(),
                authority: self.listing.to_account_info(),
                mint: self.mint.to_account_info(),
            },
            &s,
        );

        transfer_checked(cpi_ctx, 1, 0);

        Ok(())
    }
}
