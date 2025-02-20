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

## TODOs

1. Cannot clone anything in Rust SDK very hard to use and not malleable, could just implement are own set up as it isn't hard.
2. Bench Mark the websocket and cache usage for comparison in the future
3. Integrate with a real wallet
4. Graceful shut down
5. Can we use the same reqwest client
6. Handling Connection Errors for WebSocket, InfoClient and Exchange Client
7. RSTest for parameterisation and reusable test fixtures.
8. Review whether we can use the subscription manager in the SDK to manage readiness and health checks (this may require SDK changes)


Negatives of this approach is that everything becomes very hard to test, we should be really looking at providing wrappers for channels etc so that we can pass in mocked data, there are alternatives to this:

1. Use Mock Channels
2. Look at a Mock Library to create a mock implementation of the class under tests
3. Rearrange so that we can pass in mock implementation of things that would require external dependencies e.g. info and exchange client.