use std::sync::{Arc, Mutex};

use connector_utils::sync::{job_handler::JobHandler, notifier::Notifier};
use dashmap::DashMap;
use reqwest::Client;

use crate::{errors::HyperLiquidNetworkErrors, index_extractor::MarketIndexData};
use crate::hyperliquid_info_client::hyperliquid_orderbook::{HyperLiquidOrderBookData, TestOrderBook};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use hyperliquid_rust_sdk::{L2BookData, BaseUrl, ClientLimit, ClientOrder, ClientOrderRequest, ExchangeClient, InfoClient, Message, Subscription};
// We want to seperate the integration of the WebSocket, mpsc channel
// The use of JobHandler is to make sure that only one instance of the WebSocket connection exists and that we can handle multiple subscriptions to it.
pub(crate) struct HyperLiquidWebSocketHandler {
    pub info_client: InfoClient,
    market_sender: UnboundedSender<Message>,
}

type Test = (HyperLiquidWebSocketHandler, UnboundedReceiver<Message>);

impl HyperLiquidWebSocketHandler {

    pub(crate) async fn new() -> Result<Test, ()>{
        let client = Client::default();
        let info_client = InfoClient::new(Some(client), Some(BaseUrl::Testnet)).await.unwrap();
        let (snd,rcv) = unbounded_channel::<Message>();

       Ok((Self{
        info_client, 
        market_sender:snd}, 
        rcv))
    }

    pub async fn subscribe_to_market_index(&mut self, required_index: &MarketIndexData) -> Result<(), HyperLiquidNetworkErrors>{
        let _ = self.info_client
        .subscribe(
            Subscription::L2Book {
                coin: required_index.market_index.clone()
            },
            self.market_sender.clone(),
        )
        .await
        ?;
    Ok(())
    }
}

pub type BookId =  String;

/// Global Handler 
pub struct HyperLiquidGlobalMarketDataHandler {
    job_handler: JobHandler,
    market_data_cache: DashMap<BookId, BookNotifier>,
    ws_handler: Arc<Mutex<HyperLiquidWebSocketHandler>>,
}

/// Responsible for pushing Data from the WebSocket to the global cache
impl HyperLiquidGlobalMarketDataHandler{

        pub async fn new(ws_handler: Arc<Mutex<HyperLiquidWebSocketHandler>>, market_data_receiver:UnboundedReceiver<Message>)-> Arc<Self> {

        let global_handler = Arc::new( Self {
            job_handler: JobHandler::new(),
            market_data_cache: Default::default(),
            ws_handler
        });

        global_handler.spawn_market_data_consumer(market_data_receiver).await;

        global_handler  
        }

        
        async fn spawn_market_data_consumer(self: &Arc<Self>, mut market_data_receiver: UnboundedReceiver<Message>) {
            println!("Here");
            let global_handler_clone  = self.clone();
            let websocket_job = tokio::spawn(async move {
                while let Some(Message::L2Book(l2_book)) = market_data_receiver.recv().await {
                    // Change to orderbook in connector commons when PR is resolved
                    // https://gitlab.com/swissborg/defi/connector-commons/-/merge_requests/7
                    let hyperliquid_order_book = HyperLiquidOrderBookData::try_from(l2_book.data).expect("Add custom error here for failing");
                    let swissborg_order_book = TestOrderBook::new_from_iter(hyperliquid_order_book.bids, hyperliquid_order_book.asks);
                    println!("Swissborg model for orderbook: {:?}", swissborg_order_book);
                    
                    // Push this data to a DashMap and check output
                }
                Ok(())
            });

        self.job_handler.replace(websocket_job);
        }
}



pub struct BookNotifier{
    latest_book : TestOrderBook,
    notifier:  Notifier, // from connector commons
}

#[cfg(test)]
mod tests
{
    use std::str::FromStr;
    use ethers::types::H128;
    use super::*;

    // Provide test url for websocket? 
    #[tokio::test]
    async fn test_websocket_handler(){
        let required_index = &MarketIndexData {
            market_index: "HYPE_USDC".to_string(),
            token_id: H128::from_str("0xbaf265ef389da684513d98d68edf4eae").unwrap()
        };
        let mut websocket_handler_under_test = HyperLiquidWebSocketHandler::new().await.unwrap();
        websocket_handler_under_test.0.subscribe_to_market_index(required_index).await.unwrap();

        let global_handler_under_test = HyperLiquidGlobalMarketDataHandler::new(Arc::new(Mutex::new(websocket_handler_under_test.0)), websocket_handler_under_test.1).await;

    }
}