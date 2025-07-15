#![allow(deprecated)] // for no warnings
#[allow(unexpected_cfgs)]
pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;
use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("5WRBcnisPTKfeiAdUdpHyQerJhw28H7ujJgHKDhNhVbX");

#[program]
pub mod escrow {
    use super::*;

    pub fn initialize(ctx: Context<Make>) -> Result<()> {
        make::make(ctx)
    }
}
