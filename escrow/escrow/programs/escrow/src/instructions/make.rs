pub use crate::state::Escrow;
use anchor_lang::{prelude::*, Bump};
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked},
};

#[derive(Accounts)]
#[instruction(seed: u64)]
pub struct Make<'info> {
    #[account[mut]]
    pub userA: Signer<'info>,
    #[account[
      mint::token_program = token_program
    ]]
    pub mintA: InterfaceAccount<'info, Mint>,
    #[account[
      mint::token_program = token_program
    ]]
    pub mintB: InterfaceAccount<'info, Mint>,
    #[account[
      mut,
      associated_token::mint = mintA,
      associated_token::authority = userA,
      associated_token::token_program = token_program
    ]]
    pub user_a_ata: InterfaceAccount<'info, TokenAccount>,
    #[account[
      init,
      space = 8 + Escrow::INIT_SPACE,
      payer = userA,
      seeds = [b"escrow", userA.key().as_ref(), seed.to_le_bytes().as_ref()],
      bump
    ]]
    pub escrow: Account<'info, Escrow>,
    #[account[
      init,
      payer=userA,
      associated_token::mint = mintA,
      associated_token::token_program = token_program,
      associated_token::authority = escrow
    ]]
    pub vault: InterfaceAccount<'info, TokenAccount>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

impl<'info> Make<'info> {
    pub fn make(&mut self, seed: u64, recieve: u64, bumps: &MakeBumps) -> Result<()> {
        self.escrow.set_inner(Escrow {
            seed: (seed),
            userA: (self.userA.key()),
            mintA: (self.mintA.key()),
            mintB: (self.mintB.key()),
            receive: (recieve),
            bumb: (bumps.escrow),
        });

        Ok(())
    }

    pub fn deposit(&mut self, deposit: u64) -> Result<()> {
        // transfer token a to vault
        let cpi_program = self.token_program.to_account_info();
        let cpi_accounts = TransferChecked {
            from: self.user_a_ata.to_account_info(),
            to: self.vault.to_account_info(),
            mint: self.mintA.to_account_info(),
            authority: self.userA.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

        transfer_checked(cpi_ctx, deposit, self.mintA.decimals);
        Ok(())
    }
}

pub fn make(ctx: Context<Make>, seed: u64, recieve: u64, deposit: u64) -> Result<()> {
    ctx.accounts.make(seed, recieve, &ctx.bumps);
    ctx.accounts.deposit(deposit);
    Ok(())
}
