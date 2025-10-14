#![allow(deprecated)] // for no warnings
#[allow(unexpected_cfgs)]
use anchor_lang::prelude::*;
use anchor_spl::token_2022::spl_token_2022::{
    extension::{
        transfer_hook::TransferHookAccount, BaseStateWithExtensionsMut, PodStateWithExtensionsMut,
    },
    pod::PodAccount,
};
use anchor_spl::token_interface::{Mint, TokenAccount};
use spl_tlv_account_resolution::solana_pubkey::Pubkey as TlvPubkey;
use spl_tlv_account_resolution::{
    account::ExtraAccountMeta, seeds::Seed, state::ExtraAccountMetaList,
};
use spl_transfer_hook_interface::instruction::ExecuteInstruction;
use spl_transfer_hook_interface::instruction::TransferHookInstruction;
use std::cell::RefMut;

declare_id!("CyusCQe7fFM4BxiqEbs5Wj7YroPuXSHtMCXy4NgZvT5v");

#[derive(Accounts)]
pub struct InitializeExtraAccountMetaList<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    /// CHECK: Your mint with transfer hook extension
    pub mint: InterfaceAccount<'info, Mint>,
    /// CHECK: Your mint with transfer hook extension
    #[account(
        init,
        payer = payer,
        seeds = [
            b"extra-account-metas",
            mint.key().as_ref(),
        ],
        bump,
        space =  ExtraAccountMetaList::size_of(
            InitializeExtraAccountMetaList::extra_account_metas()?.len()
        ).unwrap()
    )]
    pub extra_account_meta_list: AccountInfo<'info>,
    #[account(init, seeds = [b"whitelist"], bump, payer = payer, space = 200)]
    pub whitelist: Account<'info, WhiteList>,
    pub system_program: Program<'info, System>,
}

// Define extra account metas to store on extra_account_meta_list account
impl<'info> InitializeExtraAccountMetaList<'info> {
    pub fn extra_account_metas() -> Result<Vec<ExtraAccountMeta>> {
        Ok(vec![ExtraAccountMeta::new_with_seeds(
            &[Seed::Literal {
                bytes: b"whitelist".to_vec(),
            }],
            false, // is_signer
            false, // is_writable
        )?])
    }
}

#[derive(Accounts)]
pub struct TransferHook<'info> {
    #[account(token::mint = mint, token::authority = owner)]
    pub source_token: InterfaceAccount<'info, TokenAccount>,
    pub mint: InterfaceAccount<'info, Mint>,
    #[account(token::mint = mint)]
    pub destination_token: InterfaceAccount<'info, TokenAccount>,
    /// CHECK: Authority validation via Token2022 program
    pub owner: UncheckedAccount<'info>,
    /// CHECK: Token extension supplies this
    #[account(seeds = [b"extra-account-metas", mint.key().as_ref()], bump)]
    pub extra_account_meta_list: UncheckedAccount<'info>,
    #[account(seeds = [b"whitelist"], bump)]
    pub whitelist: Account<'info, WhiteList>,
}

#[derive(Accounts)]
pub struct AddToWhiteList<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    pub mint: InterfaceAccount<'info, Mint>,
    /// CHECK: New account to add to white list
    #[account()]
    pub user: AccountInfo<'info>,
    #[account(
        mut,
        seeds = [b"whitelist"],
        bump
    )]
    pub whitelist: Account<'info, WhiteList>,
}

#[program]
pub mod restricted_transfer_hook {
    use super::*;

    pub fn initialize_extra_account_meta_list(
        ctx: Context<InitializeExtraAccountMetaList>,
    ) -> Result<()> {
        msg!("i m here in init");

        // set authority field on white_list account as payer address
        ctx.accounts.whitelist.authority = ctx.accounts.payer.key();

        let extra_account_metas = InitializeExtraAccountMetaList::extra_account_metas()?;

        // initialize ExtraAccountMetaList account with extra accounts
        ExtraAccountMetaList::init::<ExecuteInstruction>(
            &mut ctx.accounts.extra_account_meta_list.try_borrow_mut_data()?,
            &extra_account_metas,
        )?;

        Ok(())
    }

    pub fn transfer_hook(ctx: Context<TransferHook>, _amount: u64) -> Result<()> {
        // Fail this instruction if it is not called from within a transfer hook
        check_is_transferring(&ctx)?;

        let contains = ctx
            .accounts
            .whitelist
            .whitelist
            .iter()
            .any(|s: &Position| s.user == ctx.accounts.owner.key());

        if !contains {
            return Err((ErrorCode::NotWhitelisted).into());
        } else {
            let cp = ctx
                .accounts
                .whitelist
                .whitelist
                .iter()
                .find(|position| position.user == ctx.accounts.owner.key())
                .unwrap();
            if cp.amount < _amount {
                return Err((ErrorCode::NotWhitelisted).into());
            }
        }

        msg!("Account in white list, all good!");

        Ok(())
    }

    pub fn add_to_whitelist(ctx: Context<AddToWhiteList>, amount: u64) -> Result<()> {
        if ctx.accounts.whitelist.authority != ctx.accounts.signer.key() {
            panic!("Only the authority can add to the white list!");
        }

        let contains = ctx
            .accounts
            .whitelist
            .whitelist
            .iter()
            .any(|s: &Position| s.user == ctx.accounts.user.key());

        if contains {
            return Err((ErrorCode::AlreadyWhitelisted).into());
        }

        ctx.accounts.whitelist.whitelist.push(Position {
            user: ctx.accounts.user.key(),
            amount: amount,
        });
        msg!(
            "New account white listed! {0}",
            ctx.accounts.user.key().to_string()
        );
        msg!(
            "White list length! {0}",
            ctx.accounts.whitelist.whitelist.len()
        );

        Ok(())
    }

    /// Fallback function to handle the transfer hook instruction.
    pub fn fallback<'a>(
        program_id: &Pubkey,
        accounts: &'a [AccountInfo<'a>],
        data: &[u8],
    ) -> Result<()> {
        let instruction = TransferHookInstruction::unpack(data)?;
        match instruction {
            TransferHookInstruction::Execute { amount } => {
                let amount = amount.to_le_bytes();
                __private::__global::transfer_hook(program_id, accounts, &amount)
            }
            _ => Err(ProgramError::InvalidInstructionData.into()),
        }
    }
}

fn check_is_transferring(ctx: &Context<TransferHook>) -> Result<()> {
    msg!("i m here in check");

    let source_token_info = ctx.accounts.source_token.to_account_info();
    let mut account_data_ref: RefMut<&mut [u8]> = source_token_info.try_borrow_mut_data()?;
    let mut account = PodStateWithExtensionsMut::<PodAccount>::unpack(*account_data_ref)?;
    let account_extension = account.get_extension_mut::<TransferHookAccount>()?;

    if !bool::from(account_extension.transferring) {
        return err!(ErrorCode::CurrentlyNotTransferring);
    }

    Ok(())
}

#[account]
pub struct WhiteList {
    pub authority: Pubkey,
    pub whitelist: Vec<Position>,
}

#[account]
pub struct Position {
    pub user: Pubkey,
    pub amount: u64,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Token is not listed yet")]
    CurrentlyNotTransferring,
    #[msg("User not whitelisted")]
    NotWhitelisted,
    #[msg("User whitelisted")]
    AlreadyWhitelisted,
}
