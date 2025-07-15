use anchor_lang::{prelude::*, Bump};

#[account]
#[derive(InitSpace)]
pub struct Escrow {
    pub seed: u64,
    pub userA: Pubkey,
    pub mintA: Pubkey,
    pub mintB: Pubkey,
    pub receive: u64,
    pub bumb: u8,
}
