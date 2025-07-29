use anchor_lang::{prelude::*, solana_program::system_instruction::transfer};

use crate::state::*;

#[derive(Accounts)]
#[instruction[seed:u128]]
pub struct PlaceBet<'info> {
    #[account[mut]]
    pub player: Signer<'info>,
    /// CHECK: Safe
    #[account[]]
    pub house: UncheckedAccount<'info>,
    #[account[
      init,
      space= 8 + Bet::INIT_SPACE,
      payer=player,
      seeds = [b"bet", vault.key().as_ref(), seed.to_le_bytes().as_ref()],
      bump
    ]]
    pub bet: Account<'info, Bet>,
    #[account[
      mut,
      seeds = [b"vault", house.key().as_ref()],
      bump
    ]]
    pub vault: SystemAccount<'info>,
    pub system_program: Program<'info, System>,
}

impl<'info> PlaceBet<'info> {
    pub fn place_bet(
        &mut self,
        bumps: &PlaceBetBumps,
        seed: u128,
        roll: u8,
        amount: u64,
    ) -> Result<()> {
        self.bet.set_inner(Bet {
            player: self.player.key(),
            amount,
            slot: Clock::get()?.slot,
            seed,
            roll,
            bump: bumps.bet,
        });
        Ok(())

        // deposit SOL from player to vault
    }
}
