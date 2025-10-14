use std::cell::RefMut;

use crate::state::Whitelist;
use anchor_lang::prelude::*;
use anchor_lang::solana_program::instruction::Instruction;
use anchor_lang::solana_program::program::invoke;
use anchor_spl::token_2022::spl_token_2022::instruction::transfer_checked;
use anchor_spl::{
    token_2022::spl_token_2022::{
        extension::{
            transfer_hook::TransferHookAccount, BaseStateWithExtensionsMut,
            PodStateWithExtensionsMut,
        },
        pod::PodAccount,
    },
    token_interface::{Mint, TokenAccount, TokenInterface},
};

#[derive(Accounts)]
pub struct TransferToken<'info> {
    #[account(
        init_if_needed,
        payer=owner,
        token::mint = mint,
        token::authority = owner,
    )]
    pub source_token: InterfaceAccount<'info, TokenAccount>,
    pub mint: InterfaceAccount<'info, Mint>,
    #[account(
        init_if_needed,
        payer=owner,
        token::mint = mint,
        token::authority = owner,    )]
    pub destination_token: InterfaceAccount<'info, TokenAccount>,
    /// CHECK: owner account
    #[account[mut]]
    pub owner: AccountInfo<'info>,
    /// CHECK: ExtraAccountMetaList Account,
    #[account(
        seeds = [b"extra-account-metas", mint.key().as_ref()],
        bump
    )]
    pub extra_account_meta_list: UncheckedAccount<'info>,
    #[account(
        seeds = [b"whitelist",  owner.key().as_ref()],
        bump,
    )]
    pub whitelist: Account<'info, Whitelist>,
    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
}

impl<'info> TransferToken<'info> {
    /// This function is called when the transfer hook is executed.
    pub fn transfer_token(&mut self, amount: u64) -> Result<()> {
        // Create the list of accounts in order
        let mut accounts = vec![
            AccountMeta::new(self.source_token.key(), false),
            AccountMeta::new_readonly(self.mint.key(), false),
            AccountMeta::new(self.destination_token.key(), false),
            AccountMeta::new_readonly(self.owner.key(), true),
            // AccountMeta::new_readonly(*token_program.key, false),
        ];
        accounts.push(AccountMeta::new(self.extra_account_meta_list.key(), false));
        accounts.push(AccountMeta::new(self.whitelist.key(), false));
        // accounts.push(AccountMeta::new(*whitelist.key, false));
        // accounts.push(AccountMeta::new(hook_program.key(), false));

        // Build the transfer_checked instruction
        let ix = transfer_checked(
            &self.token_program.key(),
            &self.source_token.key(),
            &self.mint.key(),
            &self.destination_token.key(),
            &self.owner.key(),
            &[], // multisigners if any
            amount,
            9,
        )?;

        // Manually override accounts of the instruction with full list including extras
        let mut instruction = Instruction {
            program_id: self.token_program.key(),
            accounts,
            data: ix.data,
        };

        invoke(
            &instruction,
            &[
                self.source_token.to_account_info(),
                self.mint.to_account_info(),
                self.destination_token.to_account_info(),
                self.owner.to_account_info(),
                // token_program.clone(),
                self.extra_account_meta_list.to_account_info(),
                self.whitelist.to_account_info(),
            ],
        )?;

        Ok(())
    }
}
