# HTTP Source Example

Receive graph change events via HTTP and process them with continuous Cypher queries.

## What This Example Does

1. Starts an `HttpSource` listening on **port 8080** for change events
2. Runs a continuous Cypher query that filters for sensors with `value > 80`
3. Prints alerts whenever a sensor crosses the temperature threshold
4. Demonstrates insert, update, and delete operations via HTTP

No Docker or external infrastructure is required — the HttpSource itself is the HTTP server.

## Prerequisites

- **Python 3.10+** with the `drasi` packages installed (see the top-level `python/` README)
- **curl** (for `send_changes.sh`)

## Quick Start

```bash
# Terminal 1: Start the HTTP source
python main.py

# Terminal 2: Send sample events
./send_changes.sh
```

## HTTP Event Format

Send POST requests to `http://localhost:8080/sources/sensor-source/events` with JSON:

### Insert a Node

```bash
curl -X POST http://localhost:8080/sources/sensor-source/events \
  -H "Content-Type: application/json" \
  -d '{
    "operation": "insert",
    "element": {
      "type": "node",
      "id": "sensor-1",
      "labels": ["Sensor"],
      "properties": {"name": "Temp Sensor", "value": 85.0, "location": "Room 1"}
    }
  }'
```

### Update a Node

```bash
curl -X POST http://localhost:8080/sources/sensor-source/events \
  -H "Content-Type: application/json" \
  -d '{
    "operation": "update",
    "element": {
      "type": "node",
      "id": "sensor-1",
      "labels": ["Sensor"],
      "properties": {"name": "Temp Sensor", "value": 92.1, "location": "Room 1"}
    }
  }'
```

### Delete a Node

```bash
curl -X POST http://localhost:8080/sources/sensor-source/events \
  -H "Content-Type: application/json" \
  -d '{
    "operation": "delete",
    "element": {
      "type": "node",
      "id": "sensor-1",
      "labels": ["Sensor"],
      "properties": {}
    }
  }'
```

## Expected Output

When running `main.py` and then `send_changes.sh`:

```
HTTP source listening on http://0.0.0.0:8080
Send events to: POST /sources/sensor-source/events

Try: ./send_changes.sh
Press Ctrl+C to stop...

  Hot sensor alert: {sensor_name: "Temperature Sensor B", temperature: 85.2, location: "Building B"}
  Hot sensor alert: {sensor_name: "Temperature Sensor A", temperature: 92.1, location: "Building A"}
```

- `sensor-1` is initially inserted at 72.5 (below threshold) — no alert
- `sensor-2` is inserted at 85.2 (above threshold) — **alert**
- `sensor-1` is updated to 92.1 (now above threshold) — **alert**
- `sensor-2` is updated to 78.0 (below threshold) — removed from results
- `sensor-1` is deleted — removed from results

## Files

| File | Description |
|------|-------------|
| `main.py` | Drasi pipeline: HttpSource → Cypher query → ApplicationReaction |
| `send_changes.sh` | Shell script that sends sample events via curl |
| `README.md` | This file |
