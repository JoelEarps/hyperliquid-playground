use std::str::FromStr;

use bigdecimal::BigDecimal;
use connector_model::{orderbook::{OrderBook, PriceLevel}, pricing::{Quantity, Rate}};
use hyperliquid_rust_sdk::{BookLevel, L2Book, L2BookData};

use crate::errors::HyperLiquidOrderBookErrors;

pub(super) struct HyperLiquidOrderBookData {
    timestamp: String,
    bids: Vec<HyperLiquidPriceLevel>,
    asks: Vec<HyperLiquidPriceLevel>
}

struct HyperLiquidPriceLevel {
    price: BigDecimal,
    quantity: BigDecimal
}

/// L2 Order Book gives a reference to book level, therefore for Speed of Dev we can do this, is there better way
/// 
impl TryFrom<&BookLevel> for HyperLiquidPriceLevel {
    type Error = HyperLiquidOrderBookErrors;
    fn try_from(input_book_level: &BookLevel) -> Result<Self, Self::Error> {
        Ok( HyperLiquidPriceLevel {
            price: BigDecimal::from_str(&input_book_level.px)?,
            quantity: BigDecimal::from_str(&input_book_level.px)?
            })
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
        // Bids
        let bids: Vec<HyperLiquidPriceLevel> = l2_order_book_data.levels[0].iter().map(|l2_level| {
            HyperLiquidPriceLevel::try_from(l2_level)
        }).collect::<Result<Vec<HyperLiquidPriceLevel>, HyperLiquidOrderBookErrors>>()?;

        // Asks
        let asks: Vec<HyperLiquidPriceLevel> = l2_order_book_data.levels[0].iter().map(|l2_level| {
            HyperLiquidPriceLevel::try_from(l2_level)
        }).collect::<Result<Vec<HyperLiquidPriceLevel>, HyperLiquidOrderBookErrors>>()?;

        Ok(HyperLiquidOrderBookData{
            timestamp: l2_order_book_data.time.to_string(),
            bids,
            asks
        })
    }
}

// impl From<HyperLiquidOrderBookData> for OrderBook {
//     fn from(value: HyperLiquidOrderBookData) -> Self {
        
//     }
// }
/// This function converts the orderbook data from the hyperliquid format to the required orderbook provided by connector-commons
fn convert_to_orderbook(){

}

#[cfg(test)]
mod tests {

    use hyperliquid_rust_sdk::{L2Book, L2BookData, BookLevel};

    use super::*;

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

    #[test]
    fn test_order_book_conversion(){
        let test_fixture = create_test_fixture();
        let test_fixture_bid_len = test_fixture.data.levels[0].len();
        let test_fixture_ask_len = test_fixture.data.levels[1].len();

        let test_hyper_liquid_order_book = HyperLiquidOrderBookData::try_from(test_fixture.data).unwrap();
        assert_eq!(test_hyper_liquid_order_book.asks.len(), test_fixture_ask_len);
        assert_eq!(test_hyper_liquid_order_book.bids.len(), test_fixture_bid_len);

    }
}
