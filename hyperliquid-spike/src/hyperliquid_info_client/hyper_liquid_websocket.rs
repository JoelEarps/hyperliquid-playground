use std::fmt::Debug;
use std::sync::{Arc, Mutex};
use std::time::Duration;

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
    pub market_sender: UnboundedSender<Message>,
}

type HandlerResult = (HyperLiquidWebSocketHandler, UnboundedReceiver<Message>);

impl HyperLiquidWebSocketHandler {

    pub(crate) async fn new() -> Result<HandlerResult, ()>{
        let client = Client::default();
        let info_client = InfoClient::new(Some(client), Some(BaseUrl::Testnet)).await.unwrap();
        let (snd,rcv) = unbounded_channel::<Message>();

       Ok((Self{
        info_client, 
        market_sender: snd}, 
        rcv))
    }

    pub async fn subscribe_to_market_index(&mut self, required_index: &MarketIndexData) -> Result<u32, HyperLiquidNetworkErrors>{
        let sub_id = self.info_client
        .subscribe(
            Subscription::L2Book {
                coin: required_index.market_index.clone()
            },
            self.market_sender.clone(),
        )
        .await
        ?;

        Ok(sub_id)
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
impl HyperLiquidGlobalMarketDataHandler {

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
            let global_handler_clone  = self.clone();
            let websocket_job = tokio::spawn(async move {
                    while let Some(Message::L2Book(l2_book)) = market_data_receiver.recv().await {
                    let map_key = l2_book.data.coin.clone();
                    let hyperliquid_order_book = HyperLiquidOrderBookData::try_from(l2_book.data).expect("Add custom error here for failing");
                    // Change to orderbook in connector commons when PR is resolved
                    // https://gitlab.com/swissborg/defi/connector-commons/-/merge_requests/7
                    let swissborg_order_book = TestOrderBook::new_from_iter(hyperliquid_order_book.bids, hyperliquid_order_book.asks);
                    // If key already exists, then perform an insert of new notifier, otherwise use reference to existing to notifier
                    // This will not update the latest data, only if a new reference exists, think the only option is a match statement?
                    // let result = global_handler_clone.market_data_cache.entry(map_key).or_insert(BookNotifier {latest_book: swissborg_order_book, notifier: Notifier::new()} );
                    match global_handler_clone.market_data_cache.entry(map_key) {
                        dashmap::Entry::Occupied(mut current_entry) => {
                            let values = current_entry.get_mut();
                            values.latest_book = swissborg_order_book;
                            values.notifier.notify();

                        },
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
}

pub struct BookNotifier {
    latest_book : TestOrderBook,
    notifier:  Notifier,
}

// impl Debug for BookNotifier {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         format!("Latest Book asks: {}, latest book bids: {}", self.latest_book.asks)
//     }
// }

#[cfg(test)]
mod tests
{
    use std::str::FromStr;
    use bigdecimal::BigDecimal;
    use connector_model::{orderbook::PriceLevel, pricing::{Quantity, Rate}};
    use ethers::types::H128;
    use hyperliquid_rust_sdk::{BookLevel, L2Book};
    use super::*;

    use tokio::{
        spawn,
        sync::mpsc::unbounded_channel,
        time::{sleep, self, Duration},
    };

    // This is reused in hyperliquid-spike/src/hyperliquid_info_client/hyperliquid_orderbook.rs, please make a test fixture
    fn create_test_fixture() -> L2Book{
        L2Book { data: L2BookData { 
            coin: "@1035".to_string(), 
            time: 1739197119187, 
            levels: vec![vec![BookLevel { px: "51.05".to_string(), sz: "677.32".to_string(), n: 2 }, 
                        BookLevel { px: "51.0".to_string(), sz: "605.74".to_string(), n: 2 }, 
                        BookLevel { px: "50.0".to_string(), sz: "14057.16".to_string(), n: 3 }, 
                        BookLevel { px: "46.5".to_string(), sz: "6.96".to_string(), n: 1 }, 
                        BookLevel { px: "46.0".to_string(), sz: "8.24".to_string(), n: 1 }, 
                        BookLevel { px: "41.683".to_string(), sz: "797.44".to_string(), n: 1 }, 
                        BookLevel { px: "40.0".to_string(), sz: "81577.39".to_string(), n: 4 }, 
                        BookLevel { px: "32.477".to_string(), sz: "1017.86".to_string(), n: 1 }, 
                        BookLevel { px: "32.0".to_string(), sz: "935.78".to_string(), n: 1 }, 
                        BookLevel { px: "30.0".to_string(), sz: "133333.32".to_string(), n: 2 }, 
                        BookLevel { px: "26.0".to_string(), sz: "5009.69".to_string(), n: 2 }, 
                        BookLevel { px: "25.0".to_string(), sz: "616.0".to_string(), n: 6 }, 
                        BookLevel { px: "23.2".to_string(), sz: "1.0".to_string(), n: 1 }, 
                        BookLevel { px: "22.0".to_string(), sz: "1307.53".to_string(), n: 1 }, 
                        BookLevel { px: "20.0".to_string(), sz: "200000.0".to_string(), n: 2 }, 
                        BookLevel { px: "11.92".to_string(), sz: "200.0".to_string(), n: 1 }, 
                        BookLevel { px: "11.885".to_string(), sz: "200.0".to_string(), n: 1 }, 
                        BookLevel { px: "11.85".to_string(), sz: "200.0".to_string(), n: 1 }, 
                        BookLevel { px: "11.815".to_string(), sz: "200.0".to_string(), n: 1 }, 
                        BookLevel { px: "11.78".to_string(), sz: "200.0".to_string(), n: 1 }], 
                        vec![BookLevel { px: "88.99".to_string(), sz: "0.43".to_string(), n: 1 }, 
                        BookLevel { px: "89.0".to_string(), sz: "16.38".to_string(), n: 1 }, 
                        BookLevel { px: "90.0".to_string(), sz: "0.51".to_string(), n: 1 }, 
                        BookLevel { px: "93.513".to_string(), sz: "9.98".to_string(), n: 1 }, 
                        BookLevel { px: "94.0".to_string(), sz: "84.82".to_string(), n: 1 }, 
                        BookLevel { px: "94.421".to_string(), sz: "0.27".to_string(), n: 1 }, 
                        BookLevel { px: "96.322".to_string(), sz: "500.0".to_string(), n: 1 }, 
                        BookLevel { px: "97.26".to_string(), sz: "514.74".to_string(), n: 2 }, 
                        BookLevel { px: "98.826".to_string(), sz: "0.86".to_string(), n: 1 }, 
                        BookLevel { px: "99.0".to_string(), sz: "1128.95".to_string(), n: 2 }, 
                        BookLevel { px: "100.0".to_string(), sz: "689.91".to_string(), n: 1 }, 
                        BookLevel { px: "101.01".to_string(), sz: "4602.53".to_string(), n: 1 }, 
                        BookLevel { px: "102.02".to_string(), sz: "10000.0".to_string(), n: 1 }, 
                        BookLevel { px: "103.03".to_string(), sz: "10000.0".to_string(), n: 1 }, 
                        BookLevel { px: "104.04".to_string(), sz: "10000.0".to_string(), n: 1 }, 
                        BookLevel { px: "105.05".to_string(), sz: "10000.0".to_string(), n: 1 }, 
                        BookLevel { px: "106.06".to_string(), sz: "10000.0".to_string(), n: 1 }, 
                        BookLevel { px: "107.05".to_string(), sz: "1297.88".to_string(), n: 1 }, 
                        BookLevel { px: "107.07".to_string(), sz: "10000.0".to_string(), n: 1 }, 
                        BookLevel { px: "108.08".to_string(), sz: "10000.0".to_string(), n: 1 }]]}}
    }

    // Provide test url for websocket? 
    #[tokio::test]
    async fn test_websocket_handler(){
        let required_index = &MarketIndexData {
            market_index: "@1035".to_string(),
            token_id: H128::from_str("0xbaf265ef389da684513d98d68edf4eae").unwrap()
        };

        // Test this builds and connects successfuly
        let mut websocket_handler_under_test = HyperLiquidWebSocketHandler::new().await.unwrap();
        websocket_handler_under_test.0.subscribe_to_market_index(required_index).await.unwrap();

        // create mocked channel for sender?

        // Create a channel with a buffer size of 10
        let (mocked_sender, mocked_receiver) = unbounded_channel::<Message>();

        // Spawn a sender task
        // Can handle no data sent here too, helpful for BDD scenario
        // Send the same data and always expect same length of data and same data to be present in hash map

        let mocked_book_data = create_test_fixture();

        tokio::spawn(async move {
            for i in 1..=10 {
                mocked_sender.send(Message::L2Book(mocked_book_data.clone()));
                sleep(Duration::from_millis(1000)).await; 
            }
        });

        let global_handler_under_test = HyperLiquidGlobalMarketDataHandler::new(Arc::new(Mutex::new(websocket_handler_under_test.0)), mocked_receiver).await;

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

    }
}