use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct StakeAccount {
    pub stake_token: Pubkey,
    pub owner: Pubkey,
    pub staked_at: i64,
    pub bump: u8,
}
