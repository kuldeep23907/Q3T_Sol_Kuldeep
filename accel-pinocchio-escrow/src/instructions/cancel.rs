use std::ops::{Add, Sub};

use pinocchio::{
    account_info::AccountInfo,
    instruction::{Seed, Signer},
    msg,
    pubkey::{log, Pubkey},
    sysvars::{rent::Rent, Sysvar},
    ProgramResult,
};
use pinocchio_pubkey::derive_address;

use crate::state::Escrow;

pub fn process_cancel_instruction(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    msg!("Processing Cancel instruction");

    let [maker, mint_a, maker_ata_a, escrow_ata_a, escrow_account, system_program, token_program, _associated_token_program, _rent_sysvar @ ..] =
        accounts
    else {
        return Err(pinocchio::program_error::ProgramError::NotEnoughAccountKeys);
    };

    {
        let escrow_ata_a_state =
            pinocchio_token::state::TokenAccount::from_account_info(&escrow_ata_a)?;
        if escrow_ata_a_state.owner() != escrow_account.key() {
            return Err(pinocchio::program_error::ProgramError::IllegalOwner);
        }
        if escrow_ata_a_state.mint() != mint_a.key() {
            return Err(pinocchio::program_error::ProgramError::InvalidAccountData);
        }
    }

    {
        let maker_ata_a_state =
            pinocchio_token::state::TokenAccount::from_account_info(&maker_ata_a)?;
        if maker_ata_a_state.owner() != maker.key() {
            return Err(pinocchio::program_error::ProgramError::IllegalOwner);
        }
        if maker_ata_a_state.mint() != mint_a.key() {
            return Err(pinocchio::program_error::ProgramError::InvalidAccountData);
        }
    }
    if escrow_account.owner() != &crate::ID {
        return Err(pinocchio::program_error::ProgramError::IllegalOwner);
    }

    let bump = data[0];
    let seed = [b"escrow".as_ref(), maker.key().as_slice(), &[bump]];
    let seeds = &seed[..];

    let escrow_account_pda = derive_address(&seed, None, &crate::ID);
    log(&escrow_account_pda);
    log(&escrow_account.key());
    assert_eq!(escrow_account_pda, *escrow_account.key());

    let escrow_state = Escrow::from_account_info(escrow_account)?;

    if escrow_state.maker() != *maker.key() {
        return Err(pinocchio::program_error::ProgramError::IllegalOwner);
    }

    let amount_to_give = escrow_state.amount_to_give();

    let bump = [bump.to_le()];
    let seed = [
        Seed::from(b"escrow"),
        Seed::from(maker.key()),
        Seed::from(&bump),
    ];
    let seeds = Signer::from(&seed);

    pinocchio_token::instructions::Transfer {
        from: escrow_ata_a,
        to: maker_ata_a,
        authority: escrow_account,
        amount: amount_to_give,
    }
    .invoke_signed(&[seeds.clone()])?;

    // Move lamports to maker
    let lamports_to_transfer = escrow_account.lamports();

    let mut lamport_maker_mut = maker.try_borrow_mut_lamports().unwrap();
    *lamport_maker_mut += lamports_to_transfer;

    {
        let mut lamport_escrow_mut = escrow_account.try_borrow_mut_lamports().unwrap();
        *lamport_escrow_mut = 0; // zero escrow lamports
    }

    // Close account signaling (depends on Pinocchio API, may be manual)
    escrow_account.close()?;

    Ok(())
}
