# Crypto Order Book Example

A real-time multi-asset crypto exchange order book powered by **Drasi continuous queries** and **Streamlit**.

## What This Example Does

Users select an asset (**BTC**, **ETH**, or **SOL**) and submit **bid** (buy) and **ask** (sell) orders through a web UI. Each order creates a node and a `[:FOR]` relationship to its asset node. Drasi runs four continuous Cypher queries that maintain the order book state in real time:

| Query | Purpose |
|-------|---------|
| `open-bids` | Open bid orders connected to assets via `[:FOR]` |
| `open-asks` | Open ask orders connected to assets via `[:FOR]` |
| `matched-orders` | **Cross-node matching**: finds Bid/Ask pairs on the same Asset where `bid.price >= ask.price` |
| `all-trades` | All executed trades |

The **`matched-orders`** query is the core matching engine — it is a pure Cypher
relationship traversal that detects when a bid and ask on the same asset can be
matched. When it fires, a background worker automatically:
1. Marks the bid and ask as "filled" (UPDATE via ApplicationSource)
2. Creates a new Trade node (INSERT via ApplicationSource)
3. The `open-bids`/`open-asks` queries emit DELETE diffs, removing filled orders from the UI
4. The `all-trades` query emits an ADD diff, showing the new trade

```
┌─────────────┐     ┌──────────────┐     ┌─────────────────┐     ┌──────────┐
│  Streamlit   │────▶│ Application  │────▶│ Drasi Queries   │────▶│ Reactions│
│  UI (forms)  │     │ Source       │     │ (Cypher)        │     │ (streams)│
└─────────────┘     └──────────────┘     └─────────────────┘     └──────────┘
       ▲                                                               │
       └───────────────────── UI updates ◀─────────────────────────────┘
```

## Prerequisites

- **Python 3.10+** with the `drasi` packages installed (see the top-level `python/` README)
- **Streamlit** — installed automatically via `setup.sh`

No Docker containers are needed — this example uses only the in-memory ApplicationSource.

## Quick Start

```bash
# From the python/ directory:

# 1. Install streamlit (if not already)
./examples/crypto-orderbook/setup.sh

# 2. Run the app
uv run streamlit run examples/crypto-orderbook/app.py

# 3. Open the URL shown in terminal (typically http://localhost:8501)
```

## How to Use

1. **Pick an asset**: Select BTC, ETH, or SOL from the dropdown
2. **Submit a Bid**: Select "BID", enter a price and quantity, click Submit Order
3. **Submit an Ask**: Select "ASK", enter a price and quantity, click Submit Order
4. **Watch for matches**: If a bid price ≥ an ask price for the same asset, a trade executes automatically
5. **View trades**: The Trade History table shows executed trades for the selected asset
6. **All Assets Summary**: Expand to see order counts across all assets

### Example Scenario

1. Select **BTC**
2. Submit a **BID** at $50,000 for 1.0 BTC
3. Submit an **ASK** at $49,500 for 0.5 BTC
4. Since bid ($50,000) ≥ ask ($49,500), a trade executes at $49,500 for 0.5 BTC
5. Both orders disappear from the order book
6. The trade appears in Trade History
7. Switch to **ETH** — its order book is independent

## Architecture

All data lives in Drasi's in-memory graph — there is no external database.

### Graph Model

```
(:Asset {id, name})             — BTC, ETH, SOL (seeded on startup)
(:Bid {id, price, quantity, status, submitted_at})-[:FOR]->(:Asset)
(:Ask {id, price, quantity, status, submitted_at})-[:FOR]->(:Asset)
(:Trade {id, asset, price, quantity, bid_id, ask_id, executed_at})
```

### Cypher Queries

```cypher
-- Open Bids (via relationship traversal)
MATCH
  (b:Bid)-[:FOR]->(a:Asset)
WHERE
  b.status = 'open'
RETURN
  b.id AS id,
  b.price AS price,
  b.quantity AS quantity,
  a.id AS asset,
  b.submitted_at AS submitted_at

-- Open Asks (via relationship traversal)
MATCH
  (s:Ask)-[:FOR]->(a:Asset)
WHERE
  s.status = 'open'
RETURN
  s.id AS id,
  s.price AS price,
  s.quantity AS quantity,
  a.id AS asset,
  s.submitted_at AS submitted_at

-- Matched Orders (cross-node relationship query — the matching engine)
MATCH
  (b:Bid)-[:FOR]->(a:Asset)<-[:FOR]-(s:Ask)
WHERE
  b.status = 'open'
  AND s.status = 'open'
  AND b.price >= s.price
RETURN
  b.id AS bid_id,
  b.price AS bid_price,
  b.quantity AS bid_qty,
  s.id AS ask_id,
  s.price AS ask_price,
  s.quantity AS ask_qty,
  a.id AS asset

-- All Trades
MATCH
  (t:Trade)
RETURN
  t.id AS id,
  t.price AS price,
  t.quantity AS quantity,
  t.asset AS asset,
  t.bid_id AS bid_id,
  t.ask_id AS ask_id,
  t.executed_at AS executed_at
```

### Matching Engine

The matching engine is implemented as a **Drasi Cypher query** — not Python code.
The `matched-orders` query performs a cross-node relationship traversal:
`(b:Bid)-[:FOR]->(a:Asset)<-[:FOR]-(s:Ask)` to find bid/ask pairs on the same
asset where the bid price meets or exceeds the ask price. When this query fires,
a background worker consumes the match stream and executes the trade by updating
order statuses and inserting a Trade node — all through the ApplicationSource so
the other queries react immediately.

## Files

| File | Description |
|------|-------------|
| `app.py` | Streamlit application with Drasi integration |
| `setup.sh` | Installs streamlit dependency |
| `teardown.sh` | No-op (no external resources to clean up) |
| `README.md` | This file |
