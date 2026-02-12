# PostgreSQL Source Example

Stream live database changes from PostgreSQL into Drasi using logical replication (WAL).

## What This Example Does

1. Starts a PostgreSQL 16 instance with **logical replication** enabled
2. Creates an `orders` table with sample data and `REPLICA IDENTITY FULL`
3. Configures a Drasi `PostgresSource` to stream WAL changes with table key mapping
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
2. `init.sql` creates a publication (`drasi_publication`) for the `orders` table
3. Drasi creates a replication slot (`drasi_orders_slot`) and subscribes to changes
4. Each row change is converted to a graph node operation and fed to the query engine

### Important Configuration

**Table keys**: You must configure `add_table_key("table_name", ["pk_column"])` so that
Drasi generates stable element IDs. Without this, INSERT/UPDATE/DELETE cannot be correlated
and updates/deletes will not produce results.

**Replica identity**: The table must have `REPLICA IDENTITY FULL` set so that UPDATE and DELETE
WAL messages include all column values (not just the primary key).

**Column types**: Use `DOUBLE PRECISION` or `REAL` for numeric columns used in Cypher
comparisons (e.g. `WHERE o.total_amount > 500`). PostgreSQL `DECIMAL`/`NUMERIC` types are
transmitted as text strings in WAL, which causes numeric comparisons to fail silently.

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

When you run `main.py`, it starts listening for WAL changes:

```
PostgreSQL source started — listening for WAL changes
Query: orders with total_amount > 500
Press Ctrl+C to stop...
```

When you run `simulate_changes.py`, you'll see ADD, UPDATE, and DELETE results:

```
  High-value order: {..., 'type': 'ADD', 'data': {'customer': 'Diana', 'amount': '799.99', ...}}
  High-value order: {..., 'type': 'UPDATE', 'data': {'customer': 'Diana', 'amount': '999.99', ...}}
  High-value order: {..., 'type': 'DELETE', 'data': {'customer': 'Diana', 'amount': '999.99', ...}}
```

> **Note**: Without bootstrap, only changes made _after_ the pipeline starts are tracked.
> Pre-existing rows in the database are not visible to the query engine until they are modified.

## Files

| File | Description |
|------|-------------|
| `docker-compose.yml` | PostgreSQL 16 with logical replication enabled |
| `init.sql` | Creates the `orders` table, publication, replica identity, and seed data |
| `setup.sh` | Starts the Docker container and waits for readiness |
| `teardown.sh` | Stops the container and removes volumes |
| `main.py` | Drasi pipeline: PostgresSource → Cypher query → ApplicationReaction |
| `simulate_changes.py` | Helper to INSERT/UPDATE/DELETE rows for testing |
