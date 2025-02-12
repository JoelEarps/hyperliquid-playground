use std::{collections::HashMap, fmt::Display, ops::Deref};

use bigdecimal::BigDecimal;
use connector_model::{bundle::{executed_order::ExecutedOrder, limit_order_result::{BundledLimitOrdersTransactionResult, BundledLimitOrdersTransactionResultForNetwork, TransactionExecuted}}, connector::{dex_api::{BuildTransactionRequest, DexTradingApi, ExecuteTransactionRequest, GetFeeRateRequest, GetReferenceMarketDataRequest, GetTransactionResquest, GetWalletBalanceRequest, StreamOrderBookRequest}, response::BuildTransactionResponse}, dex, network::types::NetworkTypes, orderbook::{OrderBook, ReferenceMarketDataForNetwork}, pricing::{FeeRate, Quantity}};
use hyperliquid_rust_sdk::{BaseUrl, ClientOrder, ClientOrderRequest, ClientLimit, ExchangeClient, InfoClient};
use tonic::Status;
use futures::{stream::BoxStream};
// Deprecated Library
use ethers::{signers::LocalWallet, types::{Transaction, H256}};
use std::str::FromStr;

pub(crate) struct HyperLiquidApiClient {
    exchange_client: ExchangeClient
}

impl HyperLiquidApiClient {
    // Rather than creating a seperate HTTP client we can use the same one created for the InfoClient, how will this work with concurrency?
    async fn new(info_client: InfoClient, user_wallet: LocalWallet) -> Self {
        Self {
            // What are the options here and what are they used for
            // Open Questions, can we pass in the url as a configurable address for testing
            exchange_client: ExchangeClient::new(None, user_wallet, Some(BaseUrl::Testnet), None, None).await.expect("Cannot make exchange client, fail for now, handle errors later"),
        }
    }
}   

#[derive(Sized, Debug, Display, Clone, Send, Sync)]
struct fakedSig;

/// A network in this context refers, as you can see this one is called HyperLiquidNetwork
/// Example implementation https://gitlab.com/swissborg/defi/evm-connector/-/blob/main/mont-blanc/src/network.rs
/// A network must implement the following traits: Network Types
pub(crate) struct HyperLiquidNetwork;

// Open Questions: How do we handle addresses, as we are not referencing markets, we are referencing tokens
impl NetworkTypes for HyperLiquidNetwork {
    /// A custom type referring to the DEX being used, this then dictates the rest of the meta information labelled below.
    type Dex = dex::AvalancheDex;

    /// An address in a blockchain context represents a destination or account where cryptocurrencies can be sent or received. 
    /// It's usually derived from a public key and is unique for each user or smart contract.
    /// An example of this for EVM based chains are "0x742d35Cc6634C0532925a3b844Bc454e4438f44e"
    type Address = String;

    /// A digital signature in blockchain is used to verify the authenticity of transactions. 
    /// It is generated when a private key signs a transaction, and it ensures that only the rightful owner can authorize transfers.
    /// In Ethereum (using ECDSA, secp256k1 curve), a signature is a 65-byte value containing: 
    type Signature = String;
    //. A transaction is an action submitted to the blockchain that changes its state (e.g., transferring tokens, executing smart contracts).
    /// How a transaction looks can differ from a blockchain, however looking at
    type Transaction = ClientOrderRequest;
    // With Exchange Client we have the following process Create Exchange Client -> ClientOrderRequest -> Order -> Receive 

    /// NATIVE_DECIMALS typically refers to the number of decimal places a blockchain's native asset (like ETH, BTC, or SOL) uses for its smallest unit. This value defines how many fractional units make up one whole unit of the currency.
    /// BTC uses 8, ETH use 18 and as this will use an EVM then I will also use 18
    const NATIVE_DECIMALS: u8 = 18;

    fn get_transaction_signature(tx: &Self::Transaction) -> &Self::Signature {
        // How can the return type be a none reference if I am accessing a reference? Also what is the lifetime of this is reference a part of the input variable?
        todo!()
    }
}

// Implying the DexTradingAPI
// Why, what does this do? 
// Takes in Generic Paramater Network Types and (), what are these and what does they do?
// What is all the functionality 

// What is an async trait?
#[async_trait::async_trait]
impl DexTradingApi<HyperLiquidNetwork, ()> for HyperLiquidApiClient{
    /// What does this func do?
    async fn stream_order_book(
        &self,
        request: StreamOrderBookRequest<<HyperLiquidNetwork as NetworkTypes>::Address, <HyperLiquidNetwork as NetworkTypes>::Dex>,
    ) -> Result<(String, BoxStream<'static, Result<OrderBook, Status>>), Status> {
        todo!()
    }

    /// Responsible for building the transaction in the format required for the Platform
    async fn build_transaction(
        &self,
        request: BuildTransactionRequest<<HyperLiquidNetwork as NetworkTypes>::Address, <HyperLiquidNetwork as NetworkTypes>::Dex, <HyperLiquidNetwork as NetworkTypes>::Address>,
        // Rename this as it is wrong?
    ) -> Result<BuildTransactionResponse<<HyperLiquidNetwork as NetworkTypes>::Transaction>, Status>{
        // Looking at avalanche implementation, do we need bundling?
        // Need to convert OrderBook base and quote to string with BASE?QUOTE

        let order = ClientOrderRequest {
            asset: "XYZTWO/USDC".to_string(),
            is_buy: true,
            reduce_only: false,
            limit_px: 0.00002378,
            sz: 1000000.0,
            cloid: None,
            order_type: ClientOrder::Limit(ClientLimit{
                tif: "Gtc".to_string(),
            }),
        };

        Ok(BuildTransactionResponse::Success(order))
    }

    /// This function is responsible for executing a transaction for a network, it returns a status for the transaction.
    /// The common type returned is the signature, however there is also currency and Market ID for other responses
    /// 
    // Open Question: How do we access the signature here, we may have to manipulate this
    async fn execute_transaction(
        &self,
        request: ExecuteTransactionRequest<<HyperLiquidNetwork as NetworkTypes>::Transaction>,
    ) -> Result<BundledLimitOrdersTransactionResultForNetwork<HyperLiquidNetwork, ()>, Status>{
        // Do we need the second wallet part here?
        let order_status = self.exchange_client.order(request.tx, None ).await.expect("Order could not be completed, execute custom error handling");
        // How can we get this info from th current info, this is currently mocked to show how it would work, doing this doesn't satisfyy trait bounds
        let vector_of_created_orders: Vec<ExecutedOrder<String, String>> = Vec::new();
        let order_quantity = Quantity(BigDecimal::from_str("1").expect("Custom error not needed here as we will be taking this from somewhere"));
        let signature = "faked_sig".to_string();
    

        Ok(BundledLimitOrdersTransactionResultForNetwork::TransactionExecuted(TransactionExecuted {
            tx_id: signature,
            tx_fee: order_quantity,
            orders: vector_of_created_orders
        }))
    }

    /// What does this func do?
    async fn get_reference_market_data(
        &self,
        request: GetReferenceMarketDataRequest<<HyperLiquidNetwork as NetworkTypes>::Address>,
    ) -> Result<ReferenceMarketDataForNetwork<HyperLiquidNetwork>, Status>{
        todo!()
    }

    /// What does this func do?
    async fn get_fee_rate(&self, request: GetFeeRateRequest<<HyperLiquidNetwork as NetworkTypes>::Address>)
        -> Result<FeeRate, Status>{
            todo!()
        }

    /// Returns the native token balance remaining for paying gas fee (used for monitoring)
    /// and the map of proxy's token balance - what does this mean?
    async fn get_wallet_balance(
        &self,
        request: GetWalletBalanceRequest<<HyperLiquidNetwork as NetworkTypes>::Address>,
    ) -> Result<(BigDecimal, HashMap<<HyperLiquidNetwork as NetworkTypes>::Address, BigDecimal>), Status>{
        todo!()
    }

    /// What does this func do?
    async fn get_transaction(
        &self,
        request: GetTransactionResquest<<HyperLiquidNetwork as NetworkTypes>::Signature>,
    ) -> Result<BundledLimitOrdersTransactionResultForNetwork<HyperLiquidNetwork, ()>, Status>{
        todo!()
    }

    async fn ready(&self) -> Result<bool, Status>{
        todo!()
    }
}