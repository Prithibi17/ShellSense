#!/usr/bin/env bash
# scripts/install.sh - Wrapper calling the root universal install.sh
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
exec "$SCRIPT_DIR/install.sh" "$@"
