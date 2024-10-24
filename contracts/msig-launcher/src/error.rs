use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0} was not found in the reply")]
    MissingContract(String),

    #[error("Received code id {code_id} and address {address}")]
    UnknownContract { code_id: u64, address: String },

    #[error("Received: {0}")]
    ReplyError(String),

    #[error("There is an already pending transaction happening, this should never happen in the current CosmWasm context")]
    UnexpectedDoubleTx {},

    #[error("Unauthorized")]
    Unauthorized {},
    // Add any other custom errors you like here.
    // Look at https://docs.rs/thiserror/1.0.21/thiserror/ for details.
}
