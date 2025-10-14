use crate::state::Whitelist;
use anchor_lang::prelude::*;
use anchor_spl::{
    token,
    token_2022::{self, MintTo},
    token_interface::{Mint, TokenAccount, TokenInterface},
};
use spl_tlv_account_resolution::{
    account::ExtraAccountMeta, pubkey_data::PubkeyData, seeds::Seed, state::ExtraAccountMetaList,
};
use spl_transfer_hook_interface::instruction::{
    ExecuteInstruction, InitializeExtraAccountMetaListInstruction,
};

#[derive(Accounts)]
pub struct TokenFactory<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        init,
        payer = user,
        mint::decimals = 9,
        mint::authority = user,
        extensions::transfer_hook::authority = user,
        extensions::transfer_hook::program_id = crate::ID,
    )]
    pub mint: InterfaceAccount<'info, Mint>,
    #[account(
        init_if_needed,
        payer=user,
        token::mint = mint,
        token::authority = user,
    )]
    pub source_token_ata: InterfaceAccount<'info, TokenAccount>,
    /// CHECK: ExtraAccountMetaList Account, will be checked by the transfer hook
    #[account(
        init,
        seeds = [b"extra-account-metas", mint.key().as_ref()],
        bump,
        space = ExtraAccountMetaList::size_of(
            extra_account_metas()?.len()
        )?,
        payer = user
    )]
    pub extra_account_meta_list: AccountInfo<'info>,
    // #[account(
    //     seeds = [b"whitelist"],
    //     bump
    // )]
    // pub blocklist: Account<'info, Whitelist>,
    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
}

impl<'info> TokenFactory<'info> {
    pub fn init_mint(&mut self) -> Result<()> {
        msg!("Initializing Transfer Hook...");

        // Get the extra account metas for the transfer hook
        let extra_account_metas = extra_account_metas()?;

        msg!("Extra Account Metas: {:?}", extra_account_metas);
        msg!("Extra Account Metas Length: {}", extra_account_metas.len());

        // initialize ExtraAccountMetaList account with extra accounts
        ExtraAccountMetaList::init::<ExecuteInstruction>(
            &mut self.extra_account_meta_list.try_borrow_mut_data()?,
            &extra_account_metas,
        )?;

        let ctx_accounts = token_2022::MintTo {
            mint: self.mint.to_account_info(),
            to: self.source_token_ata.to_account_info(),
            authority: self.user.to_account_info(),
        };

        let ctx = CpiContext::new(self.token_program.to_account_info(), ctx_accounts);

        token_2022::mint_to(ctx, 1000_000_000_000)?;

        Ok(())
    }
}

pub fn extra_account_metas() -> Result<Vec<ExtraAccountMeta>> {
    Ok(vec![ExtraAccountMeta::new_with_pubkey_data(
        &PubkeyData::AccountData {
            account_index: 5,
            data_index: 5,
        },
        false, // is_signer
        false, // is_writable
    )?])
}
