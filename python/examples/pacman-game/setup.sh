#!/usr/bin/env bash
# Setup for the Pac-Man Drasi example (web UI)
# Requires: flask (installed as a dev dependency)
set -e
cd "$(dirname "$0")/../.."

echo "=== Pac-Man Drasi Example Setup ==="
echo "Syncing dependencies..."
uv sync
echo ""
echo "✓ Setup complete!"
echo ""
echo "Run with:  uv run python examples/pacman-game/game_web.py"
echo "Then open: http://localhost:5050"
