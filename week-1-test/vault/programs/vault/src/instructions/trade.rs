use anchor_lang::prelude::*;
use anchor_lang::solana_program::instruction::Instruction;
use anchor_lang::solana_program::program::invoke;
use anchor_lang::solana_program::program::invoke_signed;
use anchor_spl::token_2022::spl_token_2022::instruction::transfer_checked;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_2022::{mint_to, MintTo, TransferChecked},
    token_interface::{Mint, TokenAccount, TokenInterface},
};

use crate::error::VaultError;
use crate::{state::Config, Position};

#[derive(Accounts)]

pub struct Trade<'info> {
    #[account[mut]]
    pub user: Signer<'info>,
    #[account[
      mut,
      mint::token_program=token_program
    ]]
    pub mint: InterfaceAccount<'info, Mint>,
    #[account[
      mut,
      associated_token::mint=mint,
      associated_token::token_program=token_program,
      associated_token::authority=user
    ]]
    pub user_ata: InterfaceAccount<'info, TokenAccount>,
    #[account[
      seeds = [b"config"],
      bump = config.bump
    ]]
    pub config: Account<'info, Config>,
    #[account[
      mut,
      associated_token::mint=mint,
      associated_token::token_program=token_program,
      associated_token::authority=config
    ]]
    pub vault: InterfaceAccount<'info, TokenAccount>,
    #[account[
      init_if_needed,
      payer=user,
      space = 8 + Position::INIT_SPACE,
      seeds = [b"user", mint.key().as_ref()],
      bump
    ]]
    pub user_position: Account<'info, Position>,
    /// CHECK: Transfer hook program
    #[account[mut]]
    pub hook_program: UncheckedAccount<'info>,
    /// CHECK: extra account program
    #[account[mut]]
    pub extra_account_meta: UncheckedAccount<'info>,
    /// CHECK: Transfer hook's whitelist program
    #[account[mut]]
    pub whitelist: UncheckedAccount<'info>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl Trade<'_> {
    pub fn deposit(&mut self, amount: u64) -> Result<()> {
        self.user_position.set_inner(Position {
            user: self.user.key(),
            amount: self.user_position.amount + amount,
        });

        token_transfer_with_extra(
            &self.token_program.to_account_info(),
            &self.user_ata.to_account_info(),
            &self.mint.to_account_info(),
            &self.vault.to_account_info(),
            &self.user.to_account_info(),
            &self.extra_account_meta.to_account_info(),
            &self.hook_program.to_account_info(),
            &self.whitelist.to_account_info(),
            amount,
            6,
        )?;

        Ok(())
    }

    pub fn withdraw(&mut self, amount: u64) -> Result<()> {
        require!(
            self.user_position.amount >= amount,
            VaultError::NotEnoughBalance
        );

        self.user_position.set_inner(Position {
            user: self.user.key(),
            amount: self.user_position.amount - amount,
        });

        let seeds: &[&[u8]] = &[
            b"config",           // your static seed
            &[self.config.bump], // your bump, wrapped as byte slice
        ];
        let signer_seeds = &[seeds];

        token_transfer_with_extra_and_signer_seeds(
            &self.token_program.to_account_info(),
            &self.vault.to_account_info(),
            &self.mint.to_account_info(),
            &self.user_ata.to_account_info(),
            &self.config.to_account_info(),
            &self.extra_account_meta.to_account_info(),
            &self.hook_program.to_account_info(),
            &self.whitelist.to_account_info(),
            signer_seeds,
            amount,
            6,
        )?;

        Ok(())
    }
}

pub fn token_transfer_with_extra<'info>(
    token_program: &AccountInfo<'info>,
    from: &AccountInfo<'info>,
    mint: &AccountInfo<'info>,
    to: &AccountInfo<'info>,
    authority: &AccountInfo<'info>,
    extra_account_meta_list: &AccountInfo<'info>,
    hook_program: &AccountInfo<'info>,
    whitelist: &AccountInfo<'info>,
    amount: u64,
    decimals: u8,
) -> Result<()> {
    // Create the list of accounts in order
    let mut accounts = vec![
        AccountMeta::new(*from.key, false),
        AccountMeta::new_readonly(*mint.key, false),
        AccountMeta::new(*to.key, false),
        AccountMeta::new_readonly(*authority.key, true),
    ];
    accounts.push(AccountMeta::new(*extra_account_meta_list.key, false));
    accounts.push(AccountMeta::new(*whitelist.key, false));
    accounts.push(AccountMeta::new(hook_program.key(), false));

    // Build the transfer_checked instruction
    let ix = transfer_checked(
        token_program.key,
        from.key,
        mint.key,
        to.key,
        authority.key,
        &[], // multisigners if any
        amount,
        decimals,
    )?;

    // Manually override accounts of the instruction with full list including extras
    let mut instruction = Instruction {
        program_id: *token_program.key,
        accounts,
        data: ix.data,
    };

    invoke(
        &instruction,
        &[
            from.clone(),
            mint.clone(),
            to.clone(),
            authority.clone(),
            extra_account_meta_list.clone(),
            whitelist.clone(),
            hook_program.clone(),
        ],
    )?;

    Ok(())
}

pub fn token_transfer_with_extra_and_signer_seeds<'info>(
    token_program: &AccountInfo<'info>,
    from: &AccountInfo<'info>,
    mint: &AccountInfo<'info>,
    to: &AccountInfo<'info>,
    authority: &AccountInfo<'info>,
    extra_account_meta_list: &AccountInfo<'info>,
    hook_program: &AccountInfo<'info>,
    whitelist: &AccountInfo<'info>,
    signer_seeds: &[&[&[u8]]],
    amount: u64,
    decimals: u8,
) -> Result<()> {
    // Create the list of accounts in order
    let mut accounts = vec![
        AccountMeta::new(*from.key, false),
        AccountMeta::new_readonly(*mint.key, false),
        AccountMeta::new(*to.key, false),
        AccountMeta::new_readonly(*authority.key, true),
    ];
    accounts.push(AccountMeta::new(*extra_account_meta_list.key, false));
    accounts.push(AccountMeta::new(*whitelist.key, false));
    accounts.push(AccountMeta::new(hook_program.key(), false));

    // Build the transfer_checked instruction
    let ix = transfer_checked(
        token_program.key,
        from.key,
        mint.key,
        to.key,
        authority.key,
        &[], // multisigners if any
        amount,
        decimals,
    )?;

    // Manually override accounts of the instruction with full list including extras
    let mut instruction = Instruction {
        program_id: *token_program.key,
        accounts,
        data: ix.data,
    };

    invoke_signed(
        &instruction,
        &[
            from.clone(),
            mint.clone(),
            to.clone(),
            authority.clone(),
            extra_account_meta_list.clone(),
            whitelist.clone(),
            hook_program.clone(),
        ],
        signer_seeds,
    )?;

    Ok(())
}
