use anchor_lang::prelude::*;

#[error_code]
pub enum VaultError {
    #[msg("Not enough token to withdraw")]
    NotEnoughBalance,
    #[msg("not admin")]
    Unauthorized,
}
