use bigdecimal::ParseBigDecimalError;
use thiserror::Error;

#[derive(Error, Debug)]
pub(crate) enum HyperLiquidOrderBookErrors {
    #[error("Failed to parse bid and quantity")]
    BigDecimalParsingError(#[from] ParseBigDecimalError),

    #[error("Unexpected L2OrderBook Data Structure")]
    InvalidL2OrderBook
}

#[derive(Error, Debug)]
pub(crate) enum HyperLiquidNetworkErrors {
    #[error("Failed to create sdk Info Client")]
    HyperLiquidCommsError(#[from] hyperliquid_rust_sdk::Error)
}