use crate::error::*;
use crate::state::Config;
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{burn, mint_to, Burn, MintTo},
    token_interface::{transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked},
};
use constant_product_curve::{ConstantProduct, LiquidityPair, SwapResult};

#[derive(Accounts)]
#[instruction(seed:u64)]
pub struct Swap<'info> {
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
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

impl<'info> Swap<'info> {
    pub fn swap(&mut self, amount_swap: u64, amount_receive: u64, direction: bool) -> Result<()> {
        require!(self.config.locked == false, AmmError::PoolLocked);
        require!(amount_swap != 0, AmmError::InvalidAmount);

        let mut cp = ConstantProduct::init(
            self.vault_x.amount,
            self.vault_y.amount,
            self.mint_lp.supply,
            self.config.fees,
            Some(6),
        )
        .unwrap();

        let mut result;
        if (direction) {
            result = cp
                .swap(LiquidityPair::X, amount_swap, amount_receive)
                .unwrap();
        } else {
            result = cp
                .swap(LiquidityPair::Y, amount_swap, amount_receive)
                .unwrap();
        }

        self._swap(result, direction);
        Ok(())
    }
    fn _swap(&mut self, swap_params: SwapResult, direction: bool) -> Result<()> {
        if (direction) {
            // deposit x from user
            // send y to user
            let cpi_program = self.token_program.to_account_info();
            let cpi_accounts = TransferChecked {
                from: self.user_x.to_account_info(),
                to: self.vault_x.to_account_info(),
                mint: self.mint_x.to_account_info(),
                authority: self.user.to_account_info(),
            };
            let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
            transfer_checked(cpi_ctx, swap_params.deposit, self.mint_x.decimals);

            let seeds = &[
                &b"config"[..],
                &self.config.seed.to_le_bytes(),
                &[self.config.config_bump],
            ];

            let signer_seeds = &[&seeds[..]];

            let cpi_program = self.token_program.to_account_info();
            let cpi_accounts = TransferChecked {
                from: self.vault_y.to_account_info(),
                to: self.user_y.to_account_info(),
                mint: self.mint_y.to_account_info(),
                authority: self.config.to_account_info(),
            };
            let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);
            transfer_checked(cpi_ctx, swap_params.withdraw, self.mint_y.decimals);
        } else {
            // deposit y from user
            // send x to user
            let cpi_program = self.token_program.to_account_info();
            let cpi_accounts = TransferChecked {
                from: self.user_y.to_account_info(),
                to: self.vault_y.to_account_info(),
                mint: self.mint_y.to_account_info(),
                authority: self.user.to_account_info(),
            };
            let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
            transfer_checked(cpi_ctx, swap_params.deposit, self.mint_y.decimals);

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
            transfer_checked(cpi_ctx, swap_params.withdraw, self.mint_x.decimals);
        }
        Ok(())
    }
}
