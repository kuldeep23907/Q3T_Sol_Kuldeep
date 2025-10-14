use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace, Debug)]
pub struct EscrowInterval {
    pub seed: u64,
    pub interval: u64,
    pub maker: Pubkey,
    pub mint_a: Pubkey,
    pub mint_b: Pubkey,
    pub receive: u64,
    pub bump: u8,
}
