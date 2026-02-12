# Drasi Python Examples

Each directory is a self-contained example with its own README, runnable script, and any required infrastructure setup.

## Examples

| Example | Description | Docker Required |
|---------|-------------|:---------------:|
| [basic-usage](basic-usage/) | Core pattern: ApplicationSource → Cypher Query → ApplicationReaction | No |
| [script-file-bootstrap](script-file-bootstrap/) | Load initial data from JSONL files before streaming | No |
| [multi-query-pipeline](multi-query-pipeline/) | Fan-out: one source, multiple queries and reactions | No |
| [log-reaction](log-reaction/) | Format query results with Handlebars templates | No |
| [event-log-monitoring](event-log-monitoring/) | Subscribe to lifecycle events and component logs | No |
| [http-source](http-source/) | Receive graph changes via HTTP endpoints | No |
| [postgres-source](postgres-source/) | Stream changes from PostgreSQL via WAL replication | **Yes** |

## Prerequisites

1. **Build all Python packages** from the `python/` directory:

   ```bash
   cd python
   uv sync
   make build-all
   ```

2. **Docker** (only for examples that require it — see table above)

## Running an Example

```bash
cd python

# For examples without Docker:
uv run python examples/basic-usage/main.py

# For examples with Docker:
cd examples/postgres-source
./setup.sh                              # Start infrastructure
cd ../..
uv run python examples/postgres-source/main.py   # Run the example
cd examples/postgres-source
./teardown.sh                           # Clean up
```

Each example's README has detailed, step-by-step instructions.
