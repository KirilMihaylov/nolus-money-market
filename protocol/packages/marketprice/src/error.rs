use std::convert::Infallible;

use thiserror::Error;

use sdk::cosmwasm_std::StdError;

#[derive(Error, Debug, PartialEq)]
pub enum PriceFeedsError {
    #[error("[Market Price; Feeds] {0}")]
    Std(#[from] StdError),

    #[error("[Market Price; Feeds] No price")]
    NoPrice(),

    #[error("[Market Price; Feeds] {0}")]
    FromInfallible(#[from] Infallible),

    #[error("[Market Price; Feeds] Configuration error: {0}")]
    Configuration(String),

    #[error("[Market Price; Feeds] {0}")]
    Currency(#[from] currency::error::Error),

    #[error("[Market Price; Feeds] {0}")]
    Finance(#[from] finance::error::Error),

    #[error("[Market Price; Feeds] {0}")]
    FeedsRetrieve(StdError),

    #[error("[Market Price; Feeds] {0}")]
    FeedRead(StdError),

    #[error("[Market Price; Feeds] {0}")]
    FeedPush(StdError),

    #[error("[Market Price; Feeds] {0}")]
    FeedRemove(StdError),
}

pub(crate) fn config_error_if(check: bool, msg: &str) -> Result<(), PriceFeedsError> {
    if check {
        Err(PriceFeedsError::Configuration(msg.into()))
    } else {
        Ok(())
    }
}
