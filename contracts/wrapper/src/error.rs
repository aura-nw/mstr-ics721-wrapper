use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Not enough funds")]
    NotEnoughFunds {},

    #[error("Expected received not matched")]
    ExpectedReceivedNotMatched {},

    #[error("Collection not allowed")]
    CollectionNotAllowed {},

    #[error("Collection deactivated")]
    CollectionDeactivated {},

    #[error("Token_id {val:?} not owned by sender")]
    NotOwnedBySender { val: String },

    #[error("Custom Error val: {val:?}")]
    CustomError { val: String },
    // Add any other custom errors you like here.
    // Look at https://docs.rs/thiserror/1.0.21/thiserror/ for details.
}
