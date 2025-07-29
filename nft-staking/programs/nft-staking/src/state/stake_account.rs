use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct StakeAccount {
    pub stake_token: PubKey,
    pub owner: PubKey,
    pub staked_at: i64,
    pub bump: u8,
}
