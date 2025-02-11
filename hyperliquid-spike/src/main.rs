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
mod hyperliquid_orderbook;
mod errors;

#[tokio::main]
async fn main() {
    println!("Hello Joel");
    let client = Client::default();
    let mut info_client = InfoClient::new(Some(client), Some(BaseUrl::Testnet)).await.unwrap();

    // Pass in from config?
    let required_market_pair: &str = "HYPE_USDC";

    let spot_meta = info_client.spot_meta().await.expect("Could not receive spot meta data");

    let extracted_market_indexes = extract_market_index(spot_meta);

    let required_index = extracted_market_indexes.get(required_market_pair);
    println!("{:?}", required_index);

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
        while let Some(Message::L2Book(l2_book)) = receiver.recv().await {
            println!("Received L2 book data: {l2_book:?}");
        
        }
    } else {
        panic!("Need to add this to the error dump");
    }

    

    // println!("Bye Joel");

    // OrderBook:
}

// struct  NativeBookLevel {
//     pub px: String,
//     pub sz: String,
//     pub n: u64,
// }

// impl TryFrom<Vec<Vec<NativeBookLevel>>> for PriceLevel {

// }

// // impl TryFrom<BookLevel> for PriceLevel {
// //     type Error = dyn std::error::Error;

// //     fn try_from(value: BookLevel) -> Result<Self, Self::Error> {
        
// //     }
// // }
