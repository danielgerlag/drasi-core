# Event & Log Monitoring

Demonstrates the **observability features** of DrasiLib: subscribing to component lifecycle events and logs.

## What This Example Does

1. Creates an `ApplicationSource`, a Cypher query, and an `ApplicationReaction`
2. Starts DrasiLib and checks component statuses
3. Subscribes to lifecycle events for source, query, and reaction
4. Subscribes to logs for source and query
5. Pushes data to trigger activity and observes events/logs in real-time
6. Lists all components and their statuses
7. Demonstrates the history + live streaming pattern

## Prerequisites

- All Drasi Python packages built (`cd .. && make build-all` from the `python/` directory)

## Running

```bash
cd python
uv run python examples/event-log-monitoring/main.py
```

## Key Concepts

### Event Subscriptions

Subscribe to lifecycle events for any component type:

```python
sub = await lib.subscribe_source_events("my-source")
sub = await lib.subscribe_query_events("my-query")
sub = await lib.subscribe_reaction_events("my-reaction")
```

Each subscription provides:
- **`.history`**: A list of past events at subscription time
- **`async for event in sub`**: A live stream of new events

### Log Subscriptions

Subscribe to component logs:

```python
sub = await lib.subscribe_source_logs("my-source")
sub = await lib.subscribe_query_logs("my-query")
sub = await lib.subscribe_reaction_logs("my-reaction")
```

Each log entry includes a `level` and `message`.

### Status Checking

Check individual component status:

```python
status = await lib.get_source_status("my-source")
status = await lib.get_query_status("my-query")
status = await lib.get_reaction_status("my-reaction")
```

List all components:

```python
sources = await lib.list_sources()      # [(id, status), ...]
queries = await lib.list_queries()
reactions = await lib.list_reactions()
```

## Expected Output

Shows component statuses, historical events from startup, and live events/logs as data flows through the system.
