use std::str::FromStr;

use bigdecimal::BigDecimal;
use connector_model::{orderbook::{OrderBook, OrderBookAsk, OrderBookBid, OrderBookSide, PriceLevel}, pricing::{Quantity, Rate}};
use hyperliquid_rust_sdk::{BookLevel, L2BookData};

use crate::errors::HyperLiquidOrderBookErrors;

/// This is very similar to SUI price levels can we turn this into traits? and then get an incoming OrderBook to use it?
/// But we are using traits here? Can we make a struct and get it to imply these traits? Or generic traits?
pub(crate) struct HyperLiquidOrderBookData {
    timestamp: String,
    pub(crate) bids: Vec<HyperLiquidPriceLevel>,
    pub(crate) asks: Vec<HyperLiquidPriceLevel>
}

pub(crate) struct HyperLiquidPriceLevel {
    price: BigDecimal,
    quantity: BigDecimal
}

impl From<HyperLiquidPriceLevel> for PriceLevel {
    fn from(value: HyperLiquidPriceLevel) -> Self {
        PriceLevel {
            price: Rate::new(value.price),
            quantity: Quantity::new(value.quantity)
        }
    }
}

impl From<HyperLiquidOrderBookData> for OrderBook {
    fn from(value: HyperLiquidOrderBookData) -> Self {

        let ask_price_levels = value.asks
        .into_iter()
        .map(| data | PriceLevel::from(data))
        .collect();

        let bid_price_levels= value.bids.into_iter().
        map(|data| PriceLevel::from(data))
        .collect();

        OrderBook::new(bid_price_levels, ask_price_levels)
    }
}

/// L2 Order Book gives a reference to book level, therefore for Speed of Dev we can do this, is there better way
/// 
impl TryFrom<&BookLevel> for HyperLiquidPriceLevel {
    type Error = HyperLiquidOrderBookErrors;
    fn try_from(input_book_level: &BookLevel) -> Result<Self, Self::Error> {
        Ok( HyperLiquidPriceLevel {
            price: BigDecimal::from_str(&input_book_level.px)?,
            quantity: BigDecimal::from_str(&input_book_level.sz)?
            })
    }
}

#[derive(Debug, Clone)]
pub(crate) struct TestOrderBook {
    pub bids: OrderBookSide<OrderBookBid>,
    pub asks: OrderBookSide<OrderBookAsk>,
}

impl TestOrderBook {
    pub fn new_from_iter<T: Into<PriceLevel>>(
        bids: impl IntoIterator<Item = T>,
        asks: impl IntoIterator<Item = T>,
    ) -> Self {
        Self {
            bids: OrderBookSide::new(bids.into_iter().map(Into::into).collect()),
            asks: OrderBookSide::new(asks.into_iter().map(Into::into).collect()),
        }
    }
    pub fn new_from_iter_ref<'generic_orderbook_lifetime, T: 'generic_orderbook_lifetime + Into<PriceLevel> + Copy>(
        bids: impl IntoIterator<Item = &'generic_orderbook_lifetime T>,
        asks: impl IntoIterator<Item = &'generic_orderbook_lifetime T>,
    ) -> Self {
        Self {
            bids: OrderBookSide::new(bids.into_iter().copied().map(Into::into).collect()),
            asks: OrderBookSide::new(asks.into_iter().copied().map(Into::into).collect()),
        }
    }
}

/// L2BookData - are the vec of vec lengths always two? One to simulate bids and one asks?
/// Is there any documentation that supports this?
/// There always appears to be two vectors, one where the prices start at a level and decrease, one where the prices start at a level and then increase slowly
/// This infers bids and asks and will be therefore treated as such
impl TryFrom<L2BookData> for HyperLiquidOrderBookData {
    type Error = HyperLiquidOrderBookErrors;

    fn try_from(l2_order_book_data: L2BookData) -> Result<Self, Self::Error> {
        if l2_order_book_data.levels.len() != 2 { return Err(HyperLiquidOrderBookErrors::InvalidL2OrderBook)}
        // Bids - located at index 0 i.e. the second vector
        let bids: Vec<HyperLiquidPriceLevel> = l2_order_book_data.levels[0].iter().map(|l2_level| {
            HyperLiquidPriceLevel::try_from(l2_level)
        }).collect::<Result<Vec<HyperLiquidPriceLevel>, HyperLiquidOrderBookErrors>>()?;

        // Asks - located at index 1 i.e. the second vector
        let asks: Vec<HyperLiquidPriceLevel> = l2_order_book_data.levels[1].iter().map(|l2_level| {
            HyperLiquidPriceLevel::try_from(l2_level)
        }).collect::<Result<Vec<HyperLiquidPriceLevel>, HyperLiquidOrderBookErrors>>()?;

        Ok(HyperLiquidOrderBookData{
            timestamp: l2_order_book_data.time.to_string(),
            bids,
            asks
        })
    }
}

#[cfg(test)]
mod tests {
    
    use hyperliquid_rust_sdk::{L2Book, L2BookData, BookLevel};
    use crate::__fixtures__::orderbook_fixtures::create_test_fixture;

    use super::*;

    #[test]
    fn test_order_book_conversion(){
        let test_fixture = create_test_fixture();
        let test_fixture_timestamp = test_fixture.data.time.to_string();
        let test_fixture_bid_len = test_fixture.data.levels[0].len();
        let test_fixture_ask_len = test_fixture.data.levels[1].len();

        let test_hyper_liquid_order_book = HyperLiquidOrderBookData::try_from(test_fixture.data).unwrap();
        assert_eq!(test_hyper_liquid_order_book.asks.len(), test_fixture_ask_len);
        assert_eq!(test_hyper_liquid_order_book.bids.len(), test_fixture_bid_len);
        assert_eq!(test_hyper_liquid_order_book.timestamp, test_fixture_timestamp);

        let order_book_under_test = OrderBook::from(test_hyper_liquid_order_book);
        assert_eq!(order_book_under_test.bids.get_best_price_level(), Some(&PriceLevel{ price: Rate(BigDecimal::from_str("51.05").unwrap()), quantity: Quantity(BigDecimal::from_str("677.32").unwrap())} ));
        assert_eq!(order_book_under_test.asks.get_best_price_level(), Some(&PriceLevel{ price: Rate(BigDecimal::from_str("88.99").unwrap()), quantity: Quantity(BigDecimal::from_str("0.43").unwrap())} ));
    }

    #[test]
    fn generic_order_book_conversion(){
        let test_fixture = create_test_fixture();
        let test_hyper_liquid_order_book = HyperLiquidOrderBookData::try_from(test_fixture.data).unwrap();
        let test_order_book = TestOrderBook::new_from_iter(test_hyper_liquid_order_book.bids, test_hyper_liquid_order_book.asks);

        println!("{:?}", test_order_book);
    }
}
