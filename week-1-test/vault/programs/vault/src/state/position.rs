use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]

pub struct Position {
    pub user: Pubkey,
    pub amount: u64,
}
