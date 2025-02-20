
use std::sync::Arc;
use tokio::sync::Mutex;

use connector_utils::sync::{job_handler::JobHandler, notifier::{Notifier, Notified}};
use dashmap::DashMap;
use reqwest::Client;

use crate::errors::HyperLiquidNetworkErrors;
use crate::hyperliquid_info_client::hyperliquid_orderbook::{HyperLiquidOrderBookData, TestOrderBook};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use hyperliquid_rust_sdk::{BaseUrl, InfoClient, Message, Subscription};


pub(crate) struct HyperLiquidWebSocketHandler {
    pub info_client: InfoClient,
    pub market_sender: UnboundedSender<Message>,
}

type HandlerResult = (HyperLiquidWebSocketHandler, UnboundedReceiver<Message>);

impl HyperLiquidWebSocketHandler {
    pub(crate) async fn new() -> Result<HandlerResult, ()> {
        // TODO: Can we pass in the same client to be used in the exchange and info client - any negatives to this?
        let client = Client::default();
        let info_client = InfoClient::new(Some(client), Some(BaseUrl::Testnet)).await.unwrap();
        let (snd,rcv) = unbounded_channel::<Message>();

       Ok((Self{
        info_client, 
        market_sender: snd}, 
        rcv))
    }

    async fn subscribe_to_market_index(&mut self, market_index: &BookId) -> Result<u32, HyperLiquidNetworkErrors>{
        let sub_id = self.info_client
        .subscribe(
            Subscription::L2Book {
                coin: market_index.clone()
            },
            self.market_sender.clone(),
        )
        .await
        ?;

        Ok(sub_id)
    }
}

/// A type alias used for referencing the market pair data being gathered, often used for subscribing
pub type BookId =  String;

/// Global Handler 
pub struct HyperLiquidGlobalMarketDataHandler {
    job_handler: JobHandler,
    market_data_cache: DashMap<BookId, BookNotifier>,
    ws_handler: Arc<Mutex<HyperLiquidWebSocketHandler>>,
}

/// Responsible for pushing Data from the WebSocket to the global cache
impl HyperLiquidGlobalMarketDataHandler {

        pub async fn new(ws_handler: Arc<Mutex<HyperLiquidWebSocketHandler>>, market_data_receiver:UnboundedReceiver<Message>)-> Arc<Self> {

        let global_handler = Arc::new( Self {
            job_handler: JobHandler::new(),
            market_data_cache: Default::default(),
            ws_handler,
        });

        global_handler.spawn_market_data_consumer(market_data_receiver).await;

        global_handler  
        }

        /// Mimicking 
        pub async fn subscribe_to_market(&self, market_id: &BookId) -> Result<(), HyperLiquidNetworkErrors> {
            if self.market_data_cache.contains_key(market_id) { return Ok(()) }
            else {
                self.ws_handler.lock().await.subscribe_to_market_index(&market_id).await?;
                Ok(())
            }
        }

        /// Look through concurrent DashMap and return the related notifier for a market
        /// In the overall view of the system this is used by the order_book stream in the HyperLiquidAPI client to subscribe and collect the latest orderbook data
        pub fn get_notified_for_market(&self, market_id: &BookId ) -> Option<Notified> {
            self.market_data_cache.get(market_id).map(|item| item.value().notifier.new_notified())
        }

        async fn spawn_market_data_consumer(self: &Arc<Self>, mut market_data_receiver: UnboundedReceiver<Message>) {
            let global_handler_clone  = self.clone();
            let websocket_job = tokio::spawn(async move {
                    while let Some(Message::L2Book(l2_book)) = market_data_receiver.recv().await {
                    let map_key = l2_book.data.coin.clone();
                    let hyperliquid_order_book = HyperLiquidOrderBookData::try_from(l2_book.data).expect("Add custom error here for failing");
                    // Change to orderbook in connector commons when PR is resolved
                    // https://gitlab.com/swissborg/defi/connector-commons/-/merge_requests/7
                    let swissborg_order_book = TestOrderBook::new_from_iter(hyperliquid_order_book.bids, hyperliquid_order_book.asks);
                    match global_handler_clone.market_data_cache.entry(map_key) {
                        dashmap::Entry::Occupied(mut current_entry) => {
                            let values = current_entry.get_mut();
                            values.latest_book = swissborg_order_book;
                            values.notifier.notify();
                        },
                        // Better to update through a Mutex or to grab all at once from DashMap? Performance wise I would assume here?
                        dashmap::Entry::Vacant(vacant_entry) => {
                            let new_entry = vacant_entry.insert(BookNotifier {latest_book: swissborg_order_book, notifier: Notifier::new()});
                            new_entry.notifier.notify();
                        }
                    }
                }
                Ok(())
            });
        self.job_handler.replace(websocket_job);
        }

        // TODO: Change Market ID to just string 
        pub fn get_orderbook_data_for_market(&self, market_index: &BookId) -> Option<TestOrderBook>
        {
            if let Some(data) = self.market_data_cache.get(market_index) {
                Some(data.latest_book.clone())
            } else {
                None
            }
        }
}

pub struct BookNotifier {
    latest_book : TestOrderBook,
    notifier:  Notifier,
}

#[cfg(test)]
mod tests
{
    use std::str::FromStr;
    use bigdecimal::BigDecimal;
    use connector_model::{orderbook::PriceLevel, pricing::{Quantity, Rate}};
    use ethers::types::H128;
    use tokio::{
        sync::mpsc::unbounded_channel,
        time::{sleep, self, Duration},
    };

    use super::*;
    use crate::__fixtures__::orderbook_fixtures::create_test_fixture;
    use crate::index_extractor::MarketIndexData;

    // Provide test url for websocket? use rstest to create failure and correct parameterised tests
    #[tokio::test]
    async fn test_websocket_handler(){
        let required_index = &MarketIndexData {
            market_index: "@1035".to_string(),
            token_id: H128::from_str("0xbaf265ef389da684513d98d68edf4eae").unwrap()
        };

        // Test this builds and connects successfuly
        let mut websocket_handler_under_test = HyperLiquidWebSocketHandler::new().await.unwrap();
        let sub_id = websocket_handler_under_test.0.subscribe_to_market_index(&required_index.market_index).await.unwrap();

        assert_eq!(sub_id, 0);

    }

    #[tokio::test]
    async fn global_manager() {

        let required_index = &MarketIndexData {
            market_index: "@1035".to_string(),
            token_id: H128::from_str("0xbaf265ef389da684513d98d68edf4eae").unwrap()
        };

        // TODO: Looking into providing variable mocked data, or providing a mocked API server so we can pass mocked data through and create deterministic tests
        let (websocket_handler_under_test, _) = HyperLiquidWebSocketHandler::new().await.unwrap();
        let (mocked_sender, mocked_receiver) = unbounded_channel();

        let mocked_book_data = create_test_fixture();

        tokio::spawn(async move {
            for _ in 1..=10 {
                let _ = mocked_sender.send(Message::L2Book(mocked_book_data.clone()));
                sleep(Duration::from_millis(1000)).await; 
            }
        });

        let global_handler_under_test = HyperLiquidGlobalMarketDataHandler::new(Arc::new(Mutex::new(websocket_handler_under_test)), mocked_receiver).await;
        let _ = global_handler_under_test.subscribe_to_market(&required_index.market_index).await;

        let mut interval = time::interval(Duration::from_secs(1));
        let mut target = 0;

        loop {

            interval.tick().await; // Waits for the interval to complete
            assert_eq!(1, global_handler_under_test.market_data_cache.len());
            if target > 10 {
                break;
            }
            target+=1;
        }

        if let Some(test_result) = global_handler_under_test.market_data_cache.get("@1035")  {
            assert_eq!(test_result.value().latest_book.bids.get_best_price_level(), Some(&PriceLevel{ price: Rate(BigDecimal::from_str("51.05").unwrap()), quantity: Quantity(BigDecimal::from_str("677.32").unwrap())} ));
            assert_eq!(test_result.value().latest_book.asks.get_best_price_level(), Some(&PriceLevel{ price: Rate(BigDecimal::from_str("88.99").unwrap()), quantity: Quantity(BigDecimal::from_str("0.43").unwrap())} ));
        }

        assert!(global_handler_under_test.market_data_cache.contains_key("@1035"));

        let notifier_under_test = global_handler_under_test.get_notified_for_market(&required_index.market_index);
        
        assert!(notifier_under_test.is_some());

        let notification_under_test = notifier_under_test.unwrap().notified().await;

        assert!(notification_under_test.is_ok());

    }
}