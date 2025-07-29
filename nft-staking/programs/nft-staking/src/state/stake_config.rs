use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct StakeConfig {
    pub reward_per_token: u32,
    pub max_stake: u32,
    pub freeze_period: u32,
    pub rewards_bump: u8,
    pub bump: u8,
}
