# Multi-Query Pipeline

Demonstrates the **fan-out pattern**: a single `ApplicationSource` feeding multiple independent Cypher queries, each with its own `ApplicationReaction`.

## Architecture

```
                           ┌─ Query A (qty < 10)    ─→ Reaction A (low stock alerts)
                           │
  ApplicationSource ───────┼─ Query B (price > 100) ─→ Reaction B (high value items)
  (inventory-source)       │
                           └─ Query C (all items)   ─→ Reaction C (full inventory)
```

## What This Example Does

1. Creates a single `ApplicationSource` for inventory data
2. Defines three different Cypher queries on the same source:
   - **Low Stock**: Products with quantity < 10
   - **High Value**: Products with price > $100
   - **All Items**: Complete product listing
3. Each query has its own `ApplicationReaction` to consume results independently
4. Pushes 5 products with varying prices and quantities
5. Shows how the same data change appears in different reaction streams
6. Updates a product and shows how it triggers different reactions

## Prerequisites

- All Drasi Python packages built (`cd .. && make build-all` from the `python/` directory)

## Running

```bash
cd python
uv run python examples/multi-query-pipeline/main.py
```

## Key Concepts

- **Fan-out Pattern**: One source feeds multiple queries, each detecting different patterns in the same data
- **Independent Reactions**: Each query has its own reaction, so different consumers get different views of the data
- **Selective Triggering**: An update only appears in reactions whose queries match the changed data

## Expected Output

Shows products being inserted and appearing in different reaction streams based on their properties (price, quantity). When Widget B's price is updated to $120, it newly appears in the high-value reaction.
