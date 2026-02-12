#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
echo "Starting PostgreSQL with logical replication..."
docker compose -f "$SCRIPT_DIR/docker-compose.yml" up -d --wait
echo "PostgreSQL is ready on localhost:5432"
echo "  Database: drasi_example"
echo "  User: drasi / drasi_pass"
