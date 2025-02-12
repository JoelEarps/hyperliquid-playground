use std::ops::Deref;

use connector_model::orderbook::{OrderBook, PriceLevel};

use hyperliquid_rust_sdk::{BaseUrl, InfoClient, Message, Subscription};
use reqwest::Client;
use tokio::{
    spawn,
    sync::mpsc::unbounded_channel,
    time::{sleep, Duration},
};

mod index_extractor;
use index_extractor::extract_market_index;

mod hyperliquid_info_client;
mod errors;
use hyperliquid_info_client::hyperliquid_orderbook::{HyperLiquidOrderBookData, TestOrderBook};

/// Very Rough POC - just proving the point until we implement an MVP and better practices
#[tokio::main]
async fn main() {
    println!("Hello Joel");
    let client = Client::default();
    let mut info_client = InfoClient::new(Some(client), Some(BaseUrl::Testnet)).await.unwrap();

    // Use passed in market paur to check all market ids and select the relevant one
    // Hard coded as this is the only market pair we need- would be better to pass in from the config.
    let required_market_pair: &str = "HYPE_USDC";
    let spot_meta = info_client.spot_meta().await.expect("Could not receive spot meta data");
    let extracted_market_indexes = extract_market_index(spot_meta);
    let required_index = extracted_market_indexes.get(required_market_pair);
    println!("{:?}", required_index);


    // Use the gathered Market Index to request 
    // Error if no value returned from HashMap
    // Good practice to only operate on the positive
    // Create Web Socket and stream required data
    // 1-1 relationship for now, potentially move it to MarketBuilder and Another Trait maybe? Like websocket
    if let Some(required_index) = extracted_market_indexes.get(required_market_pair) {
        let (sender, mut receiver) = unbounded_channel();
        let subscription_id = info_client
            .subscribe(
                Subscription::L2Book {
                    coin: required_index.market_index.deref().to_string()
                },
                sender,
            )
            .await
            .unwrap();
    
        println!("{}", subscription_id);
    
        spawn(async move {
            sleep(Duration::from_secs(30)).await;
            println!("Unsubscribing from L2 book data");
            info_client.unsubscribe(subscription_id).await.unwrap()
        });
        
        // Abstract while loop using futures if possible
        // For each incoming data
        // Open Questions:
        // 1. What rate do we want to gather data at, is it a long standing connection, do we want to be able to start and stop connections?
        // 2. How do we handle multiple market pairs
        while let Some(Message::L2Book(l2_book)) = receiver.recv().await {
            // Change to orderbook in connector commons when PR is resolved
            // https://gitlab.com/swissborg/defi/connector-commons/-/merge_requests/7
            let hyperliquid_order_book = HyperLiquidOrderBookData::try_from(l2_book.data).expect("Add custom error here for failing");
            let swissborg_order_book = TestOrderBook::new_from_iter(hyperliquid_order_book.bids, hyperliquid_order_book.asks);
            println!("Swissborg model for orderbook: {:?}", swissborg_order_book);

        }
    } else {
        panic!("Need to add this to the error dump");
    }

    // implement Dex API trait

    // Simple spot order example - https://github.com/hyperliquid-dex/hyperliquid-rust-sdk/blob/master/src/bin/spot_order.rs
    // Create or grab a wallet
    // Create an exchange client
    // Create a client order request
    // Place the Order 
    // Handle Response

    // What is the difference between a spot order and a spot transfer, there are also other examples
    // Open Question - does the exchange client handle signature generation, if so how?
    

    println!("Bye Joel");

}
