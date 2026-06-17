# CEX — Centralized Exchange

A centralized crypto exchange built from scratch. The matching engine is written in Rust, the backend in TypeScript/Bun, and they communicate over Redis. Frontend is in progress.

---

## What's built

- Rust matching engine with per-symbol threads (BTC_USDT, ETH_USDT, SOL_USDT each run independently)
- Limit and market order matching with FIFO price-time priority
- Real-time orderbook and trade updates over WebSocket via Redis pub/sub
- JWT-authenticated REST API for placing, cancelling, and querying orders
- PostgreSQL for user accounts via Prisma

---

## Architecture

```
                        ┌─────────────────────────────────────┐
                        │           Rust Engine                │
                        │                                      │
                        │  reader thread                       │
                        │    └─ pops from Redis queue          │
                        │    └─ routes by symbol               │
                        │                                      │
                        │  BTC_USDT thread ── OrderBook        │
                        │  ETH_USDT thread ── OrderBook        │
                        │  SOL_USDT thread ── OrderBook        │
                        │                                      │
                        │  publishes to orderbook:* trades:*   │
                        └─────────────────┬───────────────────┘
                                          │ Redis
                         ┌────────────────┴───────────────────┐
                         │         TypeScript Backend          │
                         │                                     │
                         │  REST API (port 3000)               │
                         │    POST /auth/signup                │
                         │    POST /auth/signin                │
                         │    POST /exchange/order             │
                         │    GET  /exchange/depth/:symbol     │
                         │    GET  /exchange/balance           │
                         │    GET  /exchange/order/:id         │
                         │    DELETE /exchange/order/:id       │
                         │                                     │
                         │  WebSocket server (port 3001)       │
                         │    subscribe_orderbook              │
                         │    subscribe_trades                 │
                         └─────────────────────────────────────┘
```

The engine and backend never share memory. Everything goes through Redis — the backend pushes orders to a queue, the engine processes them and replies on a per-instance response queue. Orderbook and trade updates are broadcast over Redis pub/sub to any connected WebSocket clients.

---

## Engine design

The matching engine runs one thread per trading pair. Each thread owns its orderbook and `EngineState` (open orders, balances) — no mutexes, no shared state.

```
incoming queue
      │
  reader thread
      │
  HashMap<symbol, mpsc::Sender>
      │
  ┌───┴──────────────┐
  ▼                  ▼
BTC thread        ETH thread
OrderBook         OrderBook
EngineState       EngineState
```

The orderbook uses `BTreeMap<u64, PriceLevel>` where prices are stored as integers (multiplied by 10^8) to avoid float comparison issues. Bids iterate in reverse for best-bid-first, asks iterate forward for best-ask-first.

Order matching follows standard price-time priority:
- Incoming buy → walks asks from lowest price up
- Incoming sell → walks bids from highest price down
- Trade price is always the maker's (resting) price
- Partial fills are supported — unfilled limit orders rest in the book

---

## Tech stack

| Layer | Tech |
|---|---|
| Matching engine | Rust, BTreeMap orderbook, mpsc channels |
| Message broker | Redis (queues + pub/sub) |
| Backend | TypeScript, Bun, Express |
| Auth | JWT, bcrypt |
| Database | PostgreSQL, Prisma |
| Real-time | WebSocket (ws), Redis pub/sub |

---

## Running locally

**Prerequisites:** Rust, Bun, PostgreSQL, Docker (for Redis)

```bash
# start Redis
docker run -d -p 6379:6379 --name my-redis redis

# start the engine
cd engine
cargo run --release

# start the backend
cd backend
cp .env.example .env   # fill in your values
bun install
bun dev
```

**Environment variables (backend/.env):**

```env
DATABASE_URL="postgresql://user@localhost:5432/cex_db"
PORT=3000
JWT_SECRET="your-secret"
REDIS_URL=redis://localhost:6379
BACKEND_QUEUE_ID=main
ENGINE_TIMEOUT_MS=3000
```

---

## API

All exchange routes require `Authorization: Bearer <token>`.

**Auth**

```
POST /auth/signup   { username, password }
POST /auth/signin   { username, password }
```

**Exchange**

```
POST   /exchange/order            place a limit or market order
GET    /exchange/depth/:symbol    orderbook snapshot (top 20 levels)
GET    /exchange/balance          user balance
GET    /exchange/order/:orderId   order status
DELETE /exchange/order/:orderId   cancel an open order
```

**Order body:**
```json
{
  "type": "limit",
  "side": "buy",
  "symbol": "BTC_USDT",
  "qty": 1.0,
  "price": 50000
}
```

**WebSocket (port 3001)**

```json
// subscribe to live orderbook updates
{ "type": "subscribe_orderbook", "symbol": "BTC_USDT" }

// subscribe to trade feed
{ "type": "subscribe_trades", "symbol": "BTC_USDT" }

// server pushes these automatically
{ "type": "orderbook_update", "symbol": "BTC_USDT", "data": { "bids": [], "asks": [] } }
{ "type": "trade_update", "symbol": "BTC_USDT", "data": { ... } }
```

On subscribe you immediately get an `orderbook_snapshot` with the current state, then `orderbook_update` events stream in as orders are placed.

---

## Load test results

Tested with 30,000 orders across 3 pairs (10k each) in release mode:

```
Total orders:    30,000
Errors:          0
Send throughput: 344,828 orders/sec (Redis pipeline)
Engine throughput: ~5,000 orders/sec
Min latency:     0ms
p50 latency:     ~2ms
p99 latency:     ~15ms
```

---

## What's next

- [ ] Frontend (React) — order placement UI, live orderbook, trade history
- [ ] Order history persistence to PostgreSQL
- [ ] Balance deduction on order placement
- [ ] More trading pairs
- [ ] Candlestick/OHLCV data