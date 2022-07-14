use cosmwasm_std::{Coin, StdError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    /// Only authorized entities are allowed to execute
    #[error("Unauthorized")]
    Unauthorized,

    /// If the token has already been claimed
    #[error("token_id already claimed")]
    Claimed,

    #[error("Cannot set approval that is already expired")]
    Expired,

    #[error("Approval not found for: {operator}")]
    ApprovalNotFound { operator: String },

    #[error("Invalid amount. Expected {val:?} received {funds:?}")]
    InvalidAmount { val: Coin, funds: Coin },

    #[error("Following error occured: {val:?}")]
    CustomError { val: String },
    // Add any other custom errors you like here.
    // Look at https://docs.rs/thiserror/1.0.21/thiserror/ for details.
}
