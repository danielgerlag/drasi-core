#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR/../.."

echo "Installing streamlit..."
uv add --dev streamlit
echo ""
echo "Setup complete. Run the app with:"
echo "  cd python"
echo "  uv run streamlit run examples/crypto-orderbook/app.py"
