use cosmwasm_std::{Coin, StdError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    /// Only authorized entities are allowed to execute.
    #[error("Unauthorized")]
    Unauthorized,

    /// The approvals has expired.
    #[error("Cannot set approval that is already expired")]
    Expired,

    /// operator has already been approved previously.
    #[error("{operator} has already been approved")]
    OperatorApproved { operator: String },

    /// No approval found for given address.
    #[error("Approval not found for: {operator}")]
    ApprovalNotFound { operator: String },

    /// Supplied token id is invalid.
    #[error("token_id: {token_id} does not exist")]
    InvalidToken { token_id: u64 },

    /// Amount sent in not equal to the price of the NFT.
    #[error("Invalid amount. Expected {val:?} received {funds:?}")]
    InvalidAmount { val: Coin, funds: Coin },

    /// Any other error not which has not been covered.
    #[error("Following error occured: {val:?}")]
    CustomError { val: String },
    // Add any other custom errors you like here.
    // Look at https://docs.rs/thiserror/1.0.21/thiserror/ for details.
}
