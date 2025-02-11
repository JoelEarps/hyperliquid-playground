use std::collections::HashMap;

use ethers::types::H128;
// Spot Meta is not an availble type from the SDK maybe create a patch, created patch, maybe take this forward? and recommend as a PR.
use hyperliquid_rust_sdk::SpotMeta;
use crate::hyperliquid_info_client::hyperliquid_types::RequiredTokenInfo;

/// Responsible for generating the data in a required format for use within the OMS system
/// It takes the current spot meta data extracted from the info_client.spot_meta() method call
/// Gathers each item required from token meta data and use that to create and validate the indexes in the hyperliquid universe vector.
/// 
/// # Errors
pub(crate) fn extract_market_index(spot_meta: SpotMeta) -> MarketIndexMap {
    let mut market_data_hashmap: MarketIndexMap = HashMap::new();
     // Start with Token structure, extract name, token address and index ✅
     let spot_token_info: Vec<RequiredTokenInfo> = spot_meta.tokens.into_iter().map(|token_info| {
        RequiredTokenInfo {
            name: token_info.name,
            index: token_info.index,
            token_id: token_info.token_id
        }
    }).collect();
    // Potential Idea could we make some sort of relation struct where it maps the base and quote like nodes in a map or tree struct?
    // Turn this into a graph algorithm that periodically updates, or hash map
    for uni_item in spot_meta.universe.into_iter() {
        // Is this the best way to do it, speak to BD on whether this is what we need?
        // Speak to a Data Engineer about maybe a better method for relational data models, maybe a map?
        // Minimum, everything is mapped for USDC, so only worry about the base currency for now
        let (base, quote) = (uni_item.tokens[0], uni_item.tokens[1]);
        let base_match = spot_token_info.clone().into_iter().find(|spot_token| {
            base == spot_token.index
        });

        let quote_match = spot_token_info.clone().into_iter().find(|spot_token| {
            quote == spot_token.index
        });

        // Would it be better to assert on them here?
        match (&base_match, &quote_match) {
            (Some(valid_base_match), Some(valid_quote_match)) => {
                println!("{:?} ---- {:?}, with market index name {}", valid_base_match, valid_quote_match, uni_item.name);
                let key = format!("{}_{}", valid_base_match.name, valid_quote_match.name);
                market_data_hashmap.insert(key, MarketIndexData { market_index: uni_item.name, token_id: valid_base_match.token_id });
            }
            (Some(_), None) => println!("Error no quote to match"),
            (None, Some(_)) => println!("Error no base to match"),
            (None, None) => println!("Error")
        }   
    }

    market_data_hashmap
}

pub type MarketIndexMap = HashMap<String, MarketIndexData>;

#[derive(PartialEq, Eq, Debug)]
pub struct MarketIndexData {
    pub market_index: String,
    token_id: H128,
}

#[cfg(test)]
mod tests {

    use super::*;
    use std::str::FromStr;

    const TEST_FIXTURE : &str = r#"
    {
      "universe": [
        {
          "tokens": [1, 0],
          "name": "PURR/USDC",
          "index": 0,
          "isCanonical": true
        },
        {
          "tokens": [2, 0],
          "name": "@1",
          "index": 1,
          "isCanonical": false
        },
        {
          "tokens": [3, 0],
          "name": "@2",
          "index": 2,
          "isCanonical": false
        },
        {
          "tokens": [4, 0],
          "name": "@3",
          "index": 3,
          "isCanonical": false
        },
        {
          "tokens": [5, 0],
          "name": "@4",
          "index": 4,
          "isCanonical": false
        }
      ],
      "tokens": [
        {
          "name": "USDC",
          "szDecimals": 8,
          "weiDecimals": 8,
          "index": 0,
          "tokenId": "0x6d1e7cde53ba9467b783cb7c530ce054",
          "isCanonical": true,
          "evmContract": null,
          "fullName": null,
          "deployerTradingFeeShare": "0.0"
        },
        {
          "name": "PURR",
          "szDecimals": 0,
          "weiDecimals": 5,
          "index": 1,
          "tokenId": "0xc1fb593aeffbeb02f85e0308e9956a90",
          "isCanonical": true,
          "evmContract": null,
          "fullName": null,
          "deployerTradingFeeShare": "0.0"
        },
        {
          "name": "HFUN",
          "szDecimals": 2,
          "weiDecimals": 8,
          "index": 2,
          "tokenId": "0xbaf265ef389da684513d98d68edf4eae",
          "isCanonical": false,
          "evmContract": null,
          "fullName": null,
          "deployerTradingFeeShare": "0.0"
        },
        {
          "name": "LICK",
          "szDecimals": 0,
          "weiDecimals": 5,
          "index": 3,
          "tokenId": "0xba3aaf468f793d9b42fd3328e24f1de9",
          "isCanonical": false,
          "evmContract": null,
          "fullName": null,
          "deployerTradingFeeShare": "0.0"
        },
        {
          "name": "MANLET",
          "szDecimals": 0,
          "weiDecimals": 5,
          "index": 4,
          "tokenId": "0xe9ced9225d2a69ccc8d6a5b224524b99",
          "isCanonical": false,
          "evmContract": null,
          "fullName": null,
          "deployerTradingFeeShare": "0.0"
        }
      ]
    }
    "#;
    
    #[test]
    fn check_for_base_quote_match() {
        let valid_test_data: SpotMeta = serde_json::from_str(TEST_FIXTURE).unwrap();
        let test_data_mapping = extract_market_index(valid_test_data);
        assert_eq!(test_data_mapping.len(), 4);

        let expected_hashmap = MarketIndexMap::from([
            ("PURR_USDC".to_string(), MarketIndexData { market_index: "PURR/USDC".to_string(), token_id: H128::from_str("0xc1fb593aeffbeb02f85e0308e9956a90").unwrap() }),
            ("HFUN_USDC".to_string(), MarketIndexData { market_index: "@1".to_string(), token_id: H128::from_str("0xbaf265ef389da684513d98d68edf4eae").unwrap() }),
            ("LICK_USDC".to_string(), MarketIndexData { market_index: "@2".to_string(), token_id: H128::from_str("0xba3aaf468f793d9b42fd3328e24f1de9").unwrap() }),
            ("MANLET_USDC".to_string(), MarketIndexData { market_index: "@3".to_string(), token_id: H128::from_str("0xe9ced9225d2a69ccc8d6a5b224524b99").unwrap() }),
        ]);

        assert_eq!(expected_hashmap, test_data_mapping);
    }
}