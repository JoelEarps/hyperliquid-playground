use std::{collections::HashMap, fmt::Display, ops::Deref, sync::Arc, thread::spawn};

use bigdecimal::BigDecimal;
use connector_model::{bundle::{executed_order::ExecutedOrder, limit_order_result::{BundledLimitOrdersTransactionResult, BundledLimitOrdersTransactionResultForNetwork, TransactionExecuted}}, connector::{dex_api::{BuildTransactionRequest, DexTradingApi, ExecuteTransactionRequest, GetFeeRateRequest, GetReferenceMarketDataRequest, GetTransactionResquest, GetWalletBalanceRequest, StreamOrderBookRequest}, market_builder::MarketBuilder, market_type::{MarketBuilderParameters, MarketType}, response::BuildTransactionResponse}, dex, network::types::NetworkTypes, orderbook::{OrderBook, OrderBookSide, ReferenceMarketDataForNetwork}, pricing::{FeeRate, Quantity}};
use connector_utils::sync::notifier::{MergedNotifier, Notified};
use hyperliquid_rust_sdk::{BaseUrl, ClientLimit, ClientOrder, ClientOrderRequest, ExchangeClient, InfoClient, Message, Subscription};
use tonic::Status;
use futures::{stream::BoxStream};
// Deprecated Library
use ethers::{signers::LocalWallet, types::{Transaction, H256}};
use std::str::FromStr;

use crate::index_extractor::MarketIndexData;

use super::{hyper_liquid_websocket::{HyperLiquidGlobalMarketDataHandler, HyperLiquidWebSocketHandler}, hyperliquid_orderbook::TestOrderBook};

// TODO: Imply Metrics and GRPC Connector
pub(crate) struct HyperLiquidApiClient {
    pub(crate) exchange_client: ExchangeClient,
     global_market_handler: Arc<HyperLiquidGlobalMarketDataHandler>,
    market_id: MarketIndexData
}

impl HyperLiquidApiClient {
    // Rather than creating a seperate HTTP client we can use the same one created for the InfoClient, how will this work with concurrency?
    pub(crate) async fn new(user_wallet: LocalWallet, global_market_handler: &Arc<HyperLiquidGlobalMarketDataHandler>, market_id: &MarketIndexData) -> Self {
        Self {
            // TODO: Configurable Address for 
            exchange_client: ExchangeClient::new(None, user_wallet, Some(BaseUrl::Testnet), None, None).await.expect("Cannot make exchange client, fail for now, handle errors later"),
            market_id: market_id.clone(),
            global_market_handler: global_market_handler.clone()
        }
    }
}

#[async_trait::async_trait]
impl MarketBuilder<u64> for HyperLiquidApiClient {
    async fn fetch_orderbook(&self, params: &MarketBuilderParameters<u64>) -> anyhow::Result<OrderBook> {
        if let Some(orderbook) = self.global_market_handler.get_orderbook_data_for_market(&self.market_id.market_index) {
            Ok(OrderBook { bids: OrderBookSide::empty(), asks: OrderBookSide::empty() })
        } else {
            return Err(anyhow::Error::msg("Test message for now"));
        }
   }

    fn market_type(&self) -> MarketType { todo!() }

    fn new_notified(&self) -> Option<Notified> {
        self.global_market_handler.get_notified_for_market(&self.market_id.market_index)
    }
}

pub(crate) struct HyperLiquidNetwork;

impl NetworkTypes for HyperLiquidNetwork {

    type Dex = dex::AvalancheDex;
    type Address = String;
    type Signature = String;
    type Transaction = ClientOrderRequest; 

    const NATIVE_DECIMALS: u8 = 18;

    fn get_transaction_signature(tx: &Self::Transaction) -> &Self::Signature {
        // How can the return type be a none reference if I am accessing a reference? Also what is the lifetime of this is reference a part of the input variable?
        todo!()
    }
}

// // Implying the DexTradingAPI
// // Why, what does this do? 
// // Takes in Generic Paramater Network Types and (), what are these and what does they do?
// // What is all the functionality 

// // What is an async trait?
// #[async_trait::async_trait]
// impl DexTradingApi<HyperLiquidNetwork, ()> for HyperLiquidApiClient{
//     /// What does this func do?
//     /// In crusty this function calls another function called stream_order_book_can that is implemented in the GRPC connector traits - why?
//     /// The fetch orderbook function is responsible for creating the stream, which we can look at once this basic implementation is done!
//     /// This is called after calling build_orderbook_stream
//     /// This is all called by functions that call the DynMarketClient, which represents market ids for DEX's.
//     async fn stream_order_book(
//         &self,
//         request: StreamOrderBookRequest<<HyperLiquidNetwork as NetworkTypes>::Address, <HyperLiquidNetwork as NetworkTypes>::Dex>,
//     ) -> Result<(String, BoxStream<'static, Result<OrderBook, Status>>), Status> {
//         // Build DynClient - do we need this?
//         // Build get_or_sub_client - do we need this?
//         // Build orderbook stream
//         // fetch_orderbook
//         todo!()
//     }

//     /// Responsible for building the transaction in the format required for the Platform
//     async fn build_transaction(
//         &self,
//         request: BuildTransactionRequest<<HyperLiquidNetwork as NetworkTypes>::Address, <HyperLiquidNetwork as NetworkTypes>::Dex, <HyperLiquidNetwork as NetworkTypes>::Address>,
//         // Rename this as it is wrong?
//     ) -> Result<BuildTransactionResponse<<HyperLiquidNetwork as NetworkTypes>::Transaction>, Status>{
//         // Looking at avalanche implementation, do we need bundling?
//         // Need to convert OrderBook base and quote to string with BASE?QUOTE
//         let order = ClientOrderRequest {
//             // Pass in required values to here
//             // Generate unique CLOID and this will be how we identify transactions
//             asset: "HYPE/USDC".to_string(),
//             is_buy: true,
//             reduce_only: false,
//             limit_px: 0.00002378,
//             sz: 1000000.0,
//             cloid: None,
//             order_type: ClientOrder::Limit(ClientLimit{
//                 tif: "Gtc".to_string(),
//             }),
//         };

//         // How do we deal with failures?
//         Ok(BuildTransactionResponse::Success(order))
//     }

//     /// This function is responsible for executing a transaction for a network, it returns a status for the transaction.
//     /// The common type returned is the signature, however there is also currency and Market ID for other responses
//     /// 
//     // Open Question: How do we access the signature here, we may have to manipulate this
//     async fn execute_transaction(
//         &self,
//         request: ExecuteTransactionRequest<<HyperLiquidNetwork as NetworkTypes>::Transaction>,
//     ) -> Result<BundledLimitOrdersTransactionResultForNetwork<HyperLiquidNetwork, ()>, Status>{
//         // Do we need the second wallet part here?
//         let order_status = self.exchange_client.order(request.tx, None ).await.expect("Order could not be completed, execute custom error handling");
//         // How can we get this info from th current info, this is currently mocked to show how it would work, doing this doesn't satisfyy trait bounds
//         let vector_of_created_orders: Vec<ExecutedOrder<String, String>> = Vec::new();
//         let order_quantity = Quantity(BigDecimal::from_str("1").expect("Custom error not needed here as we will be taking this from somewhere"));
//         let signature = "faked_sig".to_string();

//         Ok(BundledLimitOrdersTransactionResultForNetwork::<HyperLiquidNetwork, ()>::TransactionExecuted(TransactionExecuted {
//             tx_id: signature,
//             tx_fee: order_quantity,
//             orders: vector_of_created_orders
//         }))
//     }

//     /// What does this func do?
//     async fn get_reference_market_data(
//         &self,
//         request: GetReferenceMarketDataRequest<<HyperLiquidNetwork as NetworkTypes>::Address>,
//     ) -> Result<ReferenceMarketDataForNetwork<HyperLiquidNetwork>, Status>{
//         todo!()
//     }

//     /// What does this func do?
//     async fn get_fee_rate(&self, request: GetFeeRateRequest<<HyperLiquidNetwork as NetworkTypes>::Address>)
//         -> Result<FeeRate, Status>{
//             todo!()
//         }

//     /// Returns the native token balance remaining for paying gas fee (used for monitoring)
//     /// and the map of proxy's token balance - what does this mean?
//     async fn get_wallet_balance(
//         &self,
//         request: GetWalletBalanceRequest<<HyperLiquidNetwork as NetworkTypes>::Address>,
//     ) -> Result<(BigDecimal, HashMap<<HyperLiquidNetwork as NetworkTypes>::Address, BigDecimal>), Status>{
//         todo!()
//     }

//     /// What does this func do?
//     async fn get_transaction(
//         &self,
//         request: GetTransactionResquest<<HyperLiquidNetwork as NetworkTypes>::Signature>,
//     ) -> Result<BundledLimitOrdersTransactionResultForNetwork<HyperLiquidNetwork, ()>, Status>{
//         todo!()
//     }

//     async fn ready(&self) -> Result<bool, Status>{
//         todo!()
//     }
// }


#[cfg(test)]
mod tests {
    use crate::__fixtures__::orderbook_fixtures::create_test_fixture;

    use super::*;
    use std::{sync::Arc, time::Duration};
    use tokio::{sync::{mpsc::unbounded_channel, Mutex}, time::sleep};

    use connector_model::{connector::{market_builder::MarketBuilder, market_type::MarketBuilderParameters}, orderbook::OrderBook};
    use ethers::{signers::LocalWallet, types::H128};
    use futures::{stream::BoxStream, StreamExt};


    
#[tokio::test(flavor = "multi_thread")]
// What are the key characteristics do we care abotu testing here?
async fn fetch_orderbook_hyperliquid_client(){
    let wallet: LocalWallet = "e908f86dbb4d55ac876378565aafeabc187f6690f046459397b17d9b9a19688e"
        .parse()
        .unwrap();

    let required_index = &MarketIndexData {
            market_index: "@1035".to_string(),
            token_id: H128::from_str("0xbaf265ef389da684513d98d68edf4eae").unwrap()
    };

    // Reused Logic, can this be made into a test fixture
    // TODO: Looking into providing variable mocked data, or providing a mocked API server so we can pass mocked data through and create deterministic tests
    let (websocket_handler_under_test, _) = HyperLiquidWebSocketHandler::new().await.unwrap();
    let (mocked_sender, mocked_receiver) = unbounded_channel();

    let global_handler_under_test = HyperLiquidGlobalMarketDataHandler::new(Arc::new(Mutex::new(websocket_handler_under_test)), mocked_receiver).await;
    let _ = global_handler_under_test.subscribe_to_market(&required_index.market_index).await;

    let mocked_book_data = create_test_fixture();

        tokio::spawn(async move {
            for _ in 1..=10 {
                let _ = mocked_sender.send(Message::L2Book(mocked_book_data.clone()));
                sleep(Duration::from_millis(1000)).await; 
            }
    });
    
    let hyperliquid_api_client_under_test = Arc::new(HyperLiquidApiClient::new(wallet, &global_handler_under_test, required_index).await);

    
    let mut output_stream: BoxStream<'static, anyhow::Result<OrderBook>> = hyperliquid_api_client_under_test.build_orderbook_stream(MarketBuilderParameters { orders_limit: 15, convertion_params: None}, 1000, true).await;
    let mut orderbook_results: Vec<OrderBook> = Vec::new();

    for _ in 0..5 {
        if let Some(orderbook) = output_stream.next().await {
            orderbook_results.push(orderbook.unwrap());
        } else {
            panic!("Stream did not produce any output!");
        }
    }

    assert_eq!(orderbook_results.len(), 5);

}
}