#  High-Performance Order Book *🦀*

A blazingly fast order book implementation in Rust with lock-free architecture and multi-exchange real-time data aggregation.

## ⚡ Performance

```
Benchmarks :
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
add_order:           1.4 µs   ⚡
match_orders:        886 ns   🔥
get_depth:           134 ns   💨
1000 orders:         197 µs   💪

Throughput: 1,128,668 orders/second
```

## 🏗️ Architecture

```
┌─────────────────────────────────────────┐
│   Multi-Exchange Aggregation 🌐        │
├─────────────────────────────────────────┤
│  Binance │ Coinbase │ Bybit            │
│    ↓          ↓         ↓               │
├─────────────────────────────────────────┤
│      Lock-Free Order Book ⚡            │
│  • DashMap (concurrent HashMap)         │
│  • SegQueue (lock-free FIFO)            │
│  • parking_lot::RwLock                  │
│  • AtomicU64 counters                   │
├─────────────────────────────────────────┤
│      WebSocket + REST API 📡            │
│  • Real-time updates (100ms)            │
│  • HTTP endpoints                       │
│  • CORS enabled                         │
└─────────────────────────────────────────┘
```

## 🎯 Features

- ✅ **Sub-millisecond latency** - 1.4µs order processing
- ✅ **Lock-free architecture** - DashMap + SegQueue + Atomics
- ✅ **Multi-exchange data** - Binance, Coinbase, Bybit
- ✅ **Real-time WebSocket** - 10 updates/second
- ✅ **REST API** - Full CRUD operations
- ✅ **Multi-coin support** - BTC, ETH, SOL
- ✅ **Thread-safe** - Concurrent order processing
- ✅ **Production-ready** - Proven with benchmarks

## 🚀 Quick Start

### Prerequisites
```bash
# Rust 1.70+
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Build & Run
```bash
# Clone repository
git clone https://github.com/OGGY418/order-book-hybrid
cd rust-order-book

# Build release version
cargo build --release

# Run server
RUST_LOG=info cargo run --release

# Server starts on:
# HTTP:      http://127.0.0.1:8080
# WebSocket: ws://127.0.0.1:8080/ws
```

## 📡 API Endpoints

### Health Check
```bash
GET /health

Response:
{
  "status": "healthy",
  "service": "order-book-hybrid"
}
```

### Get Order Book Depth
```bash
GET /depth

Response:
{
  "bids": [
    {"price": 43250.0, "quantity": 5.0},
    {"price": 43245.0, "quantity": 10.0}
  ],
  "asks": [
    {"price": 43255.0, "quantity": 3.0},
    {"price": 43260.0, "quantity": 7.0}
  ]
}
```

### Get Market Statistics
```bash
GET /stats

Response:
{
  "total_orders_created": 1000,
  "total_orders_matched": 500,
  "total_orders_cancelled": 50,
  "total_volume_traded": 50000.0,
  "best_bid": 43250.0,
  "best_ask": 43255.0,
  "spread": 5.0,
  "mid_price": 43252.5,
  "last_match_time": 1704988800000
}
```

### Place Order
```bash
POST /order
Content-Type: application/json

{
  "price": 43250.0,
  "quantity": 1.0,
  "user_id": "trader123",
  "side": "Buy",
  "order_type": "Limit"
}

Response:
{
  "order_id": "1",
  "filled_quantity": 0.5,
  "remaining_quantity": 0.5,
  "average_price": 43250.0,
  "fills": [
    {
      "trade_id": "1",
      "quantity": 0.5,
      "price": 43250.0,
      "maker_order_id": "100",
      "taker_order_id": "1",
      "timestamp": 1704988800000
    }
  ],
  "status": "PartiallyFilled"
}
```

### Cancel Order
```bash
DELETE /order
Content-Type: application/json

{
  "order_id": "1",
  "user_id": "trader123"
}

Response:
{
  "success": true,
  "remaining_quantity": 0.5,
  "filled_quantity": 0.5
}
```

### WebSocket Connection
```javascript
const ws = new WebSocket('ws://127.0.0.1:8080/ws');

ws.onmessage = (event) => {
  const data = JSON.parse(event.data);
  
  // DepthUpdate - Order book changes
  if (data.type === 'DepthUpdate') {
    console.log('Bids:', data.bids);
    console.log('Asks:', data.asks);
  }
  
  // StatsUpdate - Market statistics
  if (data.type === 'StatsUpdate') {
    console.log('Best Bid:', data.best_bid);
    console.log('Best Ask:', data.best_ask);
  }
};
```

## 🧪 Testing

### Run Benchmarks
```bash
cargo bench
```
### Sample output 
```bash
add_order               time:   [1.3982 µs 1.4365 µs 1.4801 µs]
match_orders            time:   [881.34 ns 886.02 ns 891.28 ns]
get_depth               time:   [132.89 ns 134.21 ns 136.53 ns]
high_frequency_1000     time:   [196.81 µs 197.10 µs 197.45 µs]
```

### Test API
```bash
# Health check
curl http://127.0.0.1:8080/health

# Get depth
curl http://127.0.0.1:8080/depth | jq

# Place order
curl -X POST http://127.0.0.1:8080/order \
  -H "Content-Type: application/json" \
  -d '{
    "price": 43250.0,
    "quantity": 1.0,
    "user_id": "test",
    "side": "Buy",
    "order_type": "Limit"
  }' | jq

# Get stats
curl http://127.0.0.1:8080/stats | jq
```

## 🔧 Technology Stack

- **Rust** - Systems programming language
- **Actix-web** - High-performance HTTP server
- **DashMap** - Lock-free concurrent HashMap
- **Crossbeam** - Lock-free data structures
- **parking_lot** - Fast synchronization primitives
- **tokio-tungstenite** - WebSocket client (Binance/Coinbase/Bybit)

## 📊 Performance Optimizations

1. **Lock-Free Structures**
   - `DashMap` for concurrent order access
   - `SegQueue` for FIFO ordering
   - `AtomicU64` for counters

2. **Compiler Optimizations**
   ```toml
   [profile.release]
   opt-level = 3
   lto = "fat"
   codegen-units = 1
   ```

3. **Smart Locking**
   - `RwLock` for BTreeMap (multiple readers)
   - Matching lock only during execution
   - Lock-free operations inside price levels

## 🎯 Use Cases

### Trading Bots
Connect your algorithmic trading bot via WebSocket for real-time data and REST API for order execution.

### Mobile Apps
Use as backend for iOS/Android trading applications.

### Web Platforms
Build your own frontend (React, Vue, Angular) connected to this backend.

### Market Analysis
Stream real-time data for analysis and visualization tools.




## 📝 License

MIT


**⭐ Star this repo if you find it useful!**

Built with 🦀 Rust...



