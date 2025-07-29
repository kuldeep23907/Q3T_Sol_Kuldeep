pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("GL7yCFFDvAQxqrN2U8KEPa8duuFiZEQFxi3S8dosfi8j");

#[program]
pub mod marketplace {
    use super::*;
    pub fn listing_nft(ctx: Context<List>, price: u64) -> Result<()> {
        ctx.accounts.listing(price, &ctx.bumps);
        ctx.accounts.deposit_nft();
        Ok(())
    }

    pub fn purchase_nft(ctx: Context<Purchase>) -> Result<()> {
        ctx.accounts.purchase();
        ctx.accounts.purchasing_nft();
        Ok(())
    }

    pub fn unlist_nft(ctx: Context<Unlist>) -> Result<()> {
        ctx.accounts.unlisting();
        Ok(())
    }
}
