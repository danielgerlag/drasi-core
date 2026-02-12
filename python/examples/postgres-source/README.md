# PostgreSQL Source Example

Stream live database changes from PostgreSQL into Drasi using logical replication (WAL).

## What This Example Does

1. Starts a PostgreSQL 16 instance with **logical replication** enabled
2. Creates an `orders` table with sample data
3. Configures a Drasi `PostgresSource` to stream WAL changes
4. Runs a continuous Cypher query that filters for high-value orders (`total_amount > 500`)
5. Prints matching results in real time as rows are inserted, updated, or deleted

## Prerequisites

- **Docker** (with Docker Compose v2)
- **Python 3.10+** with the `drasi` packages installed (see the top-level `python/` README)
- **asyncpg** — `pip install asyncpg` (only needed for `simulate_changes.py`)

## Quick Start

```bash
# 1. Start PostgreSQL
./setup.sh

# 2. Run the Drasi pipeline (keep this running)
python main.py

# 3. In another terminal, simulate database changes
python simulate_changes.py

# 4. Observe the output in the main.py terminal

# 5. Clean up
./teardown.sh
```

## How It Works

### PostgreSQL Logical Replication

PostgreSQL's [logical replication](https://www.postgresql.org/docs/current/logical-replication.html)
streams row-level changes (INSERT, UPDATE, DELETE) from a **publication** to subscribers.
Drasi uses this mechanism to capture changes in real time:

1. The Docker container starts PostgreSQL with `wal_level=logical`
2. `init.sql` creates a publication (`drasi_pub`) for the `orders` table
3. Drasi creates a replication slot (`drasi_orders_slot`) and subscribes to changes
4. Each row change is converted to a graph node operation and fed to the query engine

### The Cypher Query

```cypher
MATCH (o:orders)
WHERE o.total_amount > 500
RETURN o.id AS order_id, o.customer_name AS customer,
       o.product AS product, o.total_amount AS amount, o.status AS status
```

Each row in the `orders` table becomes a node labeled `orders`. The query continuously
evaluates against the current state and emits diffs when results change.

## Expected Output

When you run `main.py`, you'll see the initial high-value orders from `init.sql`:

```
PostgreSQL source started — listening for WAL changes
Query: orders with total_amount > 500
Press Ctrl+C to stop...

  High-value order: {order_id: 1, customer: "Alice", product: "Laptop", amount: 1299.99, status: "completed"}
  High-value order: {order_id: 3, customer: "Charlie", product: "Monitor", amount: 599.99, status: "shipped"}
```

When you run `simulate_changes.py`, you'll see additional updates:

```
  High-value order: {order_id: 4, customer: "Diana", product: "Tablet", amount: 799.99, status: "pending"}
  High-value order: {order_id: 1, customer: "Alice", product: "Laptop", amount: 1399.99, status: "shipped"}
```

## Files

| File | Description |
|------|-------------|
| `docker-compose.yml` | PostgreSQL 16 with logical replication enabled |
| `init.sql` | Creates the `orders` table, publication, and seed data |
| `setup.sh` | Starts the Docker container and waits for readiness |
| `teardown.sh` | Stops the container and removes volumes |
| `main.py` | Drasi pipeline: PostgresSource → Cypher query → ApplicationReaction |
| `simulate_changes.py` | Helper to INSERT/UPDATE/DELETE rows for testing |
