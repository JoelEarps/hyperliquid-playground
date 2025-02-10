# Requirements

## Current assumptions

We start with a requested base pair
The current system only uses USDC as a quote, there is potential for this to change in the future.

## Functional Requirements

1. Must make a request to http server and get information for all market pairs and their associated market index.
2. Use the captured market index and open a web socket client to retrieve live market data at a rate of once per second? 
3. Convert this market data into a usable orderbook.

The rest is TBC.

Open Questions

1. How are we going to handle the correct pairings?
2. What rate of request do we need for the websocket
3. What rate of request is needed for requesting market ids?