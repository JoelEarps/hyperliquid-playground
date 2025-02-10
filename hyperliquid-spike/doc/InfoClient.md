# InfoClient

The info client is used to grab information from the hyperliquid main net.
It consists of two parts a http client and a websocket client that is initially null.

When subscribing this then creates an instance of the websocket client and then you susbscribe to a particular channel

## Orderbook data

Order book data comes in the form of L2 data, which has a data key which then holds L2BookData

The data looks like the following:

```
coin:
time:
levels:
```

Where Levels can be described as a vector of vectors containing BookLevel data with the following structure:

```

BookLevel { 
    px: "2752.8", 
    sz: "13.9962", 
    n: 2 
}

```

where:

px = a string representing price
sz = a string representing size
n = a number representing "The number of different orders that comprise the level" referinf  to the total count of distinct orders that exist at a given price level in an order book.

Spot vs PERP(https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/info-endpoint#:~:text=startTime%20for%20pagination.-,Perpetuals%20vs%20Spot,-The%20endpoints%20in)

Requesting Spot you specify the base and quote, seperated by a /
for per you just specify the coin

## The universe and its Tokens

BASE/QUOTE pairs are not represented by string e.g. HYPE/USDC but by index
Each market as a unique string id, so we need to take that string id and use it but also know which market for OMS
Make request, extract information from the universe field, mapping the Market ID to the index and token 


### Data Contract

Right now all quote currencies are quoted in USDC, however this could change

How do we need to adapt this functionality in order to support different quote currencies?

TODO:
1. Create a Mock Implementation for subscribing and getting faked and mocked data.
2. Subscribe per coin
3. Turn BookLevel Data into PriceLevel
