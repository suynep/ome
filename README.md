# Order Matching Engine

> An OME featuring price/time priority matching methodology implemented with <3 and BTreeMaps


> Jump to [Reproducing the PDF example](https://github.com/suynep/ome?tab=readme-ov-file#reproducing-the-example-in-the-given-pdf) section

## Overview

This project implements a complete order matching engine similar to those used in electronic trading systems by exchanges, trading platforms, and brokerage firms. The engine matches buy and sell orders for financial instruments using strict price/time priority rules.

## Assumptions
1. *Market Orders* are *canceled* when there is no Order in the oppposing side
2. *Price* field is in Cents *(implemented as `u64` instead of `f32/f64` to avoid dealing with floating-point precision issues)*
3. *Timestamps* are POSIX time *(current implementation uses `u64` starting from `1`, however, since POSIX timestamps are `u64`s there should be a bijection between this implementation and the actual timestamp implementation)*

## Architecture

The system is built with three core modules:

### 1. Order Module (`order.rs`)
Defines the fundamental data structures:
- **Order**: Represents a trading order with ID, side (Buy/Sell), type (Limit/Market), price, quantity, and timestamp
- **Trade**: Records executed trades between matched orders
- **Comparison Functions**: Implements price/time priority logic for order matching

**Design Choices:**
- Uses `u64` for prices (in smallest units like cents) to avoid floating-point precision issues
- Enums for `Side` and `OrderType` ensure type safety
- Separate comparison functions for buy and sell orders maintain clear priority logic

### 2. Order Book Module (`order_book.rs`)
Manages active orders using price levels with FIFO queues:
- **Buy Orders (Bids)**: `BTreeMap<price, Vec<Order>>` iterated in descending price
- **Sell Orders (Asks)**: `BTreeMap<price, Vec<Order>>` iterated in ascending price
- **Time Priority**: Within each price level, orders are kept sorted by timestamp (earlier first)
- **Lazy Deletion**: Canceled orders are marked and skipped during matching

**Design Choices:**
- `BTreeMap` provides ordered traversal by price for predictable matching
- `HashMap` enables O(1) order lookup by ID for cancellation

### 3. Matching Engine Module (`matching_engine.rs`)
Carries out the matching process:
- Processes incoming orders
- Matches orders according to price/time priority
- Executes trades and updates the order book
- Tracks all executed trades

**Matching Logic:**
1. New orders are checked against the opposite side of the book
2. Best opposing orders are selected based on price/time priority
3. Trades are executed at the price of the order already in the book
4. Partial fills are supported (orders can match multiple times)
5. Unfilled limit order portions remain in the book

## Features

### Price/Time Priority
- **Price Priority**: Buy orders with higher prices and sell orders with lower prices are matched first
- **Time Priority**: Among orders at the same price, earlier orders (lower timestamp) have priority

### Order Types
- **Limit Orders**: Execute at specified price or better
- **Market Orders**: Execute immediately at best available price

### Matching Rules
- Orders must be on opposite sides (Buy vs Sell)
- For limit orders, buy price must be ≥ sell price to match
- Market orders match with any available opposing limit order (won't match if no orders exist in the book)
- Execution price is determined by the order already in the book (price/time priority)

## Building and Running

### Prerequisites
- Rust 2021 edition or later
- Cargo build system


### Repository Setup

```bash
git clone https://github.com/suynep/ome.git
cd ome/
```

### Build
```bash
cargo build
```

### Run
```bash
cargo run
```

### Test
The tests are added to each of the *aforementioned* modules under the `mod tests` augmented with `#[cfg(test)]` attribute *(prensently, 18 tests exist for unit testing)*. You can run them as:
```bash
cargo test
```
### HTTP API

The engine exposes a simple HTTP API using Axum.

#### Endpoints
- `GET /orderbook` → returns current bids and asks.
- `POST /orders` → submits a new order and returns executed trades + updated orderbook.
- `DELETE /orders/:id/cancel` → cancel an order by its `id`

#### Start the server
```bash
cargo run
```

Server runs on `http://localhost:61666`.

#### Examples

Submit a limit buy order for 100 units @ $10.00:
```bash
curl -s -X POST http://localhost:61666/orders \
  -H 'Content-Type: application/json' \
  -d '{"side":"Buy","order_type":"Limit","price":1000,"quantity":100}' | jq
```

Submit a market sell order for 50 units:
```bash
curl -s -X POST http://localhost:61666/orders \
  -H 'Content-Type: application/json' \
  -d '{"side":"Sell","order_type":"Market","quantity":50}' | jq
```

Fetch the current orderbook:
```bash
curl -s http://localhost:61666/orderbook | jq
```

Cancel an order by ID:
```bash
curl -X DELETE "http://localhost:61666/orders/<id>/cancel" | jq
```

## Reproducing the Example in the given `pdf`
#### Start the server
```bash
cargo run
```
#### Add the Buy and Sell orders (execute the following one-after-the-other)
> Note that we are using **Cents** as the primary currency (instead of dollars) in order to avoid floating point precision issues (for bulky order trades)

```bash
curl -X POST "http://localhost:61666/orders" -H "Content-Type: application/json" -d '{"side": "Buy", "order_type": "Limit", "price": 950, "quantity": 100}' | jq
```

*Note: To execute the above `cURL` request, you need to install `curl` and `jq` packages from your distros package manager*


```bash
curl -X POST "http://localhost:61666/orders" -H "Content-Type: application/json" -d '{"side": "Buy", "order_type": "Limit", "price": 900, "quantity": 200}' | jq
 ```


```bash
curl -X POST "http://localhost:61666/orders" -H "Content-Type: application/json" -d '{"side": "Sell", "order_type": "Limit", "price": 1050, "quantity": 150}' | jq
 ```

 ```bash
curl -X POST "http://localhost:61666/orders" -H "Content-Type: application/json" -d '{"side": "Sell", "order_type": "Limit", "price": 1000, "quantity": 100}' | jq
 ```

 > At this point, all orders (from the example in pdf) are added however none are resolved because of price mismatch

 #### Add the order that essentially executes a Trade

 ```bash
curl -X POST "http://localhost:61666/orders" -H "Content-Type: application/json" -d '{"side": "Buy", "order_type": "Limit", "price": 1050, "quantity": 150}' | jq
 ```

 **Note the response at this point** (especially the `trade` field, which shows the matches of each pre-existing order). The output you see is the one expected in the `pdf`. *QED*.

## Limitations
1. Persistent Storage not present as of now
2. Multi-threaded tests not implemented (though the functionality is implemented with multithreaded operations in mind)
3. Actual POSIX timestamps not implemented (see *Assumptions* section for the rationale)
