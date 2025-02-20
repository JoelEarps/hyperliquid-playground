use ethers::abi::ethereum_types::H128;
use hyperliquid_rust_sdk::Message;
use tokio::sync::mpsc::UnboundedReceiver;

use crate::HyperLiquidWebSocketHandler;


#[derive(Debug, Clone)]
pub(crate) struct RequiredTokenInfo {
    pub name: String,
    pub index: usize,
    pub token_id: H128,
}

/// A type alias for the tuple returned for creating a new HyperLiquidWebSocketHandler
pub(super) type HandlerConstructorResult = (HyperLiquidWebSocketHandler, UnboundedReceiver<Message>);

/// A type alias used for referencing the market pair data being gathered, often used for subscribing
pub(super) type BookId =  String;