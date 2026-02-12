# Basic Usage

Demonstrates the fundamental Drasi pattern: **ApplicationSource → Cypher Query → ApplicationReaction**.

## What This Example Does

1. Creates an in-process `ApplicationSource` for programmatic data pushing
2. Defines a Cypher continuous query that selects Person nodes
3. Attaches an `ApplicationReaction` to consume query results
4. Pushes Person nodes and observes real-time query result changes
5. Demonstrates insert, update, and delete operations

## Prerequisites

- All Drasi Python packages built (`cd .. && make build-all` from the `python/` directory)

## Running

```bash
cd python
uv run python examples/basic-usage/main.py
```

## Key Concepts

- **ApplicationSource**: An in-process source that lets you push graph changes (nodes/relations) programmatically
- **PropertyMapBuilder**: Builds typed property maps for nodes and relations
- **Continuous Query**: Unlike traditional queries, Drasi queries run continuously and emit diffs when results change
- **ApplicationReaction**: An in-process reaction that collects query result changes for consumption

## Expected Output

Shows inserts, updates, and deletes flowing through the continuous query with result diffs.
