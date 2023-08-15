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

    #[error("Invalid time to unstake")]
    InvalidTimeToUnStake {},

    #[error("Invalid time to add reward")]
    InvalidTimeToAddReward {},

    #[error("Invalid time to withdraw reward")]
    InvalidTimeToWithdrawReward {},

    #[error("Already exist")]
    AlreadyExist {},

}
