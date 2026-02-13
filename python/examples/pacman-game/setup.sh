#!/usr/bin/env bash
# Setup for the Pac-Man Drasi example
# No external dependencies needed — pygame-ce is installed as a dev dependency
set -e
cd "$(dirname "$0")/../.."

echo "=== Pac-Man Drasi Example Setup ==="
echo "Ensuring pygame-ce is installed..."
uv sync
echo ""
echo "✓ Setup complete!"
echo ""
echo "Run with:  uv run python examples/pacman-game/game.py"
