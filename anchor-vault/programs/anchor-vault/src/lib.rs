#![allow(deprecated)] // for no warnings
#![allow(unexpected_cfgs)]
use anchor_lang::{
    prelude::*,
    solana_program::lamports,
    system_program::{transfer, Transfer},
};

declare_id!("FHou4Jf1ZKEZA7ZEZiWoFMYr5BUoHBBwMmXryvYcKF88");

#[program]
pub mod anchor_vault {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        ctx.accounts.initialize((&ctx.bumps))
    }

    pub fn deposit(ctx: Context<Payment>, amount: u64) -> Result<()> {
        ctx.accounts.deposit(amount)
    }

    pub fn withdraw(ctx: Context<Payment>, amount: u64) -> Result<()> {
        ctx.accounts.withdraw(amount)
    }

    pub fn close_vault(ctx: Context<CloseVault>) -> Result<()> {
        ctx.accounts.close_vault()
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account[mut]]
    pub user: Signer<'info>,
    #[account[
     init,
     payer = user,
     space = VaultState::INIT_SPACE,
     seeds = [b"state", user.key().as_ref()],
     bump
    ]]
    pub vault_state: Account<'info, VaultState>,
    #[account[
      mut,
      seeds = [b"vault", vault_state.key().as_ref()],
      bump
    ]]
    pub vault: SystemAccount<'info>,
    pub system_program: Program<'info, System>,
}

impl<'info> Initialize<'info> {
    pub fn initialize(&mut self, bumps: &InitializeBumps) -> Result<()> {
        let rent_exempt = Rent::get()?.minimum_balance(self.vault.to_account_info().data_len());
        let cpi_program = self.system_program.to_account_info();
        let cpi_account = Transfer {
            from: self.user.to_account_info(),
            to: self.vault.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(cpi_program, cpi_account);
        transfer(cpi_ctx, rent_exempt)?;

        self.vault_state.vault_bump = bumps.vault;
        self.vault_state.state_bump = bumps.vault_state;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Payment<'info> {
    #[account[mut]]
    pub user: Signer<'info>,
    #[account[
      mut,
      seeds = [b"vault", vault_state.key().as_ref()],
      bump = vault_state.vault_bump
    ]]
    pub vault: SystemAccount<'info>,
    #[account[
      seeds = [b"state", user.key().as_ref()],
      bump = vault_state.state_bump
    ]]
    pub vault_state: Account<'info, VaultState>,
    pub system_program: Program<'info, System>,
}

impl<'info> Payment<'info> {
    pub fn deposit(&mut self, amount: u64) -> Result<()> {
        let cpi_program = self.system_program.to_account_info();
        let cpi_accounts = Transfer {
            from: self.user.to_account_info(),
            to: self.vault.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

        transfer(cpi_ctx, amount)
    }

    pub fn withdraw(&mut self, amount: u64) -> Result<()> {
        let cpi_program = self.system_program.to_account_info();
        let cpi_accounts = Transfer {
            from: self.vault.to_account_info(),
            to: self.user.to_account_info(),
        };

        let seeds = &[
            b"vault",
            self.vault_state.to_account_info().key.as_ref(),
            &[self.vault_state.vault_bump],
        ];

        let signer_seeds = &[&seeds[..]];

        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);

        transfer(cpi_ctx, amount)
    }
}

#[derive(Accounts)]
pub struct CloseVault<'info> {
    #[account[mut]]
    pub user: Signer<'info>,
    #[account[
      mut,
      seeds = [b"vault",vault_state.key().as_ref()],
      bump = vault_state.vault_bump
    ]]
    pub vault: SystemAccount<'info>,
    #[account[
      mut,
      seeds = [b"state",user.key().as_ref()],
      bump = vault_state.state_bump
    ]]
    pub vault_state: Account<'info, VaultState>,
    pub system_program: Program<'info, System>,
}

impl<'info> CloseVault<'info> {
    pub fn close_vault(&mut self) -> Result<()> {
        // let cpi_program = self.system_program.to_account_info();
        // let cpi_accounts = Transfer {
        //     from: self.vault.to_account_info(),
        //     to: self.user.to_account_info(),
        // };

        // let seeds = &[
        //     b"vault",
        //     self.vault_state.to_account_info().key.as_ref(),
        //     &[self.vault_state.vault_bump],
        // ];

        // let signer_seeds = &[&seeds[..]];

        // let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);

        // let new_data = 0usize;
        // let rent_for_zero = Rent::get()?.minimum_balance(new_data);

        // let lamports_to_tranfer = self.vault.lamports() - rent_for_zero;

        // transfer(cpi_ctx, lamports_to_tranfer);

        // self.vault.realloc(new_data, true);

        // self.vault.close(self.user.to_account_info())?;

        let ix = anchor_lang::solana_program::system_instruction::transfer(
            self.vault.key,
            self.user.key,
            self.vault.lamports(),
        );

        anchor_lang::solana_program::program::invoke_signed(
            &ix,
            &[self.vault.to_account_info(), self.user.to_account_info()],
            &[&[
                b"vault",
                self.vault_state.key().as_ref(),
                &[self.vault_state.vault_bump],
            ]],
        )?;
        Ok(())
    }
}

#[account]
// #[derive(InitSpace)]
pub struct VaultState {
    pub vault_bump: u8,
    pub state_bump: u8,
}

impl Space for VaultState {
    const INIT_SPACE: usize = 8 + 1 + 1;
}
