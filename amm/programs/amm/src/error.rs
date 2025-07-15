use anchor_lang::prelude::*;

#[error_code]
pub enum AmmError {
    #[msg("Pool locked")]
    PoolLocked,
    #[msg("Invalid amount")]
    InvalidAmount,
    #[msg("Slippage Exceeded")]
    SlippageExceeded,
}
