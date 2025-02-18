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

/// Very Rough POC - just proving the point until we implement an MVP and better practices
#[tokio::main]
async fn main() {
    println!("Hello Joel");

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
        let result = websocket_handler_under_test.0.subscribe_to_market_index(required_index).await.unwrap();
        println!("Here 2: {}", result);

        let global_handler_under_test = HyperLiquidGlobalMarketDataHandler::new(Arc::new(Mutex::new(websocket_handler_under_test.0)), websocket_handler_under_test.1).await;

        let mut interval = time::interval(Duration::from_secs(1));

        // Have this in order to keep the thread alive - will figure out the run time issues
        loop {
            interval.tick().await; // Waits for the interval to complete
            println!("Tick: {:?}", tokio::time::Instant::now());
        }
    }

    // Use Market Index to gather information

    // Build Orderbook Stream to gather market book data
    
    // Use the gathered Market Index to request 
    // Error if no value returned from HashMap
    // Good practice to only operate on the positive
    // Create Web Socket and stream required data
    // 1-1 relationship for now, potentially move it to MarketBuilder and Another Trait maybe? Like websocket
    // if let Some(required_index) = extracted_market_indexes.get(dyn_market_client.required_market_pair) {
    //     // let info_receiver = dyn_market_client.initialise_websocket(required_index).await;
    //     let order_book_stream = dyn_market_client.build_orderbook_stream(MarketBuilderParameters { orders_limit: 15, convertion_params: None}, 1000, true).await;
    //     // Abstract while loop using futures if possible
    //     // For each incoming data
    //     // Open Questions:
    //     // 1. What rate do we want to gather data at, is it a long standing connection, do we want to be able to start and stop connections?
    //     // 2. How do we handle multiple market pairs
    // } else {
    //     panic!("Need to add this to the error dump, no market data");
    // }

    // implement Dex API trait

    // Simple spot order example - https://github.com/hyperliquid-dex/hyperliquid-rust-sdk/blob/master/src/bin/spot_order.rs
    // Create or grab a wallet ✅
    // Create an exchange client ✅
    // Create a client order request 
    // Place the Order 
    // Handle Response

    println!("Bye Joel");
}
