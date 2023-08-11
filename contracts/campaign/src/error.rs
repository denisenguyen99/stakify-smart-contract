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

    #[error("Max 3 years since start date")]
    LimitStartDate {},

    #[error("Limit Character")]
    LimitCharacter {},

    #[error("Invalid LockupTerm")]
    InvalidLockupTerm {},

    #[error("Insufficient balance")]
    InsufficientBalance {},

    #[error("Too many token ids")]
    TooManyTokenIds {},

    #[error("Invalid time to update")]
    InvalidTimeToUpdate {},

    #[error("Invalid time to stake nft")]
    InvalidTimeToStakeNft {},

    #[error("Not available for unstake")]
    NotAvailableForUnStake {},

    #[error("Invalid time to add reward")]
    InvalidTimeToAddReward {},

    #[error("Already exist")]
    AlreadyExist {},

}
