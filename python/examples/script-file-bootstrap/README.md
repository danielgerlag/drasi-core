# ScriptFile Bootstrap

Demonstrates loading initial data from JSONL files using `ScriptFileBootstrapProvider` before streaming begins.

## What This Example Does

1. Creates a temporary JSONL file with 5 Person nodes as initial data
2. Configures a `ScriptFileBootstrapProvider` to load it at startup
3. Defines a Cypher query that filters for engineers
4. After start, the bootstrapped engineers are already in the query result set
5. Pushes additional changes and observes them flowing through the query
6. Cleans up the temporary file on exit

## Prerequisites

- All Drasi Python packages built (`cd .. && make build-all` from the `python/` directory)

## Running

```bash
cd python
uv run python examples/script-file-bootstrap/main.py
```

## Key Concepts

- **Bootstrapping**: Pre-populating the query index with existing data so that the continuous query starts with a known state rather than an empty result set
- **ScriptFileBootstrapProvider**: Reads structured JSONL (JSON Lines) files containing graph nodes and relations
- **JSONL Format**: Each line is a JSON object with a `kind` field (`Header`, `Node`, `Relation`, `Finish`)
- **Bootstrap + Streaming**: After bootstrap completes, the `ApplicationSource` continues to push live changes

## JSONL Bootstrap File Format

```jsonl
{"kind": "Header", "start_time": "2024-01-01T00:00:00Z", "description": "..."}
{"kind": "Node", "id": "node-1", "labels": ["Label"], "properties": {"key": "value"}}
{"kind": "Node", "id": "node-2", "labels": ["Label"], "properties": {"key": "value"}}
{"kind": "Relation", "id": "rel-1", "labels": ["KNOWS"], "properties": {}, "start_id": "node-1", "end_id": "node-2"}
{"kind": "Finish", "description": "Bootstrap complete"}
```

## Expected Output

Shows bootstrapped data already present in query results, followed by live changes as new nodes are pushed.
