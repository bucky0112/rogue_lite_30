#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
WEB_DIR="${PROJECT_ROOT}/dist/wasm32-unknown-unknown"

if [ ! -d "$WEB_DIR" ]; then
    echo "Error: Web build not found at ${WEB_DIR}" >&2
    echo "Please run: ./scripts/package_release.sh wasm32-unknown-unknown" >&2
    exit 1
fi

echo "Starting local web server..."
echo "========================================"
echo "Open your browser and visit:"
echo ""
echo "  http://localhost:8080"
echo ""
echo "Press Ctrl+C to stop the server"
echo "========================================"
echo ""

cd "$WEB_DIR"
python3 -m http.server 8080
