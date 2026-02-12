#!/usr/bin/env bash
set -euo pipefail

BASE_URL="http://localhost:8080"

echo "Sending node insert (sensor-1, temp=72.5 — below threshold)..."
curl -s -X POST "$BASE_URL/sources/sensor-source/events" \
  -H "Content-Type: application/json" \
  -d '{
    "operation": "insert",
    "element": {
      "type": "node",
      "id": "sensor-1",
      "labels": ["Sensor"],
      "properties": {"name": "Temperature Sensor A", "value": 72.5, "location": "Building A"}
    }
  }'
echo ""

sleep 1

echo "Sending node insert (sensor-2, temp=85.2 — above threshold)..."
curl -s -X POST "$BASE_URL/sources/sensor-source/events" \
  -H "Content-Type: application/json" \
  -d '{
    "operation": "insert",
    "element": {
      "type": "node",
      "id": "sensor-2",
      "labels": ["Sensor"],
      "properties": {"name": "Temperature Sensor B", "value": 85.2, "location": "Building B"}
    }
  }'
echo ""

sleep 1

echo "Sending node update (sensor-1, temp=92.1 — now above threshold)..."
curl -s -X POST "$BASE_URL/sources/sensor-source/events" \
  -H "Content-Type: application/json" \
  -d '{
    "operation": "update",
    "element": {
      "type": "node",
      "id": "sensor-1",
      "labels": ["Sensor"],
      "properties": {"name": "Temperature Sensor A", "value": 92.1, "location": "Building A"}
    }
  }'
echo ""

sleep 1

echo "Sending node update (sensor-2, temp=78.0 — now below threshold)..."
curl -s -X POST "$BASE_URL/sources/sensor-source/events" \
  -H "Content-Type: application/json" \
  -d '{
    "operation": "update",
    "element": {
      "type": "node",
      "id": "sensor-2",
      "labels": ["Sensor"],
      "properties": {"name": "Temperature Sensor B", "value": 78.0, "location": "Building B"}
    }
  }'
echo ""

sleep 1

echo "Sending node delete (sensor-1)..."
curl -s -X POST "$BASE_URL/sources/sensor-source/events" \
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
echo ""

echo "Done sending changes"
