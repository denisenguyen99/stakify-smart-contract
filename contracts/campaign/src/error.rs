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

    #[error("Too many token ids")]
    TooManyTokenIds {},

    #[error("Already exist")]
    AlreadyExist {},

}
