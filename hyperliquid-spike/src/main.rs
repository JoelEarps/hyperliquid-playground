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

#[tokio::main]
async fn main() {
    println!("Hello Joel");
    let client = Client::default();
    let mut info_client = InfoClient::new(Some(client), Some(BaseUrl::Testnet)).await.unwrap();

    // Pass in from config?
    let required_base = "HYPE";
    let required_quote = "USDC";

    let spot_meta = info_client.spot_meta().await.expect("Could not receive spot meta data");

    let extracted_market_indexes = extract_market_index(spot_meta);

    let (sender, mut receiver) = unbounded_channel();

    let subscription_id = info_client
        .subscribe(
            Subscription::L2Book {
                coin: "@107".to_string(),
            },
            sender,
        )
        .await
        .unwrap();

    print!("{}", subscription_id);

    spawn(async move {
        sleep(Duration::from_secs(30)).await;
        println!("Unsubscribing from l2 book data");
        info_client.unsubscribe(subscription_id).await.unwrap()
    });

    // This loop ends when we unsubscribe
    while let Some(Message::L2Book(l2_book)) = receiver.recv().await {
        println!("Received l2 book data: {l2_book:?}");
    }

    println!("Bye Joel");

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
