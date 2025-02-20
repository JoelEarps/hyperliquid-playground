use std::{ops::Deref, sync::{Arc, Mutex}};

use connector_model::{connector::{market_builder::MarketBuilder, market_type::MarketBuilderParameters}, orderbook::{OrderBook, PriceLevel}};

use ethers::signers::LocalWallet;
use hyperliquid_info_client::hyper_liquid_websocket::{HyperLiquidGlobalMarketDataHandler, HyperLiquidWebSocketHandler};
use hyperliquid_rust_sdk::{BaseUrl, InfoClient, Message, Subscription};
use tokio::{
    spawn,
    sync::mpsc::unbounded_channel,
    time::{sleep, self, Duration},
};

mod index_extractor;
use index_extractor::extract_market_index;

mod hyperliquid_info_client;

mod errors;

#[cfg(test)]
mod __fixtures__;

/// Very Rough POC - just proving the point until we implement an MVP and better practices
#[tokio::main]
async fn main() {

    // Key was randomly generated for testing and shouldn't be used with any real funds
    // TODO: Use and integrate a real wallet
    let wallet: LocalWallet = "e908f86dbb4d55ac876378565aafeabc187f6690f046459397b17d9b9a19688e"
        .parse()
        .expect("Custom Error message for wallets");

    // Use passed in market paur to check all market ids and select the relevant one
    // Hard coded as this is the only market pair we need- would be better to pass in from the config.
    let required_market_pair: &str = "HYPE_USDC";

    let mut websocket_handler_under_test = HyperLiquidWebSocketHandler::new().await.unwrap();
    let spot_meta = websocket_handler_under_test.0.info_client.spot_meta().await.expect("Could not receive spot meta data");

    let extracted_market_indexes = extract_market_index(spot_meta);
    if let Some(required_index) = extracted_market_indexes.get(required_market_pair) {
        let mut interval = time::interval(Duration::from_secs(1));
        // Have this in order to keep the thread alive - will figure out the run time issues
        loop {
            interval.tick().await; // Waits for the interval to complete
            println!("Tick: {:?}", tokio::time::Instant::now());
        }
    }
    println!("Bye Joel");
}
