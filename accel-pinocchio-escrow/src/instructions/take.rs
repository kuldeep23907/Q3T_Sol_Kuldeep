use pinocchio::{
    account_info::AccountInfo,
    instruction::{Seed, Signer},
    msg,
    pubkey::log,
    sysvars::{rent::Rent, Sysvar},
    ProgramResult,
};
use pinocchio_pubkey::derive_address;

use crate::state::Escrow;

pub fn process_take_instruction(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    msg!("Processing Take instruction");

    let [taker, maker, mint_a, mint_b, taker_ata_a, taker_ata_b, maker_ata_b, escrow_ata_a, escrow_account, system_program, token_program, _associated_token_program, _rent_sysvar @ ..] =
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
        let maker_ata_b_state =
            pinocchio_token::state::TokenAccount::from_account_info(&maker_ata_b)?;
        if maker_ata_b_state.owner() != maker.key() {
            return Err(pinocchio::program_error::ProgramError::IllegalOwner);
        }
        if maker_ata_b_state.mint() != mint_b.key() {
            return Err(pinocchio::program_error::ProgramError::InvalidAccountData);
        }
    }

    {
        let taker_ata_a_state =
            pinocchio_token::state::TokenAccount::from_account_info(&taker_ata_a)?;
        if taker_ata_a_state.owner() != taker.key() {
            return Err(pinocchio::program_error::ProgramError::IllegalOwner);
        }
        if taker_ata_a_state.mint() != mint_a.key() {
            return Err(pinocchio::program_error::ProgramError::InvalidAccountData);
        }
    }

    {
        let taker_ata_b_state =
            pinocchio_token::state::TokenAccount::from_account_info(&*taker_ata_b)?;
        if taker_ata_b_state.owner() != taker.key() {
            return Err(pinocchio::program_error::ProgramError::IllegalOwner);
        }
        if taker_ata_b_state.mint() != mint_b.key() {
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
    let amount_to_receive = escrow_state.amount_to_receive();
    let amount_to_give = escrow_state.amount_to_give();

    let bump = [bump.to_le()];
    let seed = [
        Seed::from(b"escrow"),
        Seed::from(maker.key()),
        Seed::from(&bump),
    ];
    let seeds = Signer::from(&seed);

    pinocchio_token::instructions::Transfer {
        from: taker_ata_b,
        to: maker_ata_b,
        authority: taker,
        amount: amount_to_receive,
    }
    .invoke()?;

    pinocchio_token::instructions::Transfer {
        from: escrow_ata_a,
        to: taker_ata_a,
        authority: escrow_account,
        amount: amount_to_give,
    }
    .invoke_signed(&[seeds.clone()])?;

    Ok(())
}
