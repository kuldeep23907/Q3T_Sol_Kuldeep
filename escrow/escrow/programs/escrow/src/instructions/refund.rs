pub use crate::state::Escrow;
use anchor_lang::{prelude::*, Bump};
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{
        close_account, transfer_checked, CloseAccount, Mint, TokenAccount, TokenInterface,
        TransferChecked,
    },
};

#[derive(Accounts)]
#[instruction(seed: u64)]
pub struct Refund<'info> {
    #[account[mut]]
    pub maker: Signer<'info>,
    #[account[
      mint::token_program = token_program
    ]]
    pub token_a: InterfaceAccount<'info, Mint>,
    #[account[
      mut,
      associated_token::mint = token_a,
      associated_token::authority = maker,
      associated_token::token_program = token_program
    ]]
    pub maker_ata_a: InterfaceAccount<'info, TokenAccount>,
    #[account[
      mut,
      seeds = [b"escrow", maker.key().as_ref(), escrow.seed.to_le_bytes().as_ref()],
      bump = escrow.bumb
    ]]
    pub escrow: Account<'info, Escrow>,
    #[account[
      associated_token::mint = token_a,
      associated_token::token_program = token_program,
      associated_token::authority = escrow
    ]]
    pub vault: InterfaceAccount<'info, TokenAccount>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

impl<'info> Refund<'info> {
    pub fn refund(&mut self) -> Result<()> {
        // transfer token b from taker to maker
        let cpi_program = self.token_program.to_account_info();
        let cpi_accounts = TransferChecked {
            from: self.vault.to_account_info(),
            to: self.maker.to_account_info(),
            mint: self.token_a.to_account_info(),
            authority: self.escrow.to_account_info(),
        };

        let signer_seeds: &[&[u8]] = &[
            b"escrow",
            self.maker.key.as_ref(),
            &self.escrow.seed.to_le_bytes()[..],
            &[self.escrow.bumb],
        ];
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, &[signer_seeds]);
        transfer_checked(cpi_ctx, self.vault.amount, self.token_a.decimals);

        // close the escrow
        let close_account = CloseAccount {
            account: self.escrow.to_account_info(),
            destination: self.maker.to_account_info(),
            authority: self.maker.to_account_info(),
        };

        let close_account_ctx =
            CpiContext::new_with_signer(cpi_program, close_account, &[signer_seeds]);

        close_account(close_account_ctx)?;

        Ok(())
    }
}

pub fn refund(ctx: Context<Refund>) -> Result<()> {
    ctx.accounts.refund();
    Ok(())
}
