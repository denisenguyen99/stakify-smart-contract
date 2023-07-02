use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Token_id {token_id:?} is unauthorized")]
    NotOwner { token_id: String },

    #[error("Invalid funds")]
    InvalidFunds {},

    #[error("Insufficient balance")]
    InsufficientBalance {},

    #[error("Too many token ids")]
    TooManyTokenIds {},

    #[error("Not available for update")]
    NotAvailableForUpdate {},

    #[error("Not available for staking")]
    NotAvailableForStaking {},

    #[error("Not available for unstake")]
    NotAvailableForUnStake {},

    #[error("Not available for add reward")]
    NotAvailableForAddReward {},

    #[error("Already exist")]
    AlreadyExist {},

}
