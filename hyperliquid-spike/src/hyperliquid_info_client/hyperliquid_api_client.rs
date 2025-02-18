// use std::{collections::HashMap, fmt::Display, ops::Deref, thread::spawn};

// use bigdecimal::BigDecimal;
// use connector_model::{bundle::{executed_order::ExecutedOrder, limit_order_result::{BundledLimitOrdersTransactionResult, BundledLimitOrdersTransactionResultForNetwork, TransactionExecuted}}, connector::{dex_api::{BuildTransactionRequest, DexTradingApi, ExecuteTransactionRequest, GetFeeRateRequest, GetReferenceMarketDataRequest, GetTransactionResquest, GetWalletBalanceRequest, StreamOrderBookRequest}, market_builder::MarketBuilder, market_type::{MarketBuilderParameters, MarketType}, response::BuildTransactionResponse}, dex, network::types::NetworkTypes, orderbook::{OrderBook, OrderBookSide, ReferenceMarketDataForNetwork}, pricing::{FeeRate, Quantity}};
// use hyperliquid_rust_sdk::{BaseUrl, ClientLimit, ClientOrder, ClientOrderRequest, ExchangeClient, InfoClient, Message, Subscription};
// use reqwest::Client;
// use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver};
// use tonic::Status;
// use futures::{stream::BoxStream};
// // Deprecated Library
// use ethers::{signers::LocalWallet, types::{Transaction, H256}};
// use std::str::FromStr;

// use crate::{index_extractor::MarketIndexData, HyperLiquidOrderBookData, TestOrderBook};

// use super::hyper_liquid_websocket::HyperLiquidWebSocketHandler;

// // TODO: Imply Metrics and GRPC Connector - to be done with BD when tested
// pub(crate) struct HyperLiquidApiClient {
//     pub(crate) exchange_client: ExchangeClient,
//     pub(crate) info_client: InfoClient,
//     // Temporary Market Pair
//     pub(crate) required_market_pair: &'static str,  
//     pub websocket_handler: HyperLiquidWebSocketHandler
// }

// impl HyperLiquidApiClient {
//     // Rather than creating a seperate HTTP client we can use the same one created for the InfoClient, how will this work with concurrency?
//     pub(crate) async fn new( user_wallet: LocalWallet) -> Self {
//         let client = Client::default();
//         Self {
//             // What are the options here and what are they used for
//             // Open Questions, can we pass in the url as a configurable address for testing
//             exchange_client: ExchangeClient::new(None, user_wallet, Some(BaseUrl::Testnet), None, None).await.expect("Cannot make exchange client, fail for now, handle errors later"),
//             info_client: InfoClient::new(Some(client), Some(BaseUrl::Testnet)).await.unwrap(),
//             required_market_pair: "HYPE_USDC",
//             websocket_handler: HyperLiquidWebSocketHandler::new()
//         }
//     }

//     pub async fn run_and_subscribe_to_info_websocket(&self, required_index: &MarketIndexData){
//         self.websocket_handler.initialise_websocket();
//     }

//     pub(super) async fn fetch_market_pair_data(&self, channel_receiver: UnboundedReceiver<Message>) {
//         while let Some(Message::L2Book(l2_book)) = channel_receiver.recv().await {
//             // Change to orderbook in connector commons when PR is resolved
//             // https://gitlab.com/swissborg/defi/connector-commons/-/merge_requests/7
//             let hyperliquid_order_book = HyperLiquidOrderBookData::try_from(l2_book.data).expect("Add custom error here for failing");
//             let swissborg_order_book = TestOrderBook::new_from_iter(hyperliquid_order_book.bids, hyperliquid_order_book.asks);
//             println!("Swissborg model for orderbook: {:?}", swissborg_order_book);
//         }
//     }


// }

// #[async_trait::async_trait]
// impl MarketBuilder<u64> for HyperLiquidApiClient {
//     // Called Periodically, so this should be done prior to build order book stream?
//     async fn fetch_orderbook(&self, params: &MarketBuilderParameters<u64>) -> anyhow::Result<OrderBook> {
//         // self.fetch_market_pair_data().await;
//         Ok(OrderBook { bids: OrderBookSide::empty(), asks: OrderBookSide::empty() })
//    }

//     fn market_type(&self) -> MarketType { todo!() }

//     async fn build_orderbook_stream() {

//     }
    
// }

// /// A network in this context refers, as you can see this one is called HyperLiquidNetwork
// /// Example implementation https://gitlab.com/swissborg/defi/evm-connector/-/blob/main/mont-blanc/src/network.rs
// /// A network must implement the following traits: Network Types
// pub(crate) struct HyperLiquidNetwork;

// // Open Questions: How do we handle addresses, as we are not referencing markets, we are referencing tokens
// // TODO: Add these notes to the Network Types Docs
// impl NetworkTypes for HyperLiquidNetwork {
//     /// A custom type referring to the DEX being used, this then dictates the rest of the meta information labelled below.
//     /// TODO: Add HL to Dex Types on Github - done but need to mirror
//     type Dex = dex::AvalancheDex;

//     /// An address in a blockchain context represents a destination or account where cryptocurrencies can be sent or received. 
//     /// It's usually derived from a public key and is unique for each user or smart contract.
//     /// An example of this for EVM based chains are "0x742d35Cc6634C0532925a3b844Bc454e4438f44e"
//     type Address = String;

//     /// A digital signature in blockchain is used to verify the authenticity of transactions. 
//     /// It is generated when a private key signs a transaction, and it ensures that only the rightful owner can authorize transfers.
//     /// In Ethereum (using ECDSA, secp256k1 curve), a signature is a 65-byte value containing: 
//     type Signature = String;
//     //. A transaction is an action submitted to the blockchain that changes its state (e.g., transferring tokens, executing smart contracts).
//     /// How a transaction looks can differ from a blockchain, however looking at
//     type Transaction = ClientOrderRequest;
//     // With Exchange Client we have the following process Create Exchange Client -> ClientOrderRequest -> Order -> Receive 

//     /// NATIVE_DECIMALS typically refers to the number of decimal places a blockchain's native asset (like ETH, BTC, or SOL) uses for its smallest unit. This value defines how many fractional units make up one whole unit of the currency.
//     /// BTC uses 8, ETH use 18 and as this will use an EVM then I will also use 18
//     const NATIVE_DECIMALS: u8 = 18;

//     fn get_transaction_signature(tx: &Self::Transaction) -> &Self::Signature {
//         // How can the return type be a none reference if I am accessing a reference? Also what is the lifetime of this is reference a part of the input variable?
//         todo!()
//     }
// }

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


// #[cfg(test)]
// mod tests {
//     use std::sync::Arc;

//     use connector_model::{connector::{market_builder::MarketBuilder, market_type::MarketBuilderParameters}, orderbook::OrderBook};
//     use ethers::signers::LocalWallet;
//     use futures::{stream::BoxStream, StreamExt};

//     use crate::HyperLiquidApiClient;

    
// #[tokio::test(flavor = "multi_thread")]
// async fn fetch_orderbook_hyperliquid_client(){
//     let wallet: LocalWallet = "e908f86dbb4d55ac876378565aafeabc187f6690f046459397b17d9b9a19688e"
//         .parse()
//         .expect("Custom Error message for wallets");
    
//     let test_hl_client = HyperLiquidApiClient::new(wallet).await;
//     let arc_under_test = Arc::new(test_hl_client);

    
//     let mut output_stream: BoxStream<'static, anyhow::Result<OrderBook>> = arc_under_test.build_orderbook_stream(MarketBuilderParameters { orders_limit: 15, convertion_params: None}, 1000, true).await;
//     let mut orderbook_results: Vec<OrderBook> = Vec::new();

//     for _ in 0..5 {
//         if let Some(orderbook) = output_stream.next().await {
//             orderbook_results.push(orderbook.unwrap());
//         } else {
//             panic!("Stream did not produce any output!");
//         }
//     }

//     assert_eq!(orderbook_results.len(), 5);

// }
// }