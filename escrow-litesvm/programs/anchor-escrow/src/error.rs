use anchor_lang::prelude::*;

#[error_code]
pub enum EscrowError {
    #[msg("Take time yet to pass")]
    NeedToWait,
    #[msg("Amount is too big")]
    AmountTooBig,
}
