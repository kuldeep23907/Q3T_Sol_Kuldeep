use crate::state::*;
use anchor_instruction_sysvar::Ed25519InstructionSignatures;
use anchor_lang::solana_program::blake3::hash;
use anchor_lang::solana_program::sysvar::instructions::load_instruction_at_checked;
use anchor_lang::solana_program::{self, ed25519_program};
use anchor_lang::{prelude::*, solana_program::system_instruction::transfer};

pub const HOUSE_EDGE: u16 = 150;

#[derive(Accounts)]
pub struct ResolveBet<'info> {
    #[account[mut]]
    pub house: Signer<'info>,
    /// CHECK: Safe
    #[account[]]
    pub player: UncheckedAccount<'info>,
    #[account[
      mut,
      close=player,
      has_one=player,
      seeds = [b"bet", vault.key().as_ref(), bet.seed.to_le_bytes().as_ref()],
      bump = bet.bump
    ]]
    pub bet: Account<'info, Bet>,
    #[account[
      mut,
      seeds = [b"vault", house.key().as_ref()],
      bump
    ]]
    pub vault: SystemAccount<'info>,
    #[account[
      address = solana_program::sysvar::instructions::ID,
    ]]
    pub instruction_sysvar: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

impl<'info> ResolveBet<'info> {
    pub fn verify_sig(&mut self, sig: &[u8]) -> Result<()> {
        let ix =
            load_instruction_at_checked(0, &self.instruction_sysvar.to_account_info()).unwrap();
        require_keys_eq!(
            ix.program_id,
            ed25519_program::ID,
            DiceError::Ed25519Program
        );

        require_eq!(ix.accounts.len(), 0, DiceError::Ed25519Program);

        let signatures = Ed25519InstructionSignatures::unpack(&ix.data)?.0;
        require_eq!(signatures.len(), 1, DiceError::Ed25519Program);

        let siganture = &signatures[0];

        require!(siganture.is_verifiable, DiceError::EEd25519Programd);

        require_keys_eq!(
            siganture.public_key.ok_or(DiceError::EEd25519Programd)?,
            self.house.key(),
            DiceError::EEd25519Programd
        );

        require!(
            &siganture
                .signature
                .ok_or(DiceError::EEd25519Programd)?
                .eq(sig),
            DiceError::EEd25519Programd
        );

        require!(
            &siganture
                .message
                .as_ref()
                .ok_or(DiceError::EEd25519Programd)?
                .eq(&self.bet.to_slice()),
            DiceError::EEd25519Programd
        );

        Ok(())
    }
    pub fn resolve_bet(&mut self, bumps: &ResolveBetBumps, sig: &[u8]) -> Result<()> {
        let hash = hash(sig).to_bytes();
        let mut hash_16: [u8; 16] = [0; 16];
        hash_16.copy_from_slice(&hash[0..16]);

        let lower = u128::from_le_bytes(hash_16);

        hash_16.copy_from_slice(&hash[16..32]);

        let upper = u128::from_le_bytes(hash_16);

        let roll = lower.wrapping_add(upper).wrapping_rem(100) as u8 + 1;

        if self.bet.roll > roll {

            //payout = (10000-house_edge)/target/100
            // let payout = [(self.bet.amount as u128).checked_mul(10000 - HOUSE_EDGE as u128).ok_or(DiceError::Math)?
            // ]
            // pay SOL to player from vault using seeds
        }

        Ok(())
    }
}
