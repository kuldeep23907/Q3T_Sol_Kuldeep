use crate::error::*;
use crate::state::Config;
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{mint_to, MintTo, Burn, burn},
    token_interface::{transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked},
};
use constant_product_curve::ConstantProduct;

#[derive(Accounts)]
#[instruction(seed:u64)]
pub struct Withdraw<'info> {
    #[account[mut]]
    pub user: Signer<'info>,
    pub mint_x: Account<'info, Mint>,
    pub mint_y: Account<'info, Mint>,
    #[account[
      has_one = mint_x,
      has_one = mint_y,
      seeds = [b"config", seed.to_le_bytes().as_ref()],
      bump,
    ]]
    pub config: Account<'info, Config>,
    #[account[

      seeds = [b"lp", config.key().as_ref()],
      bump,
    ]]
    pub mint_lp: Account<'info, Mint>,

    #[account[
     associated_token::mint = mint_x,
     associated_token::authority = config,
    ]]
    pub vault_x: Account<'info, TokenAccount>,
    #[account[
     associated_token::mint = mint_y,
     associated_token::authority = config,
    ]]
    pub vault_y: Account<'info, TokenAccount>,
    #[account[
      associated_token::mint = mint_x,
      associated_token::authority = user,
     ]]
    pub user_x: Account<'info, TokenAccount>,
    #[account[
      associated_token::mint = mint_y,
      associated_token::authority = user,
     ]]
    pub user_y: Account<'info, TokenAccount>,
    #[account[
      init_if_needed,
      payer=user,
      associated_token::mint = mint_lp,
      associated_token::token_program = token_program,
      associated_token::authority = user,
     ]]
    pub user_lp: Account<'info, TokenAccount>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

impl<'info> Withdraw<'info> {
    pub fn withdraw(&mut self, lp_amount: u64) -> Result<()> {
        require!(self.config.locked == false, AmmError::PoolLocked);
        require!(lp_amount != 0, AmmError::InvalidAmount);
        let amount = ConstantProduct::xy_withdraw_amounts_from_l(
            self.vault_x.amount,
            self.vault_y.amount,
            self.mint_lp.supply,
            lp_amount,
            6,
        )
        .unwrap();

        self._withdraw(lp_amount, amount.x, amount.y);
        Ok(())
    }
    fn _withdraw(&mut self, amount_lp:u64,  amount_x: u64, amount_y: u64) -> Result<()> {
        let seeds = &[
            &b"config"[..],
            &self.config.seed.to_le_bytes(),
            &[self.config.config_bump],
        ];

        let signer_seeds = &[&seeds[..]];

        let cpi_program = self.token_program.to_account_info();
        let cpi_accounts = TransferChecked {
            from: self.vault_x.to_account_info(),
            to: self.user_x.to_account_info(),
            mint: self.mint_x.to_account_info(),
            authority: self.config.to_account_info(),
        };
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);
        transfer_checked(cpi_ctx, amount_x, self.mint_x.decimals);

        let cpi_program = self.token_program.to_account_info();
        let cpi_accounts = TransferChecked {
            from: self.vault_y.to_account_info(),
            to: self.user_y.to_account_info(),
            mint: self.mint_y.to_account_info(),
            authority: self.config.to_account_info(),
        };
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds)
        transfer_checked(cpi_ctx, amount_y, self.mint_y.decimals);

        let cpi_program = self.token_program.to_account_info();
        let cpi_accounts = Burn {
          mint: self.mint_lp.to_account_info(),
          from: self.user_lp.to_account_info(),
          authority: self.user.to_account_info()
        };
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        // vur(cpi_ctx, amount_y, self.mint_y.decimals);
        burn(cpi_ctx, amount_lp);


        Ok(())
    }
}
