use anchor_lang::prelude::*;

#[error_code]
pub enum MasterChefError {

    #[msg("Math error")] 
    MathError,

    #[msg("MasterChef pool slots are full")] 
    PoolSlotsFull,

    #[msg("Pool not find")]
    PoolNotFind,

    #[msg("Pool already exists")]
    PoolAlreadyExists,

    #[msg("Invalid transfer")]
    InvalidTransfer,

    #[msg("LP token not enough")]
    LPTokenNotEnough,
}