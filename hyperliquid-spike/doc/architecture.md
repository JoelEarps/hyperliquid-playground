# Service Architecture

## Components

### Websocket Handler

#### Communication using Channels

The WebSocket handler, API client and Global Data Cache all communicate through channels in the following ways:

1. Websocket -> Global Data Cache
2. 

### Latest Data Cache - HyperLiquidGlobalMarketDataHandler

### HyperLiquidApiClient

## Service Diagram

```mermaid

graph TD
    A[Tokio Job Handle] -->|Spawns Task| B[WebSocket Connection]
    B -->|Subscribes to| C[L2 Book Snapshot HYPE_USDC]
    B -->|Subscribes to| D[L2 Book Snapshot PURR_USDC]
    B -->|Subscribes to| E[L2 Book Snapshot TOKEN1_USDC]
    B -->|Subscribes to| F[L2 Book Snapshot TOKEN2_USDC]

    C -->|Transforms and Pushes to Cache| G[L2BookData Cache]
    D -->|Transforms and Pushes to Cache| G
    E -->|Transforms and Pushes to Cache| G
    F -->|Transforms and Pushes to Cache| G

    G -->|Notifier is notified when new data hits the cache for a particular market pair| H[HyperLiquidAPI Client stream calls fetch_orderbook on new notification]
    H -->|Sends to OMS| I[OMS]

```

## Component Architecture

```mermaid
graph LR
     A[Hyperliquid WebSocket Handler] -->|Create Channels to send data from the websocket connection to a different thread for processing| B[Global Data Cache] 
     B--> |Inserted via constructor| C[HyperLiquidAPI]
     C -->|Sends preprocessed Orderbook Data to| D[OMS]

```

## Ordering Sequence Diagram

With Exchange Client we have the following process Create Exchange Client -> ClientOrderRequest -> Order -> Receive

## Notification Sequence Diagram

Each DashMap Entry has its own Notifier, thus implying a notification per orderbook for the market pair. Below is an example of how the Notifiers get instantiated.

Please see doc comments in connector commons() for what a notifier is.

```mermaid
graph LR
     A[Hyperliquid Global Cache] --> B[Intialises default Dashmap structure]
     B --> C[Global Cache receives latest data from Websocket sender to receiver mpsc channel] 
     C --> D@{ shape: diamond, label: "Does DashMap already contain a key for this index" }
     D -->|Yes| E[Take reference to existing notifier]
     D -->|No| F[Insert new data with newly constructed notifier]
     F --> G[Notify Channel]

```
