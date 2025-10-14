use anchor_lang::prelude::*;

#[account]
pub struct Whitelist {
    pub allowed: bool,
    pub bump: u8,
}
