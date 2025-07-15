use crate::error::*;
use crate::state::Config;
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{mint_to, MintTo},
    token_interface::{transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked},
};
use constant_product_curve::ConstantProduct;

#[derive(Accounts)]
#[instruction(seed:u64)]
pub struct Deposit<'info> {
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

impl<'info> Deposit<'info> {
    pub fn deposit(&mut self, amount: u64, max_x: u64, max_y: u64) -> Result<()> {
        require!(self.config.locked == false, AmmError::PoolLocked);
        require!(amount != 0, AmmError::InvalidAmount);
        let (x, y) = match self.mint_lp.supply == 0
            && self.vault_x.amount == 0
            && self.vault_y.amount == 0
        {
            true => (max_x, max_y),
            false => {
                let amount = ConstantProduct::xy_deposit_amounts_from_l(
                    self.vault_x.amount,
                    self.vault_y.amount,
                    self.mint_lp.supply,
                    amount,
                    6,
                )
                .unwrap();
                (amount.x, amount.y)
            }
        };

        require!(x <= max_x && y <= max_y, AmmError::SlippageExceeded);

        self._deposit(amount, x, y);
        Ok(())
    }
    fn _deposit(&mut self, amount_x: u64, amount_y: u64, amount_lp: u64) -> Result<()> {
        let cpi_program = self.token_program.to_account_info();
        let cpi_accounts = TransferChecked {
            from: self.user.to_account_info(),
            to: self.vault_x.to_account_info(),
            mint: self.mint_x.to_account_info(),
            authority: self.user.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        transfer_checked(cpi_ctx, amount_x, self.mint_x.decimals);

        let cpi_program = self.token_program.to_account_info();
        let cpi_accounts = TransferChecked {
            from: self.user.to_account_info(),
            to: self.vault_y.to_account_info(),
            mint: self.mint_y.to_account_info(),
            authority: self.user.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        transfer_checked(cpi_ctx, amount_y, self.mint_x.decimals);

        let cpi_program = self.token_program.to_account_info();

        let cpi_accounts = MintTo {
            mint: self.mint_lp.to_account_info(),
            to: self.user_lp.to_account_info(),
            authority: self.config.to_account_info(),
        };

        let seeds = &[
            &b"config"[..],
            &self.config.seed.to_le_bytes(),
            &[self.config.config_bump],
        ];

        let signer_seeds = &[&seeds[..]];

        let ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);

        mint_to(ctx, amount_lp);

        Ok(())
    }
}
